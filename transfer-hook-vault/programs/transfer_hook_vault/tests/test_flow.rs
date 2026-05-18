// Run with:  cargo test -- --nocapture
//
// Transfer hook flow:
//   Deposit  : user calls Token-2022 transfer_checked (user→vault) directly.
//              Hook fires, validates user whitelist. No vault in call stack → no re-entrancy.
//              Then user calls vault deposit ix to record accounting.
//
//   Withdraw : user calls vault withdraw ix (validates, approves user as delegate, records accounting).
//              User then calls Token-2022 transfer_checked (vault→user, user as delegate) directly.
//              Hook fires, validates user whitelist (delegate = index-3 authority). No re-entrancy.

use anchor_lang::{AccountDeserialize, InstructionData, ToAccountMetas};
use anchor_spl::associated_token::get_associated_token_address_with_program_id;
use litesvm::LiteSVM;
use solana_keypair::Keypair;
use solana_message::{Message, VersionedMessage};
use solana_signer::Signer;
use solana_transaction::versioned::VersionedTransaction;
use anchor_lang::solana_program::{instruction::{AccountMeta, Instruction}, pubkey::Pubkey, system_program};
use spl_token_2022_interface::ID as TOKEN_2022_ID;
use anchor_spl::associated_token::ID as ATA_PROGRAM_ID;

// ── PDA helpers ───────────────────────────────────────────────────────────────

fn program_id() -> Pubkey { transfer_hook_vault::id() }

fn config_pda(admin: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[b"vault_config", admin.as_ref()], &program_id()).0
}

fn whitelist_pda(user: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[b"whitelist_entry", user.as_ref()], &program_id()).0
}

fn extra_meta_list_pda(mint: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[b"extra-account-metas", mint.as_ref()], &program_id()).0
}

fn get_ata(owner: &Pubkey, mint: &Pubkey) -> Pubkey {
    get_associated_token_address_with_program_id(owner, mint, &TOKEN_2022_ID)
}

// ── logging helper ────────────────────────────────────────────────────────────

fn print_tx(label: &str, result: &Result<litesvm::types::TransactionMetadata, litesvm::types::FailedTransactionMetadata>) {
    match result {
        Ok(meta) => {
            println!("\n[{}] ✓ SUCCESS", label);
            for log in &meta.logs { println!("  {log}"); }
        }
        Err(failed) => {
            println!("\n[{}] ✗ FAILED  err={:?}", label, failed.err);
            for log in &failed.meta.logs { println!("  {log}"); }
        }
    }
}

// ── transaction helper ────────────────────────────────────────────────────────

fn send(
    svm: &mut LiteSVM,
    ix: Instruction,
    signers: &[&Keypair],
) -> Result<litesvm::types::TransactionMetadata, litesvm::types::FailedTransactionMetadata> {
    let payer = signers[0].pubkey();
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[ix], Some(&payer), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), signers).unwrap();
    svm.send_transaction(tx)
}

// ── vault instruction builders ────────────────────────────────────────────────

fn ix_initialize(admin: &Pubkey, mint: &Pubkey) -> Instruction {
    let config = config_pda(admin);
    Instruction {
        program_id: program_id(),
        accounts: transfer_hook_vault::accounts::InitializeVault {
            admin: *admin,
            config,
            mint: *mint,
            vault: get_ata(&config, mint),
            extra_account_meta_list: extra_meta_list_pda(mint),
            associated_token_program: ATA_PROGRAM_ID,
            token_program: TOKEN_2022_ID,
            system_program: system_program::id(),
        }.to_account_metas(None),
        data: transfer_hook_vault::instruction::Initialize {}.data(),
    }
}

fn ix_whitelist_user(admin: &Pubkey, user: &Pubkey, deposit_cap: u64) -> Instruction {
    Instruction {
        program_id: program_id(),
        accounts: transfer_hook_vault::accounts::WhitelistUser {
            admin: *admin,
            config: config_pda(admin),
            user: *user,
            whitelist_entry: whitelist_pda(user),
            system_program: system_program::id(),
        }.to_account_metas(None),
        data: transfer_hook_vault::instruction::WhitelistUser { deposit_cap }.data(),
    }
}

fn ix_mint_tokens(admin: &Pubkey, mint: &Pubkey, recipient: &Pubkey, amount: u64) -> Instruction {
    let config = config_pda(admin);
    Instruction {
        program_id: program_id(),
        accounts: transfer_hook_vault::accounts::MintTokens {
            admin: *admin,
            config,
            mint: *mint,
            recipient: *recipient,
            recipient_ata: get_ata(recipient, mint),
            token_program: TOKEN_2022_ID,
            associated_token_program: ATA_PROGRAM_ID,
            system_program: system_program::id(),
        }.to_account_metas(None),
        data: transfer_hook_vault::instruction::MintTokens { amount }.data(),
    }
}

/// Record a deposit (accounting only). The caller must have already executed
/// the Token-2022 transfer_checked (user→vault) which fires the hook.
fn ix_deposit(admin: &Pubkey, user: &Pubkey, amount: u64) -> Instruction {
    Instruction {
        program_id: program_id(),
        accounts: transfer_hook_vault::accounts::Deposit {
            user: *user,
            config: config_pda(admin),
            whitelist_entry: whitelist_pda(user),
            system_program: system_program::id(),
        }.to_account_metas(None),
        data: transfer_hook_vault::instruction::Deposit { amount }.data(),
    }
}

/// Validate, approve user as delegate, and record the withdrawal accounting.
/// The caller must then execute Token-2022 transfer_checked (vault→user) which fires the hook.
fn ix_withdraw(admin: &Pubkey, user: &Pubkey, mint: &Pubkey, amount: u64) -> Instruction {
    let config = config_pda(admin);
    Instruction {
        program_id: program_id(),
        accounts: transfer_hook_vault::accounts::Withdraw {
            user: *user,
            config,
            mint: *mint,
            vault: get_ata(&config, mint),
            whitelist_entry: whitelist_pda(user),
            token_program: TOKEN_2022_ID,
            associated_token_program: ATA_PROGRAM_ID,
            system_program: system_program::id(),
        }.to_account_metas(None),
        data: transfer_hook_vault::instruction::Withdraw { amount }.data(),
    }
}

fn ix_remove_whitelist_user(admin: &Pubkey, user: &Pubkey) -> Instruction {
    Instruction {
        program_id: program_id(),
        accounts: transfer_hook_vault::accounts::RemoveFromWhitelist {
            admin: *admin,
            user: *user,
            config: config_pda(admin),
            whitelist_entry: whitelist_pda(user),
            system_program: system_program::id(),
        }.to_account_metas(None),
        data: transfer_hook_vault::instruction::RemoveWhitelistUser {}.data(),
    }
}

// ── direct Token-2022 transfer_checked (fires the hook) ──────────────────────

/// Build a Token-2022 transfer_checked instruction with the hook extra accounts
/// appended. This fires the transfer hook. `authority` is the signer — either
/// the token account owner (deposit) or the approved delegate (withdrawal).
fn ix_token_transfer(
    source: &Pubkey,
    mint: &Pubkey,
    dest: &Pubkey,
    authority: &Pubkey, // owner or approved delegate
    amount: u64,
    decimals: u8,
) -> Instruction {
    use spl_token_2022_interface::instruction::transfer_checked as spl_tc;
    let mut ix = spl_tc(&TOKEN_2022_ID, source, mint, dest, authority, &[], amount, decimals).unwrap();
    // Extra accounts for the hook (must match ExtraAccountMetaList order):
    //   [4] extra_account_meta_list
    //   [5] hook_program  (extra meta 0 — needed by LiteSVM for CPI lookup)
    //   [6] whitelist_entry for `authority`  (extra meta 1)
    ix.accounts.push(AccountMeta::new_readonly(extra_meta_list_pda(mint), false));
    ix.accounts.push(AccountMeta::new_readonly(program_id(), false));
    ix.accounts.push(AccountMeta::new_readonly(whitelist_pda(authority), false));
    ix
}

// ── account read helpers ──────────────────────────────────────────────────────

fn read_config(svm: &LiteSVM, admin: &Pubkey) -> transfer_hook_vault::Config {
    let acc = svm.get_account(&config_pda(admin)).expect("config missing");
    transfer_hook_vault::Config::try_deserialize(&mut acc.data.as_ref()).unwrap()
}

fn read_whitelist_entry(svm: &LiteSVM, user: &Pubkey) -> transfer_hook_vault::WhitelistEntry {
    let acc = svm.get_account(&whitelist_pda(user)).expect("whitelist_entry missing");
    transfer_hook_vault::WhitelistEntry::try_deserialize(&mut acc.data.as_ref()).unwrap()
}

// ── SVM setup ─────────────────────────────────────────────────────────────────

fn make_svm() -> (LiteSVM, Keypair) {
    let mut svm = LiteSVM::new();
    svm.add_program(program_id(), include_bytes!("../../../target/deploy/transfer_hook_vault.so")).unwrap();
    let admin = Keypair::new();
    svm.airdrop(&admin.pubkey(), 10_000_000_000).unwrap();
    (svm, admin)
}

fn setup_vault(svm: &mut LiteSVM, admin: &Keypair) -> Keypair {
    let mint = Keypair::new();
    let result = send(svm, ix_initialize(&admin.pubkey(), &mint.pubkey()), &[admin, &mint]);
    print_tx("initialize", &result);
    result.expect("initialize failed");
    mint
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[test]
fn test_initialize() {
    let (mut svm, admin) = make_svm();
    let mint = setup_vault(&mut svm, &admin);

    let config = read_config(&svm, &admin.pubkey());
    assert_eq!(config.admin, admin.pubkey());
    assert_eq!(config.mint, mint.pubkey());
    assert!(!config.suspended);
    println!("  config.admin={} mint={}", config.admin, config.mint);
}

#[test]
fn test_whitelist_user() {
    let (mut svm, admin) = make_svm();
    let _mint = setup_vault(&mut svm, &admin);
    let user = Keypair::new();

    let result = send(&mut svm, ix_whitelist_user(&admin.pubkey(), &user.pubkey(), 500_000_000), &[&admin]);
    print_tx("whitelist_user", &result);
    result.expect("whitelist_user failed");

    let entry = read_whitelist_entry(&svm, &user.pubkey());
    assert_eq!(entry.deposit_cap, 500_000_000);
    assert!(matches!(entry.status, transfer_hook_vault::EntryStatus::Active));
}

#[test]
fn test_full_flow() {
    let (mut svm, admin) = make_svm();
    let mint = setup_vault(&mut svm, &admin);

    let user = Keypair::new();
    svm.airdrop(&user.pubkey(), 2_000_000_000).unwrap();

    // Whitelist user
    send(&mut svm, ix_whitelist_user(&admin.pubkey(), &user.pubkey(), 0), &[&admin])
        .expect("whitelist failed");

    // Give user tokens to deposit
    send(&mut svm, ix_mint_tokens(&admin.pubkey(), &mint.pubkey(), &user.pubkey(), 1_000_000_000), &[&admin])
        .expect("mint failed");

    let config = config_pda(&admin.pubkey());
    let vault_ata = get_ata(&config, &mint.pubkey());
    let user_ata = get_ata(&user.pubkey(), &mint.pubkey());
    let decimals = 9u8;
    let deposit_amount: u64 = 600_000_000;
    let withdraw_amount: u64 = 400_000_000;

    // ── DEPOSIT (two steps) ───────────────────────────────────────────────────
    // Step 1: User calls Token-2022 transfer_checked (user→vault).
    //         Hook fires at this point, validates user whitelist. No re-entrancy.
    let transfer_ix = ix_token_transfer(
        &user_ata, &mint.pubkey(), &vault_ata,
        &user.pubkey(), deposit_amount, decimals,
    );
    let result = send(&mut svm, transfer_ix, &[&user]);
    print_tx("deposit: token-2022 transfer (hook fires here)", &result);
    result.expect("deposit transfer failed");

    // Step 2: Vault records accounting
    let result = send(&mut svm, ix_deposit(&admin.pubkey(), &user.pubkey(), deposit_amount), &[&user]);
    print_tx("deposit: record balance", &result);
    result.expect("deposit record failed");

    let entry = read_whitelist_entry(&svm, &user.pubkey());
    assert_eq!(entry.balance_amount, deposit_amount);
    println!("  balance after deposit: {}", entry.balance_amount);

    // ── WITHDRAW (two steps) ──────────────────────────────────────────────────
    // Step 1: Vault validates, approves user as delegate, records accounting
    let result = send(&mut svm, ix_withdraw(&admin.pubkey(), &user.pubkey(), &mint.pubkey(), withdraw_amount), &[&user]);
    print_tx("withdraw: approve delegate + record balance", &result);
    result.expect("withdraw failed");

    let entry = read_whitelist_entry(&svm, &user.pubkey());
    assert_eq!(entry.balance_amount, deposit_amount - withdraw_amount);

    // Step 2: User calls Token-2022 transfer_checked as delegate (vault→user).
    //         Hook fires, validates user whitelist (user is the delegate = index-3). No re-entrancy.
    let transfer_ix = ix_token_transfer(
        &vault_ata, &mint.pubkey(), &user_ata,
        &user.pubkey(), withdraw_amount, decimals,
    );
    let result = send(&mut svm, transfer_ix, &[&user]);
    print_tx("withdraw: token-2022 transfer (hook fires here)", &result);
    result.expect("withdraw transfer failed");

    // Withdraw remaining balance
    let remainder = deposit_amount - withdraw_amount;
    send(&mut svm, ix_withdraw(&admin.pubkey(), &user.pubkey(), &mint.pubkey(), remainder), &[&user])
        .expect("withdraw remainder record failed");
    let transfer_ix = ix_token_transfer(
        &vault_ata, &mint.pubkey(), &user_ata,
        &user.pubkey(), remainder, decimals,
    );
    send(&mut svm, transfer_ix, &[&user]).expect("withdraw remainder transfer failed");

    let entry = read_whitelist_entry(&svm, &user.pubkey());
    assert_eq!(entry.balance_amount, 0);

    // Remove from whitelist (balance is zero)
    let result = send(&mut svm, ix_remove_whitelist_user(&admin.pubkey(), &user.pubkey()), &[&admin]);
    print_tx("remove_whitelist_user", &result);
    result.expect("remove failed");
    assert!(svm.get_account(&whitelist_pda(&user.pubkey())).is_none());
}

#[test]
fn test_hook_blocks_non_whitelisted_transfer() {
    let (mut svm, admin) = make_svm();
    let mint = setup_vault(&mut svm, &admin);

    let alice = Keypair::new(); // NOT whitelisted
    svm.airdrop(&alice.pubkey(), 1_000_000_000).unwrap();

    send(&mut svm, ix_mint_tokens(&admin.pubkey(), &mint.pubkey(), &alice.pubkey(), 500_000_000), &[&admin])
        .expect("mint failed");

    let config = config_pda(&admin.pubkey());
    let transfer_ix = ix_token_transfer(
        &get_ata(&alice.pubkey(), &mint.pubkey()),
        &mint.pubkey(),
        &get_ata(&config, &mint.pubkey()),
        &alice.pubkey(), 100_000_000, 9,
    );
    let result = send(&mut svm, transfer_ix, &[&alice]);
    print_tx("hook rejects non-whitelisted transfer (expected FAIL)", &result);
    assert!(result.is_err(), "hook should block non-whitelisted transfer");
}

#[test]
fn test_deposit_cap_exceeded() {
    let (mut svm, admin) = make_svm();
    let mint = setup_vault(&mut svm, &admin);

    let user = Keypair::new();
    svm.airdrop(&user.pubkey(), 2_000_000_000).unwrap();
    send(&mut svm, ix_whitelist_user(&admin.pubkey(), &user.pubkey(), 300_000_000), &[&admin])
        .expect("whitelist failed");
    send(&mut svm, ix_mint_tokens(&admin.pubkey(), &mint.pubkey(), &user.pubkey(), 500_000_000), &[&admin])
        .expect("mint failed");

    let config = config_pda(&admin.pubkey());
    let vault_ata = get_ata(&config, &mint.pubkey());
    let user_ata = get_ata(&user.pubkey(), &mint.pubkey());

    // Transfer 300 tokens to vault (hook passes)
    send(&mut svm, ix_token_transfer(&user_ata, &mint.pubkey(), &vault_ata, &user.pubkey(), 300_000_000, 9), &[&user])
        .expect("transfer failed");

    // Record deposit of 200 — under cap: OK
    let result = send(&mut svm, ix_deposit(&admin.pubkey(), &user.pubkey(), 200_000_000), &[&user]);
    print_tx("deposit 200 (under cap — expected OK)", &result);
    result.expect("should pass");

    // Try to record deposit of 200 more — now over cap (200+200=400 > 300): FAIL
    let result = send(&mut svm, ix_deposit(&admin.pubkey(), &user.pubkey(), 200_000_000), &[&user]);
    print_tx("deposit 200 more (over cap — expected FAIL)", &result);
    assert!(result.is_err());
}

#[test]
fn test_withdraw_exceeds_balance() {
    let (mut svm, admin) = make_svm();
    let mint = setup_vault(&mut svm, &admin);

    let user = Keypair::new();
    svm.airdrop(&user.pubkey(), 2_000_000_000).unwrap();
    send(&mut svm, ix_whitelist_user(&admin.pubkey(), &user.pubkey(), 0), &[&admin])
        .expect("whitelist failed");
    send(&mut svm, ix_mint_tokens(&admin.pubkey(), &mint.pubkey(), &user.pubkey(), 500_000_000), &[&admin])
        .expect("mint failed");

    let config = config_pda(&admin.pubkey());
    let vault_ata = get_ata(&config, &mint.pubkey());
    let user_ata = get_ata(&user.pubkey(), &mint.pubkey());

    send(&mut svm, ix_token_transfer(&user_ata, &mint.pubkey(), &vault_ata, &user.pubkey(), 200_000_000, 9), &[&user])
        .expect("transfer failed");
    send(&mut svm, ix_deposit(&admin.pubkey(), &user.pubkey(), 200_000_000), &[&user])
        .expect("deposit record failed");

    // Try to withdraw more than deposited
    let result = send(&mut svm, ix_withdraw(&admin.pubkey(), &user.pubkey(), &mint.pubkey(), 300_000_000), &[&user]);
    print_tx("withdraw 300 (only 200 deposited — expected FAIL)", &result);
    assert!(result.is_err());
}

#[test]
fn test_remove_user_with_balance_fails() {
    let (mut svm, admin) = make_svm();
    let mint = setup_vault(&mut svm, &admin);

    let user = Keypair::new();
    svm.airdrop(&user.pubkey(), 2_000_000_000).unwrap();
    send(&mut svm, ix_whitelist_user(&admin.pubkey(), &user.pubkey(), 0), &[&admin])
        .expect("whitelist failed");
    send(&mut svm, ix_mint_tokens(&admin.pubkey(), &mint.pubkey(), &user.pubkey(), 500_000_000), &[&admin])
        .expect("mint failed");

    let config = config_pda(&admin.pubkey());
    let vault_ata = get_ata(&config, &mint.pubkey());
    let user_ata = get_ata(&user.pubkey(), &mint.pubkey());

    send(&mut svm, ix_token_transfer(&user_ata, &mint.pubkey(), &vault_ata, &user.pubkey(), 200_000_000, 9), &[&user])
        .expect("transfer failed");
    send(&mut svm, ix_deposit(&admin.pubkey(), &user.pubkey(), 200_000_000), &[&user])
        .expect("deposit record failed");

    let result = send(&mut svm, ix_remove_whitelist_user(&admin.pubkey(), &user.pubkey()), &[&admin]);
    print_tx("remove with non-zero balance (expected FAIL)", &result);
    assert!(result.is_err());
    assert!(svm.get_account(&whitelist_pda(&user.pubkey())).is_some());
}
