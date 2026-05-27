import * as anchor from "@coral-xyz/anchor";
import { BN, Program } from "@coral-xyz/anchor";
import {
  init,
  runTask,
  taskKey,
  taskQueueAuthorityKey,
} from "@helium/tuktuk-sdk";
import os from "node:os";
import path from "node:path";
import {
  createMint,
  getAssociatedTokenAddress,
  getOrCreateAssociatedTokenAccount,
  mintTo,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import {
  Keypair,
  LAMPORTS_PER_SOL,
  PublicKey,
  SystemProgram,
  Transaction,
} from "@solana/web3.js";
import { assert } from "chai";
import { ScheduledEscrow } from "../../../target/types/scheduled_escrow";

// Allow the test to run directly with ts-mocha, not only through `anchor test`.
process.env.ANCHOR_PROVIDER_URL ??= "https://api.devnet.solana.com";
process.env.ANCHOR_WALLET ??= path.join(
  os.homedir(),
  ".config",
  "solana",
  "id.json",
);

describe("scheduled-escrow", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const connection = provider.connection;
  const program = anchor.workspace.ScheduledEscrow as Program<ScheduledEscrow>;
  const payer = (provider.wallet as anchor.Wallet).payer;

  const maker = Keypair.generate();
  const taker = Keypair.generate();

  let mintA: PublicKey;
  let mintB: PublicKey;
  let makerAtaA: PublicKey;
  let takerAtaB: PublicKey;

  const depositAmount = new BN(1_000_000);
  const receiveAmount = new BN(500_000);

  // Each flow gets its own seed so tests are independent
  const takeSeed = new BN(1);
  const refundSeed = new BN(2);
  const scheduleSeed = new BN(3);
  const expiryDuration = new BN(15);

  const escrowPda = (seed: BN, authority: PublicKey) =>
    PublicKey.findProgramAddressSync(
      [
        Buffer.from("escrow"),
        authority.toBuffer(),
        seed.toArrayLike(Buffer, "le", 8),
      ],
      program.programId,
    )[0];

  const [queueAuthority] = PublicKey.findProgramAddressSync(
    [Buffer.from("queue_authority")],
    program.programId,
  );

  async function fundAccount(recipient: PublicKey, amount: number) {
    const tx = new Transaction().add(
      SystemProgram.transfer({
        fromPubkey: payer.publicKey,
        toPubkey: recipient,
        lamports: amount,
      }),
    );
    const sig = await provider.sendAndConfirm(tx, []);
    return sig;
  }

  async function sleep(ms: number) {
    await new Promise((resolve) => setTimeout(resolve, ms));
  }

  async function waitForExpiry(expiry: number, bufferMs = 5_000) {
    const waitMs = Math.max(0, expiry * 1000 - Date.now() + bufferMs);
    if (waitMs > 0) {
      await sleep(waitMs);
    }
  }

  before(async () => {
    await Promise.all([
      fundAccount(maker.publicKey, Math.floor(0.3 * LAMPORTS_PER_SOL)),
      fundAccount(taker.publicKey, Math.floor(0.3 * LAMPORTS_PER_SOL)),
    ]);
    await new Promise((r) => setTimeout(r, 3000));

    mintA = await createMint(connection, payer, payer.publicKey, null, 6);
    mintB = await createMint(connection, payer, payer.publicKey, null, 6);

    makerAtaA = (
      await getOrCreateAssociatedTokenAccount(
        connection,
        payer,
        mintA,
        maker.publicKey,
      )
    ).address;

    takerAtaB = (
      await getOrCreateAssociatedTokenAccount(
        connection,
        payer,
        mintB,
        taker.publicKey,
      )
    ).address;

    await mintTo(connection, payer, mintA, makerAtaA, payer, 10_000_000);
    await mintTo(connection, payer, mintB, takerAtaB, payer, 10_000_000);
  });

  // --- make ------------------------------------------------------------------

  describe("make-escrow", () => {
    it("deposits token A into vault and initialises escrow state", async () => {
      const escrow = escrowPda(takeSeed, maker.publicKey);

      await program.methods
        .make(takeSeed, depositAmount, expiryDuration, receiveAmount)
        .accountsPartial({
          maker: maker.publicKey,
          mintA,
          mintB,
          makerAtaA,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([maker])
        .rpc();

      const escrowAccount = await program.account.escrow.fetch(escrow);
      assert.ok(escrowAccount.maker.equals(maker.publicKey));
      assert.ok(escrowAccount.mintA.equals(mintA));
      assert.ok(escrowAccount.mintB.equals(mintB));
      assert.equal(escrowAccount.receive.toString(), receiveAmount.toString());

      const vault = await getAssociatedTokenAddress(mintA, escrow, true);
      const vaultBalance = await connection.getTokenAccountBalance(vault);
      assert.equal(vaultBalance.value.amount, depositAmount.toString());
    });
  });

  // --- take ------------------------------------------------------------------

  describe("take-escrow", () => {
    it("taker sends token B to maker and receives token A from vault", async () => {
      const escrow = escrowPda(takeSeed, maker.publicKey);
      const vault = await getAssociatedTokenAddress(mintA, escrow, true);

      const takerAtaA = (
        await getOrCreateAssociatedTokenAccount(
          connection,
          payer,
          mintA,
          taker.publicKey,
        )
      ).address;
      const makerAtaB = (
        await getOrCreateAssociatedTokenAccount(
          connection,
          payer,
          mintB,
          maker.publicKey,
        )
      ).address;

      await program.methods
        .take()
        .accountsPartial({
          taker: taker.publicKey,
          maker: maker.publicKey,
          mintA,
          mintB,
          takerAtaA,
          takerAtaB,
          makerAtaB,
          escrow,
          vault,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([taker])
        .rpc();

      const takerBalanceA = await connection.getTokenAccountBalance(takerAtaA);
      assert.equal(takerBalanceA.value.amount, depositAmount.toString());

      const makerBalanceB = await connection.getTokenAccountBalance(makerAtaB);
      assert.equal(makerBalanceB.value.amount, receiveAmount.toString());

      assert.isNull(await connection.getAccountInfo(escrow));
      assert.isNull(await connection.getAccountInfo(vault));
    });
  });

  // --- refund ----------------------------------------------------------------

  describe("refund-escrow", () => {
    before(async () => {
      await program.methods
        .make(refundSeed, depositAmount, expiryDuration, receiveAmount)
        .accountsPartial({
          maker: maker.publicKey,
          mintA,
          mintB,
          makerAtaA,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([maker])
        .rpc();
    });

    it("returns token A from vault to maker and closes escrow", async () => {
      const escrow = escrowPda(refundSeed, maker.publicKey);
      const vault = await getAssociatedTokenAddress(mintA, escrow, true);

      const balanceBefore = await connection.getTokenAccountBalance(makerAtaA);

      await program.methods
        .manualRefund()
        .accountsPartial({
          maker: maker.publicKey,
          mintA,
          makerAtaA,
          escrow,
          vault,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([maker])
        .rpc();

      const balanceAfter = await connection.getTokenAccountBalance(makerAtaA);
      assert.equal(
        Number(balanceAfter.value.amount) - Number(balanceBefore.value.amount),
        depositAmount.toNumber(),
      );

      assert.isNull(await connection.getAccountInfo(escrow));
      assert.isNull(await connection.getAccountInfo(vault));
    });
  });

  // --- schedule --------------------------------------------------------------

  describe("schedule-refund", () => {
    const taskQueue = new PublicKey(
      "JCLv1EJLzgK6MQXhYEVpKSUu2APS5qiPMNCEgrcmqVNS", //original task queue ID of my tuktuk scheduler
    );
    const taskID = (Date.now() % 60_000) + 1;

    before(async () => {
      await program.methods
        .make(scheduleSeed, depositAmount, expiryDuration, receiveAmount)
        .accountsPartial({
          maker: maker.publicKey,
          mintA,
          mintB,
          makerAtaA,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([maker])
        .rpc();
    });

    it("queues and executes a timed refund task on the tuktuk task queue", async () => {
      const tuktukProgram = await init(provider);
      const escrow = escrowPda(scheduleSeed, maker.publicKey);
      const vault = await getAssociatedTokenAddress(mintA, escrow, true);
      const taskQueueAuthority = taskQueueAuthorityKey(
        taskQueue,
        queueAuthority,
      )[0];
      const task = taskKey(taskQueue, taskID)[0];

      const balanceBeforeRefund = await connection.getTokenAccountBalance(
        makerAtaA,
      );

      await program.methods
        .schedule(taskID)
        .accountsPartial({
          maker: maker.publicKey,
          mintA,
          makerAta: makerAtaA,
          escrow,
          vault,
          taskQueue,
          taskQueueAuthority,
          task,
          queueAuthority,
          tokenProgram: TOKEN_PROGRAM_ID,
          tuktukProgram: tuktukProgram.programId,
        })
        .signers([maker])
        .rpc({ skipPreflight: true });

      const taskAccount = await connection.getAccountInfo(task);
      assert.isNotNull(taskAccount, "task account should be created by tuktuk");

      const escrowAccount = await program.account.escrow.fetch(escrow);
      await waitForExpiry(Number(escrowAccount.expiry));

      const crankInstructions = await runTask({
        program: tuktukProgram,
        task,
        crankTurner: payer.publicKey,
      });

      await provider.sendAndConfirm(
        new Transaction().add(...crankInstructions),
        [],
      );

      const balanceAfterRefund = await connection.getTokenAccountBalance(
        makerAtaA,
      );
      assert.equal(
        Number(balanceAfterRefund.value.amount) -
          Number(balanceBeforeRefund.value.amount),
        depositAmount.toNumber(),
      );

      assert.isNull(await connection.getAccountInfo(escrow));
      assert.isNull(await connection.getAccountInfo(vault));
    });
  });
});
