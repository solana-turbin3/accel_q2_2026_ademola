
use {
    anchor_lang::{solana_program::instruction::Instruction, InstructionData, ToAccountMetas},
    litesvm::LiteSVM,
    solana_message::{Message, VersionedMessage},
    solana_signer::Signer,
    solana_keypair::Keypair,
    solana_transaction::versioned::VersionedTransaction,
};

fn setup() -> (LiteSVM, Keypair){
    let program_id = trasnfer_hook_vault::id();
    let payer = Keypair::new();
    let mut svm = LiteSVM::new();
    let bytes = include_bytes!("../../../target/deploy/trasnfer_hook_vault.so");
    svm.add_program(program_id, bytes).unwrap();
    svm.airdrop(&payer.pubkey(), 1_000_000_000).unwrap();
    
    
    assert!(res.is_ok());
    return (svm, payer)
}

#[test]
fn test_initialize() {
    let (svm, payer) = setup();
    let instruction = Instruction::new_with_bytes(
        program_id,
        &trasnfer_hook_vault::instruction::Initialize {}.data(),
        trasnfer_hook_vault::accounts::Initialize {}.to_account_metas(None),
    );

    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[instruction], Some(&payer.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[payer]).unwrap();

    let res = svm.send_transaction(tx);
}

#[test]
fn test_add_to_whitelist() {
    let payer = Keypair::new();
}