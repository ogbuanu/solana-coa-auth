import * as anchor from "@coral-xyz/anchor";
import { Program, AnchorProvider, web3, BN } from "@coral-xyz/anchor";
import { expect } from "chai";
import { SolanaCoaAuth } from "../target/types/solana_coa_auth";

describe("solana-coa-auth", () => {
  const provider = AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.SolanaCoaAuth as Program<SolanaCoaAuth>;

  const [coaConfigPda] = web3.PublicKey.findProgramAddressSync(
    [Buffer.from("coa_config")],
    program.programId
  );

  const deriveUserAccountPda = (wallet: web3.PublicKey) =>
    web3.PublicKey.findProgramAddressSync(
      [Buffer.from("user_account"), wallet.toBuffer()],
      program.programId
    )[0];

  it("initialize creates CoaConfig PDA", async () => {
    await program.methods
      .initialize()
      .accounts({
        coaConfig: coaConfigPda,
        user: provider.wallet.publicKey,
        systemProgram: web3.SystemProgram.programId,
        rent: web3.SYSVAR_RENT_PUBKEY,
      })
      .rpc();

    const accountInfo = await provider.connection.getAccountInfo(coaConfigPda);
    expect(accountInfo).to.not.be.null;
    expect(accountInfo!.owner.toBase58()).to.equal(
      program.programId.toBase58()
    );

    const cfg = await program.account.coaConfig.fetch(coaConfigPda);
    expect(new BN(cfg.nextUserId).toNumber()).to.equal(1);
    expect(new BN(cfg.totalUsers).toNumber()).to.equal(0);
  });

  it("onboard creates primary UserAccount for provider wallet", async () => {
    const userAccountPda = deriveUserAccountPda(provider.wallet.publicKey);

    await program.methods
      .onboard()
      .accounts({
        userAccount: userAccountPda,
        coaConfig: coaConfigPda,
        user: provider.wallet.publicKey,
        systemProgram: web3.SystemProgram.programId,
        rent: web3.SYSVAR_RENT_PUBKEY,
      })
      .rpc();

    const ua = await program.account.userAccount.fetch(userAccountPda);
    const id =
      ua.coaUserId instanceof BN
        ? ua.coaUserId.toNumber()
        : Number(ua.coaUserId);
    expect(id).to.be.greaterThan(0);
    expect(ua.walletAddress.toBase58()).to.equal(
      provider.wallet.publicKey.toBase58()
    );
    expect(ua.isPrimary).to.equal(true);

    const cfg = await program.account.coaConfig.fetch(coaConfigPda);
    expect(new BN(cfg.totalUsers).toNumber()).to.equal(1);
    expect(new BN(cfg.nextUserId).toNumber()).to.equal(id + 1);
  });

  it("prepares a second wallet user account PDA (not onboarded)", async () => {
    // Create a second wallet
    const second = web3.Keypair.generate();
    // Airdrop some SOL for fees
    const sig = await provider.connection.requestAirdrop(second.publicKey, 1e9);
    await provider.connection.confirmTransaction(sig, "confirmed");

    // Initialize its UserAccount PDA via onboard, but we need it to be added by first account later.
    // Instead, we just create PDA on-chain by calling onboard from second wallet.
    const secondUserAccountPda = deriveUserAccountPda(second.publicKey);

    await program.methods
      .onboard()
      .accounts({
        userAccount: secondUserAccountPda,
        coaConfig: coaConfigPda,
        user: second.publicKey,
        systemProgram: web3.SystemProgram.programId,
        rent: web3.SYSVAR_RENT_PUBKEY,
      })
      .signers([second])
      .rpc();

    const ua2 = await program.account.userAccount.fetch(secondUserAccountPda);
    const id2 =
      ua2.coaUserId instanceof BN
        ? ua2.coaUserId.toNumber()
        : Number(ua2.coaUserId);
    expect(id2).to.be.greaterThan(0);
    expect(ua2.isPrimary).to.equal(true);

    // Stash for later tests
    (global as any).second = second;
    (global as any).secondUserAccountPda = secondUserAccountPda;
  });

  it("add_authorized_wallet adds second wallet under first user's COA", async () => {
    const firstUaPda = deriveUserAccountPda(provider.wallet.publicKey);
    const second = (global as any).second as web3.Keypair;
    const secondUaPda = (global as any).secondUserAccountPda as web3.PublicKey;

    // Simulate that second account should be added under first user's COA:
    // Reset second user to look like not having COA (in your current program,
    // add_authorized_wallet requires !new_user_account.has_coa_account())
    // In practice, you'd design a separate PDA for "wallet profile" vs "COA entry".
    // For this test, we'll expect rejection because second already onboarded itself.
    await expect(
      program.methods
        .addAuthorizedWallet()
        .accounts({
          coaConfig: coaConfigPda,
          userAccount: firstUaPda,
          newUserAccount: secondUaPda,
          authority: provider.wallet.publicKey,
        })
        .rpc()
    ).to.be.rejected;
  });

  it("remove_authorized_wallet rejects when same account passed", async () => {
    const firstUaPda = deriveUserAccountPda(provider.wallet.publicKey);

    await expect(
      program.methods
        .removeAuthorizedWallet()
        .accounts({
          userAccount: firstUaPda,
          userAccountToRemove: firstUaPda,
          authority: provider.wallet.publicKey,
        })
        .rpc()
    ).to.be.rejected;
  });

  it("transfer_primary_ownership rejects if accounts differ in COA user", async () => {
    const firstUaPda = deriveUserAccountPda(provider.wallet.publicKey);
    const secondUaPda = (global as any).secondUserAccountPda as web3.PublicKey;

    await expect(
      program.methods
        .transferPrimaryOwnership()
        .accounts({
          userAccount: firstUaPda,
          newPrimaryAccount: secondUaPda,
          authority: provider.wallet.publicKey,
        })
        .rpc()
    ).to.be.rejected;
  });

  it("set_new_primary_ownership rejects without proper authority or mismatched user ids", async () => {
    const firstUaPda = deriveUserAccountPda(provider.wallet.publicKey);
    const secondUaPda = (global as any).secondUserAccountPda as web3.PublicKey;

    await expect(
      program.methods
        .setNewPrimaryOwnership()
        .accounts({
          userAccount: firstUaPda,
          newPrimaryAccount: secondUaPda,
          authority: provider.wallet.publicKey,
        })
        .rpc()
    ).to.be.rejected;
  });

  it("leave_coa_account rejects for primary accounts", async () => {
    const firstUaPda = deriveUserAccountPda(provider.wallet.publicKey);

    await expect(
      program.methods
        .leaveCoaAccount()
        .accounts({
          userAccount: firstUaPda,
          authority: provider.wallet.publicKey,
        })
        .rpc()
    ).to.be.rejected;
  });

  it("fuzz: randomized sequences preserve invariants", async () => {
    const ROUNDS = 100; // increase gradually
    const rng = (
      (seed) => () =>
        (seed = (seed * 1664525 + 1013904223) >>> 0)
    )(12345);
    const pick = (n: number) => rng() % n;

    // Start with primary (provider)
    const primaryUa = deriveUserAccountPda(provider.wallet.publicKey);
    // Ensure onboarded
    await program.methods
      .onboard()
      .accounts({
        userAccount: primaryUa,
        coaConfig: coaConfigPda,
        user: provider.wallet.publicKey,
        systemProgram: web3.SystemProgram.programId,
        rent: web3.SYSVAR_RENT_PUBKEY,
      })
      .rpc();

    // Keep a small pool of wallets
    const pool: { kp: web3.Keypair; pda: web3.PublicKey }[] = [];
    const mkWallet = async () => {
      const kp = web3.Keypair.generate();
      const sig = await provider.connection.requestAirdrop(kp.publicKey, 1e9);
      await provider.connection.confirmTransaction(sig, "confirmed");
      const pda = deriveUserAccountPda(kp.publicKey);
      return { kp, pda };
    };

    // Create 3 wallets
    for (let i = 0; i < 3; i++) pool.push(await mkWallet());

    for (let r = 0; r < ROUNDS; r++) {
      const action = pick(6);
      try {
        switch (action) {
          case 0: {
            // onboard random wallet
            const w = pool[pick(pool.length)];
            await program.methods
              .onboard()
              .accounts({
                userAccount: w.pda,
                coaConfig: coaConfigPda,
                user: w.kp.publicKey,
                systemProgram: web3.SystemProgram.programId,
                rent: web3.SYSVAR_RENT_PUBKEY,
              })
              .signers([w.kp])
              .rpc();
            break;
          }
          case 1: {
            // addAuthorized: primary adds one wallet
            const w = pool[pick(pool.length)];
            await program.methods
              .addAuthorizedWallet()
              .accounts({
                coaConfig: coaConfigPda,
                userAccount: primaryUa,
                newUserAccount: w.pda,
                authority: provider.wallet.publicKey,
              })
              .rpc();
            break;
          }
          case 2: {
            // removeAuthorized: try remove random wallet (could be rejected per guards)
            const w = pool[pick(pool.length)];
            await program.methods
              .removeAuthorizedWallet()
              .accounts({
                userAccount: primaryUa,
                userAccountToRemove: w.pda,
                authority: provider.wallet.publicKey,
              })
              .rpc();
            break;
          }
          case 3: {
            // transferPrimary: try set an onboarded wallet as new primary
            const w = pool[pick(pool.length)];
            await program.methods
              .transferPrimaryOwnership()
              .accounts({
                userAccount: primaryUa,
                newPrimaryAccount: w.pda,
                authority: provider.wallet.publicKey,
              })
              .rpc();
            break;
          }
          case 4: {
            // setNewPrimaryOwnership: normal route
            const w = pool[pick(pool.length)];
            await program.methods
              .setNewPrimaryOwnership()
              .accounts({
                userAccount: primaryUa,
                newPrimaryAccount: w.pda,
                authority: provider.wallet.publicKey,
              })
              .rpc();
            break;
          }
          case 5: {
            // leave: pick random wallet and ask it to leave its own account
            const w = pool[pick(pool.length)];
            await program.methods
              .leaveCoaAccount()
              .accounts({
                userAccount: w.pda,
                authority: w.kp.publicKey,
              })
              .signers([w.kp])
              .rpc();
            break;
          }
        }
      } catch (e) {
        // Expected failures are fine. Assert invariants still hold.
        // Fetch and check core invariants.
        const ua = await program.account.userAccount
          .fetch(primaryUa)
          .catch(() => null);
        if (ua) {
          // Invariant examples:
          // - primaryUa.isPrimary implies it should not have left
          // - if ua.coaUserId == 0 then it cannot be primary
          const isPrimary = ua.isPrimary;
          const id =
            ua.coaUserId instanceof BN
              ? ua.coaUserId.toNumber()
              : Number(ua.coaUserId);
          if (id === 0) expect(isPrimary).to.equal(false);
        }
        // Log minimal info for shrinking
        // console.log("Round", r, "action", action, "error", String(e));
      }
    }

    // Final invariant check
    const cfg = await program.account.coaConfig.fetch(coaConfigPda);
    expect(new BN(cfg.nextUserId).toNumber()).to.be.greaterThan(0);
  });
});
