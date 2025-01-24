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
  const wallet = provider.wallet;

  const PRIVATE_KEY_BASE58 = "2AYsnuQigNBMtssNknQRQx1abP8B6K3TYYgReqrq2LZiBCXY7uyminH2YQ5LcSpFfxYVJsasNz3QyHdW8C7mdPar";
  const PRIVATE_KEY = bs58.decode(PRIVATE_KEY_BASE58);
  const project_spl_authority = Keypair.fromSecretKey(PRIVATE_KEY);

  let userSplAccount: PublicKey;
  let projectUsdcAta= new PublicKey("853pGn1rkd7ATK5ajgBD7vAYPxWDcZ9QHNi3kXFHwhJJ");
  let projectSplAccount = new PublicKey("2niaehib38tpaE1zybHC6YZUXYcFDk5Xj9jjzgWR3tDu");
  const solUsdPriceFeedAccount = new PublicKey("7UVimffxr9ow1uXYxsr4LHAcV58mLzhmwaeKvJ1pjLiE");
  const usdcUsdPriceFeedAccount = new PublicKey("Dpw1EAVrSB1ibxiDQyTAW6Zip3J4Btk2x4SgApQCeFbX");
  const usdtUsdPriceFeedAccount = new PublicKey("7UVimffxr9ow1uXYxsr4LHAcV58mLzhmwaeKvJ1pjLiE");

  const lamportsToPay = 5000000; // 0.005 SOL in lamports
  const connection = provider.connection;
  const splMint = new PublicKey("7WWz3pdvJiBg9eW1imHCQDXWL19vLA83JWUeV2W2ZgBQ");
  const usdcMint = new PublicKey("7Yz3ecFyeU6heqrNSbikenhDDUX5DkE2eehJR6K1gjBb");

  const userUsdcATA = new PublicKey("AGNFzzPZK4xvXn9bVQL4GbMhGCSAJjVQdCRtfL6mD5tJ")

  
  before(async () => {
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
      await provider.sendAndConfirm(transaction);
      console.log("Created user SPL account:", userSplAccount.toBase58());
    }
  });

  it("Buys SPL tokens with valid SOL", async () => {
    const tx = await program.methods
      .buySplWithSol(new anchor.BN(lamportsToPay))
      .accounts({
        user: wallet.publicKey,
        projectSolAccount: project_spl_authority.publicKey,
        projectSplAta: projectSplAccount,
        projectSplAuthority: project_spl_authority.publicKey,
        mint: splMint,
        priceUpdate: solUsdPriceFeedAccount
      })
      .signers([project_spl_authority])
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
    const lamportsToPay = 1_000_000_000; // 2 SOL in lamports
    try {
      const tx = await program.methods
        .buySplWithSol(new anchor.BN(lamportsToPay))
        .accounts({
          user: wallet.publicKey,
          projectSolAccount: project_spl_authority.publicKey,
          projectSplAta: projectSplAccount,
          projectSplAuthority: project_spl_authority.publicKey,
          mint: splMint,
          priceUpdate: solUsdPriceFeedAccount,
        })
        .signers([project_spl_authority])
        .rpc();

      // If the transaction succeeds unexpectedly, fail the test
      assert.fail("Expected transaction to fail due to insufficient SPL balance, but it succeeded");
    } catch (error) {
      // Parse the Anchor error and assert the error code
      const anchorError = error as anchor.AnchorError;

      // Check for the specific error code related to insufficient SPL balance
      assert.strictEqual(
        anchorError.error.errorCode.code,
        'InsufficientSPLBalance',  // Match the exact error code
        "Expected error due to insufficient SPL balance"
      );
    }
  });

  it("Fails to buy SPL tokens with amount below minimum limit", async () => {
    const lamportsToPayBelowMin = 100000; // Example value below the minimum limit

    try {
      const tx = await program.methods
        .buySplWithSol(new anchor.BN(lamportsToPayBelowMin))
        .accounts({
          user: wallet.publicKey,
          projectSolAccount: project_spl_authority.publicKey,
          projectSplAta: projectSplAccount,
          projectSplAuthority: project_spl_authority.publicKey,
          mint: splMint,
          priceUpdate: solUsdPriceFeedAccount,
        })
        .signers([project_spl_authority])
        .rpc();

      // If the transaction succeeds unexpectedly, fail the test
      assert.fail("Expected transaction to fail due to purchase amount being below the minimum limit, but it succeeded");
    } catch (error) {
      // Parse the Anchor error and assert the error code
      const anchorError = error as anchor.AnchorError;
      // Check for the specific error code related to amount being below the minimum limit
      assert.strictEqual(
        anchorError.error.errorCode.code,
        'PurchaseAmountTooLow',  // Match the exact error code
        "Expected error due to purchase amount being below the minimum limit"
      );
    }
  });

  it("Fails to buy SPL tokens with amount above maximum limit", async () => {
    const lamportsToPayAboveMax = 600000000000;

    try {
      const tx = await program.methods
        .buySplWithSol(new anchor.BN(lamportsToPayAboveMax))
        .accounts({
          user: wallet.publicKey,
          projectSolAccount: project_spl_authority.publicKey,
          projectSplAta: projectSplAccount,
          projectSplAuthority: project_spl_authority.publicKey,
          mint: splMint,
          priceUpdate: solUsdPriceFeedAccount,
        })
        .signers([project_spl_authority])
        .rpc();

      // If the transaction succeeds unexpectedly, fail the test
      assert.fail("Expected transaction to fail due to purchase amount being above the maximum limit, but it succeeded");
    } catch (error) {
      // Parse the Anchor error and assert the error code
      const anchorError = error as anchor.AnchorError;

      // Check for the specific error code related to amount being above the maximum limit
      assert.strictEqual(
        anchorError.error.errorCode.code,
        'PurchaseAmountTooHigh',  // Match the exact error code
        "Expected error due to purchase amount being above the maximum limit"
      );
    }
  });

  it("buy spl toke with valid usdc/usdt amount", async () => {
    try {
      const tokenAmountBelowMin = 500000;
      const tx =await program.methods
        .buySplWithSpl(new anchor.BN(tokenAmountBelowMin))
        .accounts({
          user: wallet.publicKey,
          userTokenAta: userUsdcATA,
          projectTokenAta: projectUsdcAta,
          projectSplAta: projectSplAccount,
          projectSplAuthority: project_spl_authority.publicKey,
          mint: splMint,
          userMint: usdcMint,
          priceUpdate: usdcUsdPriceFeedAccount
        })
        .signers([project_spl_authority])
        .rpc();
        
      console.log("Transaction signature:", tx);
    } catch (error) {
      console.log(error)
    }
  })

})
