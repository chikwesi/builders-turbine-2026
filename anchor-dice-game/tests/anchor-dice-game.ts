import * as anchor from "@anchor-lang/core";
import { Program } from "@anchor-lang/core";
import { AnchorDiceGame } from "../target/types/anchor_dice_game";
import {
  Keypair,
  LAMPORTS_PER_SOL,
  PublicKey,
  SystemProgram,
  SYSVAR_INSTRUCTIONS_PUBKEY,
  Transaction,
} from "@solana/web3.js";
import { BN } from "bn.js";
import { SYSTEM_PROGRAM_ID } from "@anchor-lang/core/dist/cjs/native/system";
import { expect } from "chai";
import { createHash, randomBytes } from "crypto";

const FINALITY = "confirmed";

const SECRET_BYTES = randomBytes(32);
const HASHED_SECRET = createHash("sha256").update(SECRET_BYTES).digest();

const ROLL_GUESS = 50;
const WAGER_AMOUNT = BigInt(LAMPORTS_PER_SOL / 100);
const EDGE_BPS = 150;

describe("dice-game", () => {
  const prov = anchor.AnchorProvider.env();
  anchor.setProvider(prov);

  const { connection: conn } = prov;
  const diceProgram = anchor.workspace.AnchorDiceGame as Program<AnchorDiceGame>;

  const houseKeypair = Keypair.generate();
  const playerKeypair = Keypair.generate();
  const betSeed = new BN(randomBytes(16));

  const [vaultPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("vault"), houseKeypair.publicKey.toBuffer()],
    diceProgram.programId,
  );

  const [betPda] = PublicKey.findProgramAddressSync(
    [
      Buffer.from("bet"),
      vaultPda.toBuffer(),
      playerKeypair.publicKey.toBuffer(),
      betSeed.toBuffer("le", 16),
    ],
    diceProgram.programId,
  );

  const waitForTx = async (txSig: string): Promise<void> => {
    const recentBlockhash = await conn.getLatestBlockhash();
    await conn.confirmTransaction({ signature: txSig, ...recentBlockhash }, FINALITY);
  };

  it("Airdrop", async () => {
    await Promise.all(
      [houseKeypair, playerKeypair].map(({ publicKey }) =>
        anchor.getProvider().connection.requestAirdrop(publicKey, 1000 * LAMPORTS_PER_SOL).then(waitForTx)
      )
    );
  });

  it("initialize =>", async () => {
    await diceProgram.methods
      .initialize(new BN(100 * LAMPORTS_PER_SOL))
      .accountsStrict({
        house: houseKeypair.publicKey,
        vault: vaultPda,
        systemProgram: SYSTEM_PROGRAM_ID,
      })
      .signers([houseKeypair])
      .rpc()
      .then(waitForTx);
  });

  it("place bet =>", async () => {
    await diceProgram.methods
      .placeBet(betSeed, new BN(WAGER_AMOUNT), ROLL_GUESS, Array.from(HASHED_SECRET))
      .accountsStrict({
        player: playerKeypair.publicKey,
        house: houseKeypair.publicKey,
        vault: vaultPda,
        bet: betPda,
        systemProgram: SYSTEM_PROGRAM_ID,
      })
      .signers([playerKeypair])
      .rpc()
      .then(waitForTx);
  });

  it("refund bet fail => cannot refund before timeout", async () => {
    try {
      await diceProgram.methods
        .refundBet()
        .accountsStrict({
          player: playerKeypair.publicKey,
          house: houseKeypair.publicKey,
          vault: vaultPda,
          bet: betPda,
          systemProgram: SystemProgram.programId,
        })
        .signers([playerKeypair])
        .rpc();
      throw new Error("Expected refund to fail but it succeeded");
    } catch (err) {
      if (!`${err}`.includes("TimeoutNotReached")) throw err;
    }
  });

  it("resolve bet fail => ", async () => {
    const [fakeRevealIx, resolveIx] = await Promise.all([
      diceProgram.methods
        .reveal(randomBytes(32))
        .accountsStrict({ house: houseKeypair.publicKey })
        .signers([houseKeypair])
        .instruction(),
      diceProgram.methods
        .resolveBet()
        .accountsStrict({
          house: houseKeypair.publicKey,
          player: playerKeypair.publicKey,
          vault: vaultPda,
          bet: betPda,
          instructionSysvar: SYSVAR_INSTRUCTIONS_PUBKEY,
          systemProgram: SYSTEM_PROGRAM_ID,
        })
        .instruction(),
    ]);

    const badTx = new Transaction().add(fakeRevealIx, resolveIx);

    let didFail = false;
    try {
      await prov.sendAndConfirm(badTx, [houseKeypair]);
    } catch (err) {
      didFail = true;
      expect(err).to.be.instanceOf(Error);
    }

    if (!didFail) throw new Error("Expected resolve transaction to fail but it succeeded");
  });

  it("resolve bet => ", async () => {
    const [revealIx, resolveIx] = await Promise.all([
      diceProgram.methods
        .reveal(SECRET_BYTES)
        .accountsStrict({ house: houseKeypair.publicKey })
        .signers([houseKeypair])
        .instruction(),
      diceProgram.methods
        .resolveBet()
        .accountsStrict({
          house: houseKeypair.publicKey,
          player: playerKeypair.publicKey,
          vault: vaultPda,
          bet: betPda,
          instructionSysvar: SYSVAR_INSTRUCTIONS_PUBKEY,
          systemProgram: SYSTEM_PROGRAM_ID,
        })
        .instruction(),
    ]);

    const settleTx = new Transaction().add(revealIx, resolveIx);
    await waitForTx(await prov.sendAndConfirm(settleTx, [houseKeypair]));

    expect(await conn.getAccountInfo(betPda)).to.be.null;
  });
});
