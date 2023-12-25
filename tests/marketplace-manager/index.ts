import { MarketplaceManager } from "../../target/types/marketplace_manager";
import * as anchor from "@coral-xyz/anchor";
import { assert } from "chai";
import {
  TOKEN_2022_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
  getOrCreateAssociatedTokenAccount,
  createTransferInstruction,
  getAssociatedTokenAddressSync,
  ASSOCIATED_TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import { 
  createFundedAssociatedTokenAccount, 
  createFundedWallet, 
  createMint,
} from "../utils";
import { 
  ConfirmOptions, 
  SYSVAR_RENT_PUBKEY, 
  SystemProgram, 
  Transaction 
} from "@solana/web3.js";
import BN from "bn.js";
import { v4 as uuid, parse } from "uuid";

describe("marketplace_manager", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.MarketplaceManager as anchor.Program<MarketplaceManager>;
  const confirmOptions: ConfirmOptions = { commitment: "confirmed", skipPreflight: true };

  // Keypairs:
  let marketplaceAuth: anchor.web3.Keypair;
  let seller: anchor.web3.Keypair;
  let buyer: anchor.web3.Keypair;
  let exploiter: anchor.web3.Keypair;

  // Mints, vaults and balances:
  let paymentMints: anchor.web3.PublicKey[] = [];
  let marketplaceVaults: [anchor.web3.PublicKey, number][] = [];
  let buyerVaults: [anchor.web3.PublicKey, number][] = [];
  let sellerVaults: [anchor.web3.PublicKey, number][] = [];
  let bountyVaults: [anchor.web3.PublicKey, number][] = [];

  // Program account addresses:
  let marketplacePubkey: anchor.web3.PublicKey;
  let marketplaceBump: number;
  let productPubkey: anchor.web3.PublicKey;

  // Marketplace properties:
  let discountMint: anchor.web3.PublicKey;
  let fee: number;
  let feeReduction: number;
  let rewardMint: anchor.web3.PublicKey;
  let sellerRewardConfig: number;
  let buyerRewardConfig: number;
  let accessMint: anchor.web3.PublicKey;
  let accessMintBump: number;
  const FeePayer = {
    Buyer: { buyer: {} },
    Seller: { seller: {} },
  };

  // Product properties
  let productPrice: BN;
  let id: Uint8Array;

  it("Should create marketplace account", async () => {
    const balance = 1000;
    marketplaceAuth = await createFundedWallet(provider, balance, confirmOptions);
    rewardMint = discountMint = await createMint(provider, confirmOptions);

    [marketplacePubkey, marketplaceBump] = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("marketplace", "utf-8"),
        marketplaceAuth.publicKey.toBuffer()
      ],
      program.programId
    );
    [accessMint, accessMintBump] = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("access_mint", "utf-8"),
        marketplacePubkey.toBuffer(),
      ],
      program.programId
    );

    fee = feeReduction = sellerRewardConfig = buyerRewardConfig = 0;
    const feesConfig = {
      fee,
      feePayer: FeePayer.Seller,
      discountMint,
      feeReduction,
    }
    const rewardsConfig = {
      sellerReward: sellerRewardConfig,
      buyerReward: buyerRewardConfig,
      rewardMint,
    }
    await program.methods
      .initMarketplace(
        accessMintBump,
        feesConfig,
        rewardsConfig,
      )
      .accounts({
        signer: marketplaceAuth.publicKey,
        marketplace: marketplacePubkey,
        accessMint: accessMint,
        rent: SYSVAR_RENT_PUBKEY,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
      })
      .signers([marketplaceAuth])
      .rpc(confirmOptions)
      .catch(console.error);
  
    const marketplaceAccount = await program.account.marketplace.fetch(marketplacePubkey);
    assert.equal(marketplaceAccount.authority.toString(), marketplaceAuth.publicKey.toString());
    assert.equal(marketplaceAccount.bumps.bump, marketplaceBump);
    assert.equal(marketplaceAccount.bumps.accessMintBump, accessMintBump);
    assert.equal(marketplaceAccount.accessMint?.toString(), accessMint.toString());
    assert.equal(marketplaceAccount.feesConfig?.fee, fee);
    assert.equal(marketplaceAccount.feesConfig?.feePayer.toString(), FeePayer.Seller.toString());
    assert.equal(marketplaceAccount.feesConfig?.discountMint?.toString(), discountMint.toString());
    assert.equal(marketplaceAccount.feesConfig?.feeReduction, feeReduction);
    assert.equal(marketplaceAccount.rewardsConfig?.sellerReward, sellerRewardConfig);
    assert.equal(marketplaceAccount.rewardsConfig?.buyerReward, buyerRewardConfig);

    /// marketplace pda is created with "marketpalce" and signer address, lets try to create the same pda
    /// another user cant create the previous marketplace and authority cant be changed
    try {
      await program.methods
        .initMarketplace(
          accessMintBump,
          feesConfig,
          rewardsConfig,
        )
        .accounts({
          signer: marketplaceAuth.publicKey,
          marketplace: marketplacePubkey,
          accessMint: accessMint,
          rent: SYSVAR_RENT_PUBKEY,
          systemProgram: SystemProgram.programId,
          tokenProgram: TOKEN_2022_PROGRAM_ID,
        })
        .signers([marketplaceAuth])
        .rpc(confirmOptions)
    } catch (e) {
      const inUse = e.logs.some(log => log.includes("already in use"));
      assert.isTrue(inUse);   
    }
  });

  it("Should edit marketplace data", async () => {
    await program.methods
      .editMarketplace(null, null)
      .accounts({
        signer: marketplaceAuth.publicKey,
        marketplace: marketplacePubkey,
        accessMint: null,
      })
      .signers([marketplaceAuth])
      .rpc(confirmOptions)
      .catch(console.error);

    const changedMarketplaceAccount = await program.account.marketplace.fetch(marketplacePubkey);
    assert.isDefined(changedMarketplaceAccount);   
    assert.equal(changedMarketplaceAccount.feesConfig, null);
    assert.equal(changedMarketplaceAccount.rewardsConfig, null);
    
    // another wallet tries to change product data
    const balance = 1000;
    exploiter = await createFundedWallet(provider, balance, confirmOptions);
    try {
      await program.methods
        .editMarketplace(null, null)
        .accounts({
          signer: exploiter.publicKey,
          marketplace: marketplacePubkey,
          accessMint: null,
        })
        .signers([exploiter])
        .rpc();
    } catch (e) {
      // marketplace seeds are composed by "marketplace" & signer
      if (e as anchor.AnchorError)
        assert.equal(e.error.errorCode.code, "IncorrectAuthority");
    }
  });

  it("Should create a product account", async () => {
    paymentMints[0] = await createMint(provider, confirmOptions);
    id = parse(uuid());
    const balance = 1000;
    seller = await createFundedWallet(provider, balance, confirmOptions);

    [productPubkey] = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("product", "utf-8"),
        marketplacePubkey.toBuffer(),
        id,
      ],
      program.programId
    );
    productPrice = new BN(10000);

    await program.methods
      .initProduct([...id], productPrice)
      .accounts({
        signer: seller.publicKey,
        marketplace: marketplacePubkey,
        product: productPubkey,
        paymentMint: paymentMints[0],
        accessMint: null,
        accessVault: null,
        rent: SYSVAR_RENT_PUBKEY,
        systemProgram: SystemProgram.programId,
      })
      .signers([seller])
      .rpc(confirmOptions)
      .catch(console.error);

    const productAccount = await program.account.product.fetch(productPubkey);
    assert.isDefined(productAccount);
    assert.equal(productAccount.authority.toString(), seller.publicKey.toString());
    assert.equal(productAccount.id.toString(), id.toString());
    assert.equal(productAccount.sellerConfig.paymentMint.toString(), paymentMints[0].toString());
    assert.equal(Number(productAccount.sellerConfig.productPrice), Number(productPrice));
  });

  it("Should edit product data", async () => {
    const newPaymentMintPubkey = await createMint(provider, confirmOptions);
    const newPrice = new BN(88);

    await program.methods
      .editProduct(newPrice)
      .accounts({
        signer: seller.publicKey,
        product: productPubkey,
        paymentMint: newPaymentMintPubkey,
      })
      .signers([seller])
      .rpc()
      .catch(console.error);

    const changedProductAccount = await program.account.product.fetch(productPubkey);
    assert.isDefined(changedProductAccount);
    assert.equal(changedProductAccount.sellerConfig.paymentMint.toString(), newPaymentMintPubkey.toString());
    assert.equal(Number(changedProductAccount.sellerConfig.productPrice), Number(newPrice));

    // another wallet tries to change product data
    try {
      await program.methods
        .editProduct(productPrice)
        .accounts({
          signer: exploiter.publicKey,
          product: productPubkey,
          paymentMint: newPaymentMintPubkey,
        })
        .signers([exploiter])
        .rpc();
    } catch (e) {
      if (e as anchor.AnchorError)
        assert.equal(e.error.errorCode.code, "IncorrectAuthority");
    }

    // to be able to re-use this account and its data, the account data will be the same that was before this unit test
    await program.methods
      .editProduct(productPrice)
      .accounts({
        signer: seller.publicKey,
        product: productPubkey,
        paymentMint: paymentMints[0],
      })
      .signers([seller])
      .rpc()
      .catch(console.error);

    const productAccount = await program.account.product.fetch(productPubkey);
    assert.isDefined(productAccount);
    assert.equal(productAccount.sellerConfig.paymentMint.toString(), paymentMints[0].toString());
    assert.equal(Number(productAccount.sellerConfig.productPrice), Number(productPrice));
  });

  it("Token-gate marketplace for listing products", async () => {
    await program.methods
      .editMarketplace(null, null)
      .accounts({
        signer: marketplaceAuth.publicKey,
        marketplace: marketplacePubkey,
        accessMint,
      })
      .signers([marketplaceAuth])
      .rpc(confirmOptions)
      .catch(console.error);

    const id = parse(uuid());
    const [productPubkey] = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("product", "utf-8"),
        marketplacePubkey.toBuffer(), 
        id,
      ],
      program.programId
    );
    const accessVault = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      exploiter as anchor.web3.Signer,
      accessMint,
      exploiter.publicKey,
      false,
      "confirmed",
      confirmOptions,
      TOKEN_2022_PROGRAM_ID,
    );
    try {
      await program.methods
        .initProduct([...id], productPrice)
        .accounts({
          signer: exploiter.publicKey,
          marketplace: marketplacePubkey,
          product: productPubkey,
          paymentMint: paymentMints[0],
          accessMint,
          accessVault: accessVault.address,
          rent: SYSVAR_RENT_PUBKEY,
          systemProgram: SystemProgram.programId,
        })
        .signers([exploiter])
        .rpc(confirmOptions);
    } catch (e) {
      // The user does not have any access token
      if (e as anchor.AnchorError)
        assert.equal(e.error.errorCode.code, "NotAllowed");
    }

    await program.methods
      .airdropAccess()
      .accounts({
        signer: marketplaceAuth.publicKey,
        receiver: seller.publicKey,
        marketplace: marketplacePubkey,
        accessMint,
        accessVault: getAssociatedTokenAddressSync(accessMint, seller.publicKey, false, TOKEN_2022_PROGRAM_ID),
        rent: SYSVAR_RENT_PUBKEY,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      })
      .signers([marketplaceAuth])
      .rpc()
      .catch(console.error);

    await program.methods
      .initProduct([...id], productPrice)
      .accounts({
        signer: seller.publicKey,
        marketplace: marketplacePubkey,
        product: productPubkey,
        paymentMint: paymentMints[0],
        accessMint,
        accessVault: getAssociatedTokenAddressSync(accessMint, seller.publicKey, false, TOKEN_2022_PROGRAM_ID),
        rent: SYSVAR_RENT_PUBKEY,
        systemProgram: SystemProgram.programId,
      })
      .signers([seller])
      .rpc(confirmOptions)
      .catch(console.error);
  });

  it("Should register a purchase with spl, no fees, no rewards", async () => {
    const buyerSOLBalance = 1000;
    buyer = await createFundedWallet(provider, buyerSOLBalance, confirmOptions);

    const vaultBalances = 1000000000;
    marketplaceVaults.push([
      await createFundedAssociatedTokenAccount(
        provider,
        paymentMints[0],
        vaultBalances,
        marketplaceAuth
      ),
      vaultBalances
    ]);
    sellerVaults.push([
      await createFundedAssociatedTokenAccount(
        provider,
        paymentMints[0],
        vaultBalances,
        seller
      ),
      vaultBalances
    ]);
    buyerVaults.push([
      await createFundedAssociatedTokenAccount(
        provider,
        paymentMints[0],
        vaultBalances,
        buyer
      ),
      vaultBalances
    ]);

    await program.methods
      .registerBuy(1)
      .accounts({
        signer: buyer.publicKey,
        marketplace: marketplacePubkey,
        product: productPubkey,
        paymentMint: paymentMints[0],
        buyerVault: buyerVaults[0][0],
        sellerVault: sellerVaults[0][0],
        marketplaceVault: null,
        bountyVault: null,
        rent: SYSVAR_RENT_PUBKEY,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([buyer])
      .rpc(confirmOptions)
      .catch(console.error) as string;
    
    const buyerVaultAccount = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      buyer as anchor.web3.Signer,
      paymentMints[0],
      buyer.publicKey,
      false,
      "confirmed",
      confirmOptions,
      TOKEN_PROGRAM_ID,
    );
    buyerVaults[0][1] = buyerVaults[0][1] - Number(productPrice);
    assert.equal(Number(buyerVaultAccount.amount), buyerVaults[0][1]);
    
    const sellerVaultAccount = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      seller as anchor.web3.Signer,
      paymentMints[0],
      seller.publicKey,
      false,
      "confirmed",
      confirmOptions,
      TOKEN_PROGRAM_ID,
    );
    sellerVaults[0][1] = sellerVaults[0][1] + Number(productPrice);
    assert.equal(Number(sellerVaultAccount.amount), sellerVaults[0][1]);
  });

  it("Should register a buy with spl, fees, no rewards", async () => {
    const feesConfig = {
      fee: 100, // 1%
      feePayer: FeePayer.Seller,
      discountMint: null,
      feeReduction: null,
    }
    await program.methods
      .editMarketplace(feesConfig, null)
      .accounts({
        signer: marketplaceAuth.publicKey,
        marketplace: marketplacePubkey,
        accessMint: null,
      })
      .signers([marketplaceAuth])
      .rpc(confirmOptions)
      .catch(console.error);

    const purchaseAmount = 2;
    await program.methods
      .registerBuy(purchaseAmount)
      .accounts({
        signer: buyer.publicKey,
        marketplace: marketplacePubkey,
        product: productPubkey,
        paymentMint: paymentMints[0],
        buyerVault: buyerVaults[0][0],
        sellerVault: sellerVaults[0][0],
        marketplaceVault: marketplaceVaults[0][0],
        bountyVault: null,
        rent: SYSVAR_RENT_PUBKEY,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([buyer])
      .rpc(confirmOptions)
      .catch(console.error) as string;
    
    const marketplaceFee = Math.floor((Number(productPrice) * purchaseAmount * feesConfig.fee) / 10000);
    const marketAuthVaultAccount = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      marketplaceAuth as anchor.web3.Signer,
      paymentMints[0],
      marketplaceAuth.publicKey,
      false,
      "confirmed",
      confirmOptions,
      TOKEN_PROGRAM_ID,
    );
    marketplaceVaults[0][1] = marketplaceVaults[0][1] + marketplaceFee;
    assert.equal(Number(marketAuthVaultAccount.amount), marketplaceVaults[0][1]);

    const buyerVaultAccount = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      buyer as anchor.web3.Signer,
      paymentMints[0],
      buyer.publicKey,
      false,
      "confirmed",
      confirmOptions,
      TOKEN_PROGRAM_ID,
    );
    buyerVaults[0][1] = buyerVaults[0][1] - Number(productPrice) * purchaseAmount;
    assert.equal(Number(buyerVaultAccount.amount), buyerVaults[0][1]);
    
    const sellerVaultAccount = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      seller as anchor.web3.Signer,
      paymentMints[0],
      seller.publicKey,
      false,
      "confirmed",
      confirmOptions,
      TOKEN_PROGRAM_ID,
    );
    sellerVaults[0][1] = sellerVaults[0][1] - marketplaceFee + Number(productPrice) * purchaseAmount;
    assert.equal(Number(sellerVaultAccount.amount), sellerVaults[0][1]);
  });

  it("Discount mint test", async () => {
    const feesConfig = {
      fee: 200, // 2%
      feePayer: FeePayer.Seller,
      discountMint: paymentMints[0],
      feeReduction: 150, // reducing to 0,5% (reduction in absolute terms)
    }
    await program.methods
      .editMarketplace(feesConfig, null)
      .accounts({
        signer: marketplaceAuth.publicKey,
        marketplace: marketplacePubkey,
        accessMint: null,
      })
      .signers([marketplaceAuth])
      .rpc(confirmOptions)
      .catch(console.error);

    const purchaseAmount = 2;
    await program.methods
      .registerBuy(purchaseAmount)
      .accounts({
        signer: buyer.publicKey,
        marketplace: marketplacePubkey,
        product: productPubkey,
        paymentMint: paymentMints[0],
        buyerVault: buyerVaults[0][0],
        sellerVault: sellerVaults[0][0],
        marketplaceVault: marketplaceVaults[0][0],
        bountyVault: null,
        rent: SYSVAR_RENT_PUBKEY,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([buyer])
      .rpc(confirmOptions)
      .catch(console.error) as string;
    
    const marketplaceFee = Math.floor((Number(productPrice) * purchaseAmount * (feesConfig.fee - feesConfig.feeReduction)) / 10000);
    const marketAuthVaultAccount = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      marketplaceAuth as anchor.web3.Signer,
      paymentMints[0],
      marketplaceAuth.publicKey,
      false,
      "confirmed",
      confirmOptions,
      TOKEN_PROGRAM_ID,
    );
    marketplaceVaults[0][1] = marketplaceVaults[0][1] + marketplaceFee;
    assert.equal(Number(marketAuthVaultAccount.amount), marketplaceVaults[0][1]);

    const buyerVaultAccount = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      buyer as anchor.web3.Signer,
      paymentMints[0],
      buyer.publicKey,
      false,
      "confirmed",
      confirmOptions,
      TOKEN_PROGRAM_ID,
    );
    buyerVaults[0][1] = buyerVaults[0][1] - Number(productPrice) * purchaseAmount;
    assert.equal(Number(buyerVaultAccount.amount), buyerVaults[0][1]);
    
    const sellerVaultAccount = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      seller as anchor.web3.Signer,
      paymentMints[0],
      seller.publicKey,
      false,
      "confirmed",
      confirmOptions,
      TOKEN_PROGRAM_ID,
    );
    sellerVaults[0][1] = sellerVaults[0][1] - marketplaceFee + Number(productPrice) * purchaseAmount;
    assert.equal(Number(sellerVaultAccount.amount), sellerVaults[0][1]);
  });

  it("Should register a purchase with spl, no fees, rewards", async () => {
    const rewardsConfig = {
      sellerReward: 100,
      buyerReward: 100,
      rewardMint: paymentMints[0],
    };
    await program.methods
      .editMarketplace(null, rewardsConfig)
      .accounts({
        signer: marketplaceAuth.publicKey,
        marketplace: marketplacePubkey,
        accessMint: null,
      })
      .signers([marketplaceAuth])
      .rpc(confirmOptions)
      .catch(console.error);

    const [bountyVault] = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("bounty_vault", "utf-8"), 
        marketplacePubkey.toBuffer(),
        paymentMints[0].toBuffer()
      ],
      program.programId
    );
    await program.methods
      .initBounty()
      .accounts({
        signer: marketplaceAuth.publicKey,
        marketplace: marketplacePubkey,
        rewardMint: paymentMints[0],
        bountyVault,
        rent: SYSVAR_RENT_PUBKEY,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      })
      .signers([marketplaceAuth])
      .rpc(confirmOptions)
      .catch(console.error);

    bountyVaults.push([bountyVault, 10000]);
    await provider.sendAndConfirm(
      new Transaction()
        .add(
          createTransferInstruction(
            marketplaceVaults[0][0],
            bountyVaults[0][0],
            marketplaceAuth.publicKey,
            bountyVaults[0][1]
          )
        ),
      [marketplaceAuth as anchor.web3.Signer]
    );

    await program.methods
      .registerBuy(1)
      .accounts({
        signer: buyer.publicKey,
        marketplace: marketplacePubkey,
        product: productPubkey,
        paymentMint: paymentMints[0],
        buyerVault: buyerVaults[0][0],
        sellerVault: sellerVaults[0][0],
        marketplaceVault: null,
        bountyVault: bountyVaults[0][0],
        rent: SYSVAR_RENT_PUBKEY,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([buyer])
      .rpc(confirmOptions)
      .catch(console.error);

    const buyerVaultAccount = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      buyer as anchor.web3.Signer,
      paymentMints[0],
      buyer.publicKey,
      false,
      "confirmed",
      confirmOptions,
      TOKEN_PROGRAM_ID,
    );
    buyerVaults[0][1] = buyerVaults[0][1] - Number(productPrice) + Number(productPrice) * rewardsConfig.buyerReward / 10000;
    assert.equal(Number(buyerVaultAccount.amount), buyerVaults[0][1]);
    
    const sellerVaultAccount = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      seller as anchor.web3.Signer,
      paymentMints[0],
      seller.publicKey,
      false,
      "confirmed",
      confirmOptions,
      TOKEN_PROGRAM_ID,
    );
    sellerVaults[0][1] = sellerVaults[0][1] + Number(productPrice) + Number(productPrice) * rewardsConfig.sellerReward / 10000;
    assert.equal(Number(sellerVaultAccount.amount), sellerVaults[0][1]);
  }); 
})