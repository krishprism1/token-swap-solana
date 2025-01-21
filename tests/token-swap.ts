import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Keypair, PublicKey, SystemProgram } from "@solana/web3.js";
import { TokenSwap } from "../target/types/token_swap";
import {
  TOKEN_PROGRAM_ID,
  createMint,
  createAccount,
  mintTo,
  getAssociatedTokenAddress,
  createAssociatedTokenAccountInstruction,
} from "@solana/spl-token";
import { bs58 } from "@coral-xyz/anchor/dist/cjs/utils/bytes";
import { assert } from "chai";

describe("token_swap", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.TokenSwap as Program<TokenSwap>;
  const project_spl_authority = provider.wallet;

  const PRIVATE_KEY_BASE58 = "2AYsnuQigNBMtssNknQRQx1abP8B6K3TYYgReqrq2LZiBCXY7uyminH2YQ5LcSpFfxYVJsasNz3QyHdW8C7mdPar";
  const PRIVATE_KEY = bs58.decode(PRIVATE_KEY_BASE58);
  const wallet = Keypair.fromSecretKey(PRIVATE_KEY);

  let userSplAccount: PublicKey;
  let projectSplAccount = new PublicKey("CWjKYqg7yucQURrCsMrdK3MpJm2H3YZAiBqY45BkB8vD");
  const solUsdPriceFeedAccount = new PublicKey("7UVimffxr9ow1uXYxsr4LHAcV58mLzhmwaeKvJ1pjLiE");

  const lamportsToPay = 10; // 0.0.00000001 SOL in lamports
  const connection = provider.connection;

  before(async () => {
    const splMint = new PublicKey("7WWz3pdvJiBg9eW1imHCQDXWL19vLA83JWUeV2W2ZgBQ");
    // Get the associated token account address
    userSplAccount = await getAssociatedTokenAddress(
      splMint,                // SPL token mint
      wallet.publicKey        // Owner of the account
    );

    // Check if the associated token account already exists
    const accountInfo = await connection.getAccountInfo(userSplAccount);
    if (!accountInfo) {
      // Create an associated token account if it doesn't exist
      const transaction = new anchor.web3.Transaction().add(
        createAssociatedTokenAccountInstruction(
          wallet.publicKey,    // Payer
          userSplAccount,      // Associated token account
          wallet.publicKey,    // Owner
          splMint              // SPL token mint
        )
      );

      // Send and confirm the transaction
      await provider.sendAndConfirm(transaction, [wallet]);
      console.log("Created user SPL account:", userSplAccount.toBase58());
    }
  });

  it("Buys SPL tokens with SOL", async () => {
    const tx = await program.methods
      .buySplWithSol(new anchor.BN(lamportsToPay))
      .accounts({
        user: wallet.publicKey,
        projectSolAccount: project_spl_authority.publicKey,
        projectSplAta: projectSplAccount,
        projectSplAuthority: project_spl_authority.publicKey,
        userSplAta: userSplAccount,
        priceUpdate: solUsdPriceFeedAccount
      })
      .signers([wallet])
      .rpc();

    console.log("Transaction signature:", tx);

    // Fetch the user's SPL token account balance
    const userSplAccountInfo = await connection.getParsedAccountInfo(
      userSplAccount
    );
    const balance = userSplAccountInfo.value?.data["parsed"]["info"]["tokenAmount"]["uiAmount"];
    console.log("User SPL Token Balance:", balance);
  });

  it("Fails to buy SPL tokens when the project wallet has insufficient balance", async () => {
    const lamportsToPay = 1_000_000_000; // 1 SOL in lamports
    try {
      const tx = await program.methods
        .buySplWithSol(new anchor.BN(lamportsToPay))
        .accounts({
          user: wallet.publicKey,
          projectSolAccount: project_spl_authority.publicKey,
          projectSplAta: projectSplAccount,
          projectSplAuthority: project_spl_authority.publicKey,
          userSplAta: userSplAccount,
          priceUpdate: solUsdPriceFeedAccount,
        })
        .signers([wallet])
        .rpc();

      // If the transaction succeeds unexpectedly, fail the test
      assert.fail("Expected transaction to fail due to insufficient SPL balance, but it succeeded");
    } catch (error) {
      // Parse the Anchor error and assert the error code
      const anchorError = error as anchor.AnchorError;
      assert.strictEqual(anchorError.error.errorCode.number, 6000, "Expected error code 6000 (InsufficientSPLBalance)");
      assert.strictEqual(
        anchorError.error.errorMessage,
        "Not enough SPL tokens in project wallet.",
        "Expected error message about insufficient SPL tokens"
      );

      console.log("Transaction failed as expected with error:", anchorError.error.errorMessage);
    }
  });
});
