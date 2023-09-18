import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { assert } from "chai";
import {
  TOKEN_2022_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
  getOrCreateAssociatedTokenAccount,
  getAccount,
  createTransferInstruction,
  getAssociatedTokenAddressSync,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  NATIVE_MINT,
} from "@solana/spl-token";
import { 
  createFundedAssociatedTokenAccount, 
  createFundedWallet, 
  createMint, 
  getSplitId 
} from "./utils";
import { 
  ConfirmOptions, 
  SYSVAR_RENT_PUBKEY, 
  SystemProgram, 
  Transaction 
} from "@solana/web3.js";
import { MarketplaceManager } from "../../target/types/marketplace_manager";
import BN from "bn.js";
import { v4 as uuid } from "uuid";

describe("brick", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.MarketplaceManager as Program<MarketplaceManager>;
  const confirmOptions: ConfirmOptions = { commitment: "confirmed" };

  // Keypairs:
  let marketplaceAuth: anchor.web3.Keypair;
  let seller: anchor.web3.Keypair;
  let buyer: anchor.web3.Keypair;
  let exploiter: anchor.web3.Keypair;

  // Mints, vaults and balances:
  let paymentMints: anchor.web3.PublicKey[] = [];
  let productMint: anchor.web3.PublicKey;
  let mintBump: number;
  let marketplaceVaults: [anchor.web3.PublicKey, number][] = [];
  let buyerVaults: [anchor.web3.PublicKey, number][] = [];
  let sellerVaults: [anchor.web3.PublicKey, number][] = [];
  let sellerRewardVaults: [anchor.web3.PublicKey, number][] = [];
  let buyerRewardVaults: [anchor.web3.PublicKey, number][] = [];
  let bountyVaults: [anchor.web3.PublicKey, number][] = [];

  // Program account addresses:
  let marketplacePubkey: anchor.web3.PublicKey;
  let productPubkey: anchor.web3.PublicKey;
  let sellerReward: anchor.web3.PublicKey;
  let buyerReward: anchor.web3.PublicKey;

  // Marketplace properties:
  let discountMint: anchor.web3.PublicKey;
  let fee: number;
  let feeReduction: number;
  let rewardMint: anchor.web3.PublicKey;
  let sellerRewardMarketplace: number;
  let buyerRewardMarketplace: number;
  let useCnfts: boolean;
  let deliverToken: boolean;
  let transferable: boolean;
  let chainCounter: boolean;
  let permissionless: boolean;
  let rewardsEnabled: boolean;
  let accessMint: anchor.web3.PublicKey;
  let accessMintBump: number;
  const FeePayer = {
    Buyer: { buyer: {} },
    Seller: { seller: {} },
  };

  // Product properties
  let productPrice: BN;
  let firstId: Buffer;
  let secondId: Buffer;

  it("Should create marketplace account", async () => {
    rewardMint = discountMint = paymentMints[0] = await createMint(provider, confirmOptions);

    const balance = 1000;
    marketplaceAuth = await createFundedWallet(provider, balance);

    [marketplacePubkey] = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("marketplace", "utf-8"),
        marketplaceAuth.publicKey.toBuffer()
      ],
      program.programId
    );

    const [bountyVault] = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("bounty_vault", "utf-8"),
        marketplacePubkey.toBuffer(),
        rewardMint.toBuffer()
      ],
      program.programId
    );
    bountyVaults.push([bountyVault, 0])

    fee = feeReduction = sellerRewardMarketplace = buyerRewardMarketplace = 0;
    deliverToken = transferable = rewardsEnabled = useCnfts = false;
    chainCounter = permissionless = true;

    [accessMint, accessMintBump] = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("access_mint", "utf-8"),
        marketplacePubkey.toBuffer(),
      ],
      program.programId
    );

    const initMarketplaceParams = {
      fee: fee,
      feeReduction: feeReduction,
      sellerReward: sellerRewardMarketplace,
      buyerReward: buyerRewardMarketplace,
      useCnfts: useCnfts,
      deliverToken: deliverToken,
      transferable: transferable,
      chainCounter: chainCounter,
      permissionless: permissionless,
      rewardsEnabled: rewardsEnabled,
      accessMintBump: accessMintBump,
      feePayer: FeePayer.Seller,
    };
    const initMarketplaceAccounts = {
      systemProgram: SystemProgram.programId,
      tokenProgramV0: TOKEN_PROGRAM_ID,
      tokenProgram2022: TOKEN_2022_PROGRAM_ID,
      rent: SYSVAR_RENT_PUBKEY,
      signer: marketplaceAuth.publicKey,
      marketplace: marketplacePubkey,
      accessMint: accessMint,
      rewardMint: rewardMint,
      discountMint: discountMint,
      bountyVault: bountyVault,
    };
     
    await program.methods
      .initMarketplace(initMarketplaceParams)
      .accounts(initMarketplaceAccounts)
      .signers([marketplaceAuth])
      .rpc(confirmOptions)
      .catch(console.error);

    const marketplaceAccount = await program.account.marketplace.fetch(marketplacePubkey);
    assert.equal(marketplaceAccount.authority.toString(), marketplaceAuth.publicKey.toString());
    assert.equal(marketplaceAccount.tokenConfig.useCnfts, useCnfts);
    assert.equal(marketplaceAccount.tokenConfig.deliverToken, deliverToken);
    assert.equal(marketplaceAccount.tokenConfig.transferable, transferable);
    assert.equal(marketplaceAccount.tokenConfig.chainCounter, chainCounter);
    assert.equal(marketplaceAccount.permissionConfig.accessMint.toString(), accessMint.toString());
    assert.equal(marketplaceAccount.permissionConfig.permissionless, permissionless);
    assert.equal(marketplaceAccount.feesConfig.discountMint.toString(), discountMint.toString());
    assert.equal(marketplaceAccount.feesConfig.fee, fee);
    assert.equal(marketplaceAccount.feesConfig.feeReduction, feeReduction);
    assert.equal(marketplaceAccount.feesConfig.feePayer.toString(), FeePayer.Seller.toString());
    assert.equal(marketplaceAccount.rewardsConfig.rewardMint.toString(), rewardMint.toString());
    assert.equal(marketplaceAccount.rewardsConfig.bountyVaults[0].toString(), bountyVaults[0][0].toString());
    assert.equal(marketplaceAccount.rewardsConfig.sellerReward, sellerRewardMarketplace);
    assert.equal(marketplaceAccount.rewardsConfig.buyerReward, buyerRewardMarketplace);
    assert.equal(marketplaceAccount.rewardsConfig.rewardsEnabled, rewardsEnabled);

    /// marketplace pda is created with "marketpalce" and signer address, lets try to create the same pda
    /// another user cant create the previous marketplace and authority cant be changed
    try {
      await program.methods
        .initMarketplace(initMarketplaceParams)
        .accounts(initMarketplaceAccounts)
        .signers([marketplaceAuth])
        .rpc(confirmOptions)
    } catch (e) {
      const logsWithError = e.logs;
      const isAlreadyInUse = logsWithError.some(log => log.includes("already in use"));
      assert.isTrue(isAlreadyInUse);   
    }
  });    

  it("Should edit marketplace data", async () => {
    const editMarketplaceInfoParams = {
      fee: 100,
      feeReduction: 100,
      sellerReward: 100,
      buyerReward: 100,
      useCnfts: !useCnfts,
      deliverToken: !deliverToken,
      transferable: !transferable,
      chainCounter: !chainCounter,
      permissionless: !permissionless,
      rewardsEnabled: !rewardsEnabled,
      feePayer: FeePayer.Buyer,
    };

    const editMarketplaceInfoAccounts = {
      signer: marketplaceAuth.publicKey,
      marketplace: marketplacePubkey,
      rewardMint: await createMint(provider, confirmOptions),
      discountMint: await createMint(provider, confirmOptions),
    };

    await program.methods
      .editMarketplace(editMarketplaceInfoParams)
      .accounts(editMarketplaceInfoAccounts)
      .signers([marketplaceAuth])
      .rpc(confirmOptions)
      .catch(console.error);

    const changedMarketplaceAccount = await program.account.marketplace.fetch(marketplacePubkey);
    assert.isDefined(changedMarketplaceAccount);
    assert.equal(changedMarketplaceAccount.authority.toString(), marketplaceAuth.publicKey.toString());
    assert.equal(changedMarketplaceAccount.tokenConfig.deliverToken, !deliverToken);
    assert.equal(changedMarketplaceAccount.tokenConfig.useCnfts, !useCnfts);
    assert.equal(changedMarketplaceAccount.tokenConfig.transferable, !transferable);
    assert.equal(changedMarketplaceAccount.tokenConfig.chainCounter, !chainCounter);
    assert.equal(changedMarketplaceAccount.permissionConfig.accessMint.toString(), accessMint.toString());
    assert.equal(changedMarketplaceAccount.permissionConfig.permissionless, !permissionless);
    assert.equal(changedMarketplaceAccount.feesConfig.discountMint.toString(), editMarketplaceInfoAccounts.discountMint.toString());
    assert.equal(changedMarketplaceAccount.feesConfig.feePayer.toString(), FeePayer.Buyer.toString());
    assert.equal(changedMarketplaceAccount.feesConfig.fee, 100);
    assert.equal(changedMarketplaceAccount.feesConfig.feeReduction, 100);
    assert.equal(changedMarketplaceAccount.rewardsConfig.rewardMint.toString(), editMarketplaceInfoAccounts.rewardMint.toString());
    assert.equal(changedMarketplaceAccount.rewardsConfig.bountyVaults[0].toString(), bountyVaults[0][0].toString());
    assert.equal(changedMarketplaceAccount.rewardsConfig.sellerReward, 100);
    assert.equal(changedMarketplaceAccount.rewardsConfig.buyerReward, 100);
    assert.equal(changedMarketplaceAccount.rewardsConfig.rewardsEnabled, !rewardsEnabled);

    // another wallet tries to change product data
    const balance = 1000;
    exploiter = await createFundedWallet(provider, balance);
    const exploiterEditInfoParams = {
      fee: fee,
      feeReduction: feeReduction,
      sellerReward: sellerRewardMarketplace,
      buyerReward: buyerRewardMarketplace,
      useCnfts: useCnfts,
      deliverToken: deliverToken,
      transferable: transferable,
      chainCounter: chainCounter,
      permissionless: permissionless,
      rewardsEnabled: rewardsEnabled,
      feePayer: FeePayer.Seller,
    };
    const exploiterEditInfoAccounts = {
      signer: exploiter.publicKey,
      marketplace: marketplacePubkey,
      rewardMint: rewardMint,
      discountMint: discountMint,
    };

    try {
      await program.methods
        .editMarketplace(exploiterEditInfoParams)
        .accounts(exploiterEditInfoAccounts)
        .signers([exploiter])
        .rpc();
    } catch (e) {
      // marketplace seeds are composed by "marketplace" & signer
      if (e as anchor.AnchorError)
        assert.equal(e.error.errorCode.code, "ConstraintSeeds");
    }
  
    // to be able to re-use this account and its data, the account data will be the same that was before this unit test
    const initMarketplaceParams = {
      fee: fee,
      feeReduction: feeReduction,
      sellerReward: sellerRewardMarketplace,
      buyerReward: buyerRewardMarketplace,
      useCnfts: useCnfts,
      deliverToken: deliverToken,
      transferable: transferable,
      chainCounter: chainCounter,
      permissionless: permissionless,
      rewardsEnabled: rewardsEnabled,
      feePayer: FeePayer.Seller,
    };
    const initMarketplaceAccounts = {
      signer: marketplaceAuth.publicKey,
      marketplace: marketplacePubkey,
      rewardMint: rewardMint,
      discountMint: discountMint,
    };
    await program.methods
      .editMarketplace(initMarketplaceParams)
      .accounts(initMarketplaceAccounts)
      .signers([marketplaceAuth])
      .rpc(confirmOptions)
      .catch(console.error);

    const marketplaceAccount = await program.account.marketplace.fetch(marketplacePubkey);
    assert.isDefined(marketplaceAccount);
    assert.equal(marketplaceAccount.authority.toString(), marketplaceAuth.publicKey.toString());
    assert.equal(marketplaceAccount.tokenConfig.deliverToken, deliverToken);
    assert.equal(marketplaceAccount.tokenConfig.useCnfts, useCnfts);
    assert.equal(marketplaceAccount.tokenConfig.transferable, transferable);
    assert.equal(marketplaceAccount.tokenConfig.chainCounter, chainCounter);
    assert.equal(marketplaceAccount.permissionConfig.accessMint.toString(), accessMint.toString());
    assert.equal(marketplaceAccount.permissionConfig.permissionless, permissionless);
    assert.equal(marketplaceAccount.feesConfig.discountMint.toString(), discountMint.toString());
    assert.equal(marketplaceAccount.feesConfig.feePayer.toString(), FeePayer.Seller.toString());
    assert.equal(marketplaceAccount.feesConfig.fee, fee);
    assert.equal(marketplaceAccount.feesConfig.feeReduction, feeReduction);
    assert.equal(marketplaceAccount.rewardsConfig.rewardMint.toString(), rewardMint.toString());
    assert.equal(marketplaceAccount.rewardsConfig.bountyVaults[0].toString(), bountyVaults[0][0].toString());
    assert.equal(marketplaceAccount.rewardsConfig.sellerReward, sellerRewardMarketplace);
    assert.equal(marketplaceAccount.rewardsConfig.buyerReward, buyerRewardMarketplace);
    assert.equal(marketplaceAccount.rewardsConfig.rewardsEnabled, rewardsEnabled);
  });

  it("Should create a product account", async () => {
    [firstId, secondId] = getSplitId(uuid());
    const balance = 1000;
    seller = await createFundedWallet(provider, balance);

    [productPubkey] = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("product", "utf-8"), 
        firstId, 
        secondId,
        marketplacePubkey.toBuffer()
      ],
      program.programId
    );
    [productMint, mintBump] = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("product_mint", "utf-8"), 
        productPubkey.toBuffer()
      ],
      program.programId
    );
    productPrice = new BN(10000);

    const initProductParams = {
      firstId: [...firstId],
      secondId: [...secondId],
      productPrice: productPrice,
      productMintBump: mintBump,
    };
    const initProductAccounts = {
      systemProgram: SystemProgram.programId,
      tokenProgram2022: TOKEN_2022_PROGRAM_ID,
      rent: SYSVAR_RENT_PUBKEY,
      signer: seller.publicKey,
      marketplace: marketplacePubkey,
      product: productPubkey,
      productMint: productMint,
      paymentMint: paymentMints[0],
      accessMint: null,
      accessVault: null,
    };

    await program.methods
      .initProduct(initProductParams)
      .accounts(initProductAccounts)
      .signers([seller])
      .rpc(confirmOptions)
      .catch(console.error);

    const productAccount = await program.account.product.fetch(productPubkey);
    assert.isDefined(productAccount);
    assert.equal(productAccount.authority.toString(), seller.publicKey.toString());
    assert.equal(productAccount.firstId.toString(), [...firstId].toString());
    assert.equal(productAccount.secondId.toString(), [...secondId].toString());
    assert.equal(productAccount.marketplace.toString(), marketplacePubkey.toString());
    assert.equal(productAccount.productMint.toString(), productMint.toString());
    assert.equal(productAccount.sellerConfig.paymentMint.toString(), paymentMints[0].toString());
    assert.equal(Number(productAccount.sellerConfig.productPrice), Number(productPrice));
  });

  it("Should edit product data", async () => {
    const newPaymentMintPubkey = await createMint(provider, confirmOptions);
    const newPrice = new BN(88);

    const editProductInfoAccounts = {
      signer: seller.publicKey,
      product: productPubkey,
      paymentMint: newPaymentMintPubkey,
    };
    await program.methods
      .editProduct(newPrice)
      .accounts(editProductInfoAccounts)
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
          paymentMint: newPaymentMintPubkey
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

  it("Should register a buy with spl, no fees, no token, two times calls register_buy", async () => {
    const buyerSOLBalance = 1000;
    buyer = await createFundedWallet(provider, buyerSOLBalance);

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
  
    const [paymentPubkey] = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("payment", "utf-8"), 
        buyer.publicKey.toBuffer(), 
        productPubkey.toBuffer(),
      ],
      program.programId
    );

    const registerBuyAccounts = {
      systemProgram: SystemProgram.programId,
      tokenProgramV0: TOKEN_PROGRAM_ID,
      rent: SYSVAR_RENT_PUBKEY,
      signer: buyer.publicKey,
      seller: null,
      marketplaceAuth: null,
      marketplace: marketplacePubkey,
      product: productPubkey,
      payment: paymentPubkey,
      paymentMint: paymentMints[0],
      buyerTransferVault: buyerVaults[0][0],
      sellerTransferVault: sellerVaults[0][0],
      marketplaceTransferVault: marketplaceVaults[0][0],
      bountyVault: null,
      sellerReward: null,
      sellerRewardVault: null,
      buyerReward: null,
      buyerRewardVault: null,
    };

    const sig = await program.methods
      .registerBuyCounter(1)
      .accounts(registerBuyAccounts)
      .signers([buyer])
      .postInstructions(
        [
          await program.methods
            .registerBuyCounter(1)
            .accounts(registerBuyAccounts)
            .instruction()
        ]
      )
      .rpc(confirmOptions)
      .catch(console.error) as string;

    const tx = provider.connection.getParsedTransaction(sig, { commitment: "confirmed"});
    console.log('Purchase a product, spl as a payment method compute units ' + (await tx).meta.computeUnitsConsumed);

    const paymentAccount = await program.account.payment.fetch(paymentPubkey);
    assert.isDefined(paymentAccount);
    assert.equal(paymentAccount.units, 2);

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
    buyerVaults[0][1] = vaultBalances - 2 * Number(productPrice);
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
    sellerVaults[0][1] = vaultBalances + 2 * Number(productPrice);
    assert.equal(Number(sellerVaultAccount.amount), sellerVaults[0][1]);
  });

  it("Should register a buy with spl and fees (seller fee payer)", async () => {
    [fee, feeReduction, sellerRewardMarketplace, buyerRewardMarketplace] = [100, 0, 0, 0];
    const editMarketplaceInfoParams = {
      fee: fee,
      feeReduction: feeReduction,
      sellerReward: sellerRewardMarketplace,
      buyerReward: buyerRewardMarketplace,
      deliverToken: deliverToken,
      useCnfts: useCnfts,
      transferable: transferable,
      chainCounter: chainCounter,
      permissionless: permissionless,
      rewardsEnabled: rewardsEnabled,
      feePayer: FeePayer.Seller,
    };
    const editMarketplaceInfoAccounts = {
      signer: marketplaceAuth.publicKey,
      marketplace: marketplacePubkey,
      rewardMint: rewardMint,
      discountMint: discountMint,
    };

    await program.methods
      .editMarketplace(editMarketplaceInfoParams)
      .accounts(editMarketplaceInfoAccounts)
      .signers([marketplaceAuth])
      .rpc()
      .catch(console.error);

    const [paymentPubkey, bump] = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("payment", "utf-8"), 
        buyer.publicKey.toBuffer(), 
        productPubkey.toBuffer(),
      ],
      program.programId
    );

    const registerBuyAccounts = {
      systemProgram: SystemProgram.programId,
      tokenProgramV0: TOKEN_PROGRAM_ID,
      rent: SYSVAR_RENT_PUBKEY,
      signer: buyer.publicKey,
      seller: null,
      marketplaceAuth: null,
      marketplace: marketplacePubkey,
      product: productPubkey,
      payment: paymentPubkey,
      paymentMint: paymentMints[0],
      buyerTransferVault: buyerVaults[0][0],
      sellerTransferVault: sellerVaults[0][0],
      marketplaceTransferVault: marketplaceVaults[0][0],
      bountyVault: null,
      sellerReward: null,
      sellerRewardVault: null,
      buyerReward: null,
      buyerRewardVault: null,
    };

    await program.methods
      .registerBuy(1)
      .accounts(registerBuyAccounts)
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
    const marketplaceFee = Math.floor((Number(productPrice) * (fee - feeReduction)) / 10000);
    sellerVaults[0][1] = sellerVaults[0][1] + Number(productPrice) - marketplaceFee;
    assert.equal(Number(sellerVaultAccount.amount), sellerVaults[0][1]);

    const marketAuthTransferVaultAccount = await getOrCreateAssociatedTokenAccount(
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
    assert.equal(Number(marketAuthTransferVaultAccount.amount), marketplaceVaults[0][1]);
  });

  it("Should register a buy (with fees and native mint)", async () => {
    const newPaymentMintPubkey = NATIVE_MINT;
    const newPrice = new BN(88);

    const editProductInfoAccounts = {
      signer: seller.publicKey,
      product: productPubkey,
      paymentMint: newPaymentMintPubkey,
    };
    await program.methods
      .editProduct(newPrice)
      .accounts(editProductInfoAccounts)
      .signers([seller])
      .rpc()
      .catch(console.error);

    const [paymentPubkey, bump] = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("payment", "utf-8"), 
        buyer.publicKey.toBuffer(), 
        productPubkey.toBuffer(),
      ],
      program.programId
    );

    const marketAuthBalance = await provider.connection.getBalance(marketplaceAuth.publicKey, confirmOptions);
    const sellerBalance = await provider.connection.getBalance(seller.publicKey, confirmOptions);
    const buyerBalance = await provider.connection.getBalance(buyer.publicKey, confirmOptions);

    const registerBuyAccounts = {
      systemProgram: SystemProgram.programId,
      tokenProgramV0: TOKEN_PROGRAM_ID,
      rent: SYSVAR_RENT_PUBKEY,
      signer: buyer.publicKey,
      seller: seller.publicKey,
      marketplaceAuth: marketplaceAuth.publicKey,
      marketplace: marketplacePubkey,
      product: productPubkey,
      payment: paymentPubkey,
      paymentMint: newPaymentMintPubkey,
      buyerTokenVault: null,
      buyerTransferVault: null,
      sellerTransferVault: null,
      marketplaceTransferVault: null,
      bountyVault: null,
      sellerReward: null,
      sellerRewardVault: null,
      buyerReward: null,
      buyerRewardVault: null,
    };

    await program.methods
      .registerBuy(1)
      .accounts(registerBuyAccounts)
      .signers([buyer])
      .rpc(confirmOptions)
      .catch(console.error) as string;

    // Set the previous product configuration
    const initialEditProductInfoAccounts = {
      signer: seller.publicKey,
      product: productPubkey,
      paymentMint: paymentMints[0],
    };
    await program.methods
      .editProduct(productPrice)
      .accounts(initialEditProductInfoAccounts)
      .signers([seller])
      .rpc()
      .catch(console.error);

    const postMarketAuthBalance = await provider.connection.getBalance(marketplaceAuth.publicKey, confirmOptions);
    const postSellerBalance = await provider.connection.getBalance(seller.publicKey, confirmOptions);
    const postBuyerBalance = await provider.connection.getBalance(buyer.publicKey, confirmOptions);
    const marketplaceFee = Math.floor((Number(newPrice) * fee) / 10000);

    assert.equal(postMarketAuthBalance, marketAuthBalance + marketplaceFee);
    assert.equal(postSellerBalance, sellerBalance + Number(newPrice) - marketplaceFee);
    assert.equal(postBuyerBalance, buyerBalance - Number(newPrice));
  });

  it("Should register a buy (with fees and specific mint makes fee reduction)", async () => {
    [fee, feeReduction, sellerRewardMarketplace, buyerRewardMarketplace] = [100, 20, 0, 0];
    discountMint = paymentMints[0];
    const editMarketplaceInfoParams = {
      fee: fee,
      feeReduction: feeReduction,
      sellerReward: sellerRewardMarketplace,
      buyerReward: buyerRewardMarketplace,
      useCnfts: useCnfts,
      deliverToken: deliverToken,
      transferable: transferable,
      chainCounter: chainCounter,
      permissionless: permissionless,
      rewardsEnabled: rewardsEnabled,
      feePayer: FeePayer.Seller,
    };
    const editMarketplaceInfoAccounts = {
      signer: marketplaceAuth.publicKey,
      marketplace: marketplacePubkey,
      rewardMint: rewardMint,
      discountMint: discountMint,
    };

    await program.methods
      .editMarketplace(editMarketplaceInfoParams)
      .accounts(editMarketplaceInfoAccounts)
      .signers([marketplaceAuth])
      .rpc()
      .catch(console.error);

    const [paymentPubkey, bump] = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("payment", "utf-8"), 
        buyer.publicKey.toBuffer(), 
        productPubkey.toBuffer(),
      ],
      program.programId
    );

    const registerBuyAccounts = {
      systemProgram: SystemProgram.programId,
      tokenProgramV0: TOKEN_PROGRAM_ID,
      rent: SYSVAR_RENT_PUBKEY,
      signer: buyer.publicKey,
      seller: null,
      marketplaceAuth: null,
      marketplace: marketplacePubkey,
      product: productPubkey,
      payment: paymentPubkey,
      paymentMint: paymentMints[0],
      buyerTransferVault: buyerVaults[0][0],
      sellerTransferVault: sellerVaults[0][0],
      marketplaceTransferVault: marketplaceVaults[0][0],
      bountyVault: null,
      sellerReward: null,
      sellerRewardVault: null,
      buyerReward: null,
      buyerRewardVault: null,
    };

    await program.methods
      .registerBuy(1)
      .accounts(registerBuyAccounts)
      .signers([buyer])
      .rpc(confirmOptions)
      .catch(console.error);

    const marketAuthTransferVaultAccount = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      marketplaceAuth as anchor.web3.Signer,
      paymentMints[0],
      marketplaceAuth.publicKey,
      false,
      "confirmed",
      confirmOptions,
      TOKEN_PROGRAM_ID,
    );
    const marketplaceFee = Math.floor((Number(productPrice) * (fee - feeReduction)) / 10000);
    marketplaceVaults[0][1] = marketplaceVaults[0][1] + marketplaceFee;
    assert.equal(Number(marketAuthTransferVaultAccount.amount), marketplaceVaults[0][1]);

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
    sellerVaults[0][1] = sellerVaults[0][1] + Number(productPrice) - marketplaceFee;
    assert.equal(Number(sellerVaultAccount.amount), sellerVaults[0][1]);
  });

  it("Should register a buy during promo time, users can withdraw bonus when that promo is finished (not when still active)", async () => {
    // fill the token account controlled by the program to send the rewards
    bountyVaults[0][1] = 1000000;
    marketplaceVaults[0][1] = marketplaceVaults[0][1] - 1000000;
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
    [fee, feeReduction, sellerRewardMarketplace, buyerRewardMarketplace] = [100, 20, 20, 20];
    rewardsEnabled = true;
    const editMarketplaceInfoParams = {
      fee: fee,
      feeReduction: feeReduction,
      sellerReward: sellerRewardMarketplace,
      buyerReward: buyerRewardMarketplace,
      useCnfts: useCnfts,
      deliverToken: deliverToken,
      transferable: transferable,
      chainCounter: chainCounter,
      permissionless: permissionless,
      rewardsEnabled: rewardsEnabled,
      accessMintBump: accessMintBump,
      feePayer: FeePayer.Seller,
    };
    const editMarketplaceInfoAccounts = {
      signer: marketplaceAuth.publicKey,
      marketplace: marketplacePubkey,
      rewardMint: rewardMint,
      discountMint: discountMint,
    };

    await program.methods
      .editMarketplace(editMarketplaceInfoParams)
      .accounts(editMarketplaceInfoAccounts)
      .signers([marketplaceAuth])
      .rpc()
      .catch(console.error);

    [sellerReward] = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("reward", "utf-8"), 
        seller.publicKey.toBuffer(),
        marketplacePubkey.toBuffer()
      ],
      program.programId
    );
    const [sellerRewardVault] = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("reward_vault", "utf-8"), 
        seller.publicKey.toBuffer(),
        marketplacePubkey.toBuffer(),
        rewardMint.toBuffer(),
      ],
      program.programId
    );
    sellerRewardVaults.push([sellerRewardVault, 0]);

    const initSellerRewardAccounts = {
      systemProgram: SystemProgram.programId,
      tokenProgramV0: TOKEN_PROGRAM_ID,
      rent: SYSVAR_RENT_PUBKEY,
      signer: seller.publicKey,
      marketplace: marketplacePubkey,
      reward: sellerReward,
      rewardMint: paymentMints[0],
      rewardVault: sellerRewardVault,
    };
    
    await program.methods
      .initReward()
      .accounts(initSellerRewardAccounts)
      .signers([seller])
      .rpc()
      .catch(console.error);

    const sellerRewardAccount = await program.account.reward.fetch(sellerReward);
    assert.isDefined(sellerRewardAccount);
    assert.equal(sellerRewardAccount.authority.toString(), seller.publicKey.toString());
    assert.equal(sellerRewardAccount.marketplace.toString(), marketplacePubkey.toString());
    assert.equal(sellerRewardAccount.rewardVaults[0].toString(), sellerRewardVault.toString());

    [buyerReward] = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("reward", "utf-8"), 
        buyer.publicKey.toBuffer(),
        marketplacePubkey.toBuffer()
      ],
      program.programId
    );
    const [buyerRewardVault] = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("reward_vault", "utf-8"), 
        buyer.publicKey.toBuffer(),
        marketplacePubkey.toBuffer(),
        rewardMint.toBuffer(),
      ],
      program.programId
    );
    buyerRewardVaults.push([buyerRewardVault, 0]);

    const initBuyerRewardAccounts = {
      systemProgram: SystemProgram.programId,
      tokenProgramV0: TOKEN_PROGRAM_ID,
      rent: SYSVAR_RENT_PUBKEY,
      signer: buyer.publicKey,
      marketplace: marketplacePubkey,
      reward: buyerReward,
      rewardMint: paymentMints[0],
      rewardVault: buyerRewardVault,
    };
    await program.methods
      .initReward()
      .accounts(initBuyerRewardAccounts)
      .signers([buyer])
      .rpc()
      .catch(console.error);

    const buyerRewardAccount = await program.account.reward.fetch(buyerReward);
    assert.isDefined(buyerRewardAccount);
    assert.equal(buyerRewardAccount.authority.toString(), buyer.publicKey.toString());
    assert.equal(buyerRewardAccount.marketplace.toString(), marketplacePubkey.toString());
    assert.equal(buyerRewardAccount.rewardVaults[0].toString(), buyerRewardVault.toString());

    const [paymentPubkey, bump] = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("payment", "utf-8"), 
        buyer.publicKey.toBuffer(), 
        productPubkey.toBuffer(),
      ],
      program.programId
    );
    const registerRewardBuyAccounts = {
      systemProgram: SystemProgram.programId,
      tokenProgramV0: TOKEN_PROGRAM_ID,
      rent: SYSVAR_RENT_PUBKEY,
      signer: buyer.publicKey,
      seller: null,
      marketplaceAuth: null,
      marketplace: marketplacePubkey,
      product: productPubkey,
      payment: paymentPubkey,
      paymentMint: paymentMints[0],
      buyerTransferVault: buyerVaults[0][0],
      sellerTransferVault: sellerVaults[0][0],
      marketplaceTransferVault: marketplaceVaults[0][0],
      bountyVault: bountyVaults[0][0],
      sellerReward: sellerReward,
      sellerRewardVault: sellerRewardVault,
      buyerReward: buyerReward,
      buyerRewardVault: buyerRewardVault,
    };

    await program.methods
      .registerBuy(1)
      .accounts(registerRewardBuyAccounts)
      .signers([buyer])
      .rpc(confirmOptions)
      .catch(console.error);

    const oldsellerPromo = 20;
    const expectedSellerReward = Math.floor(Number(productPrice) * oldsellerPromo / 10000);
    const sellerRewardFunds = await getAccount(provider.connection, sellerRewardVault);
    assert.equal(Number(sellerRewardFunds.amount), expectedSellerReward);

    const oldBuyerPromo = 20;
    const expectedBuyerReward = Math.floor(Number(productPrice) * oldBuyerPromo / 10000);
    const buyerRewardFunds = await getAccount(provider.connection, sellerRewardVault);
    assert.equal(Number(buyerRewardFunds.amount), expectedBuyerReward);

    try {
      await program.methods
        .withdrawReward()
        .accounts({
          tokenProgramV0: TOKEN_PROGRAM_ID,
          signer: buyer.publicKey,
          marketplace: marketplacePubkey,
          reward: buyerReward,
          rewardMint: rewardMint,
          receiverVault: buyerVaults[0][0],
          rewardVault: buyerRewardVaults[0][0],
        })
      .signers([buyer])
      .rpc(confirmOptions);
    } catch (e) {
      if (e as anchor.AnchorError)
        assert.equal(e.error.errorCode.code, "OpenPromotion");
    }

    // promo is finished with rewardsEnable = false
    rewardsEnabled = false;
    const changeMarketplaceInfoParams = {
      fee: fee,
      feeReduction: feeReduction,
      sellerReward: sellerRewardMarketplace,
      buyerReward: buyerRewardMarketplace,
      useCnfts: useCnfts,
      deliverToken: deliverToken,
      transferable: transferable,
      chainCounter: chainCounter,
      permissionless: permissionless,
      rewardsEnabled: rewardsEnabled,
      accessMintBump: accessMintBump,
      feePayer: FeePayer.Seller,
    };
    const changeMarketplaceInfoAccounts = {
      signer: marketplaceAuth.publicKey,
      marketplace: marketplacePubkey,
      rewardMint: rewardMint,
      discountMint: discountMint,
    };

    await program.methods
      .editMarketplace(changeMarketplaceInfoParams)
      .accounts(changeMarketplaceInfoAccounts)
      .signers([marketplaceAuth])
      .rpc()
      .catch(console.error);

    // only the reward auth can withdraw
    try {
      await program.methods
        .withdrawReward()
        .accounts({
          tokenProgramV0: TOKEN_PROGRAM_ID,
          signer: seller.publicKey,
          marketplace: marketplacePubkey,
          reward: buyerReward,
          rewardMint: rewardMint,
          receiverVault: sellerVaults[0][0],
          rewardVault: buyerRewardVaults[0][0],
        })
        .signers([seller])
        .rpc();
    } catch (e) {
      if (e as anchor.AnchorError)
        assert.equal(e.error.errorCode.code, "ConstraintSeeds");
    }

    await program.methods
      .withdrawReward()
      .accounts({
        tokenProgramV0: TOKEN_PROGRAM_ID,
        signer: buyer.publicKey,
        marketplace: marketplacePubkey,
        reward: buyerReward,
        rewardMint: rewardMint,
        receiverVault: buyerVaults[0][0],
        rewardVault: buyerRewardVaults[0][0],
      })
      .signers([buyer])
      .rpc(confirmOptions)
      .catch(console.error);

    await program.methods
      .withdrawReward()
      .accounts({
        tokenProgramV0: TOKEN_PROGRAM_ID,
        signer: seller.publicKey,
        marketplace: marketplacePubkey,
        reward: sellerReward,
        rewardMint: rewardMint,
        receiverVault: sellerVaults[0][0],
        rewardVault: sellerRewardVaults[0][0],
      })
      .signers([seller])
      .rpc(confirmOptions)
      .catch(console.error);

    const marketplaceTokenVaultAccount = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      marketplaceAuth as anchor.web3.Signer,
      paymentMints[0],
      marketplaceAuth.publicKey,
      false,
      "confirmed",
      confirmOptions,
      TOKEN_PROGRAM_ID,
    );
    const buyerTokenTransferVaultAccount = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      buyer as anchor.web3.Signer,
      paymentMints[0],
      buyer.publicKey,
      false,
      "confirmed",
      confirmOptions,
      TOKEN_PROGRAM_ID,
    );
    const sellerTokenVaultAccount = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      seller as anchor.web3.Signer,
      paymentMints[0],
      seller.publicKey,
      false,
      "confirmed",
      confirmOptions,
      TOKEN_PROGRAM_ID,
    ); 

    const governanceFee = Math.floor(Number(productPrice) * (fee - feeReduction) / 10000);
    marketplaceVaults[0][1] = marketplaceVaults[0][1] + governanceFee;
    buyerVaults[0][1] = buyerVaults[0][1] - Number(productPrice) + expectedBuyerReward;
    const oldSellerPromo = 20; // Change to the actual value
    const expectedSellerBonus = Math.floor(Number(productPrice) * oldSellerPromo / 10000);
    sellerVaults[0][1] = sellerVaults[0][1] + Number(productPrice) - governanceFee + expectedSellerBonus;

    assert.equal(Number(marketplaceTokenVaultAccount.amount), marketplaceVaults[0][1]);
    assert.equal(Number(buyerTokenTransferVaultAccount.amount), buyerVaults[0][1]);    
    assert.equal(Number(sellerTokenVaultAccount.amount), sellerVaults[0][1]);
  });

  it("Should register a buy with SOL as payment, with rewards active (should not give rewards and not errors)", async () => {
    [fee, feeReduction, sellerRewardMarketplace, buyerRewardMarketplace] = [100, 20, 20, 20];
    rewardsEnabled = true;
    [rewardMint] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("null", "utf-8")],
      program.programId
    );
    const editMarketplaceInfoParams = {
      fee: fee,
      feeReduction: feeReduction,
      sellerReward: sellerRewardMarketplace,
      buyerReward: buyerRewardMarketplace,
      useCnfts: useCnfts,
      deliverToken: deliverToken,
      transferable: transferable,
      chainCounter: chainCounter,
      permissionless: permissionless,
      rewardsEnabled: rewardsEnabled,
      accessMintBump: accessMintBump,
      feePayer: FeePayer.Seller,
    };
    const editMarketplaceInfoAccounts = {
      signer: marketplaceAuth.publicKey,
      marketplace: marketplacePubkey,
      rewardMint: rewardMint,
      discountMint: discountMint,
    };

    await program.methods
      .editMarketplace(editMarketplaceInfoParams)
      .accounts(editMarketplaceInfoAccounts)
      .signers([marketplaceAuth])
      .rpc()
      .catch(console.error);

    const productPrice = new BN(1000);
    await program.methods
      .editProduct(productPrice)
      .accounts({
        signer: seller.publicKey,
        product: productPubkey,
        paymentMint: NATIVE_MINT,
      })
      .signers([seller])
      .rpc()
      .catch(console.error);

    const [paymentPubkey] = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("payment", "utf-8"), 
        buyer.publicKey.toBuffer(), 
        productPubkey.toBuffer(),
      ],
      program.programId
    );
    const registerRewardBuyAccounts = {
      systemProgram: SystemProgram.programId,
      tokenProgramV0: TOKEN_PROGRAM_ID,
      rent: SYSVAR_RENT_PUBKEY,
      signer: buyer.publicKey,
      seller: seller.publicKey,
      marketplaceAuth: marketplaceAuth.publicKey,
      marketplace: marketplacePubkey,
      product: productPubkey,
      payment: paymentPubkey,
      paymentMint: NATIVE_MINT,
      buyerTransferVault: null,
      sellerTransferVault: null,
      marketplaceTransferVault: null,
      bountyVault: null,
      sellerReward: sellerReward,
      sellerRewardVault: null,
      buyerReward: buyerReward,
      buyerRewardVault: null,
    };

    const preSellerBalance = await provider.connection.getBalance(seller.publicKey, confirmOptions);
    const preBuyerBalance = await provider.connection.getBalance(buyer.publicKey, confirmOptions);

    await program.methods
      .registerBuy(1)
      .accounts(registerRewardBuyAccounts)
      .signers([buyer])
      .rpc(confirmOptions)
      .catch(console.error);

    const postSellerBalance = await provider.connection.getBalance(seller.publicKey, confirmOptions);
    const postBuyerBalance = await provider.connection.getBalance(buyer.publicKey, confirmOptions);
    const marketplaceFee = Math.floor((Number(productPrice) * (fee)) / 10000);

    assert.equal(preSellerBalance + Number(productPrice) - marketplaceFee, postSellerBalance);
    assert.equal(preBuyerBalance - 1000, postBuyerBalance);
  });

  it("Should allow receiving rewards with different mints (always reward == payment), also tests reward enforcement (only one mint)", async () => {
    const rewardMint = await createMint(provider, confirmOptions);
    const vaultBalances = 50000;
    marketplaceVaults.push([
      await createFundedAssociatedTokenAccount(
        provider,
        rewardMint,
        vaultBalances,
        marketplaceAuth
      ),
      vaultBalances
    ]);
    sellerVaults.push([
      await createFundedAssociatedTokenAccount(
        provider,
        rewardMint,
        vaultBalances,
        seller
      ),
      vaultBalances
    ]);
    buyerVaults.push([
      await createFundedAssociatedTokenAccount(
        provider,
        rewardMint,
        vaultBalances,
        buyer
      ),
      vaultBalances
    ]);

    [fee, feeReduction, sellerRewardMarketplace, buyerRewardMarketplace] = [100, 20, 20, 20];
    rewardsEnabled = true;
    const editMarketplaceInfoParams = {
      fee: fee,
      feeReduction: feeReduction,
      sellerReward: sellerRewardMarketplace,
      buyerReward: buyerRewardMarketplace,
      useCnfts: useCnfts,
      deliverToken: deliverToken,
      transferable: transferable,
      chainCounter: chainCounter,
      permissionless: permissionless,
      rewardsEnabled: rewardsEnabled,
      accessMintBump: accessMintBump,
      feePayer: FeePayer.Seller,
    };
    const editMarketplaceInfoAccounts = {
      signer: marketplaceAuth.publicKey,
      marketplace: marketplacePubkey,
      rewardMint: rewardMint,
      discountMint: discountMint,
    };
    await program.methods
      .editMarketplace(editMarketplaceInfoParams)
      .accounts(editMarketplaceInfoAccounts)
      .signers([marketplaceAuth])
      .rpc()
      .catch(console.error);

    const productPrice = new BN(5000);
    await program.methods
      .editProduct(productPrice)
      .accounts({
        signer: seller.publicKey,
        product: productPubkey,
        paymentMint:rewardMint,
      })
      .signers([seller])
      .rpc()
      .catch(console.error);

    await sleep(2000);

    const [bountyVault] = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("bounty_vault", "utf-8"),
        marketplacePubkey.toBuffer(),
        rewardMint.toBuffer()
      ],
      program.programId
    );

    await program.methods
      .initBounty()
      .accounts({
        systemProgram: SystemProgram.programId,
        tokenProgramV0: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        rent: SYSVAR_RENT_PUBKEY,
        signer: marketplaceAuth.publicKey,
        marketplace: marketplacePubkey,
        rewardMint: rewardMint,
        bountyVault: bountyVault,
      })
      .signers([marketplaceAuth])
      .rpc(confirmOptions)
      .catch(console.error);

    await provider.sendAndConfirm(
      new Transaction()
        .add(
          createTransferInstruction(
            marketplaceVaults[1][0],
            bountyVault,
            marketplaceAuth.publicKey,
            5000
          )
        ),
      [marketplaceAuth as anchor.web3.Signer]
    );
    bountyVaults.push([bountyVault, 5000]);
    const [sellerRewardVault] = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("reward_vault", "utf-8"), 
        seller.publicKey.toBuffer(),
        marketplacePubkey.toBuffer(),
        rewardMint.toBuffer(),
      ],
      program.programId
    );
    sellerRewardVaults.push([sellerRewardVault, 0]);

    await program.methods
      .initRewardVault()
      .accounts({
        systemProgram: SystemProgram.programId,
        tokenProgramV0: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        rent: SYSVAR_RENT_PUBKEY,
        signer: seller.publicKey,
        marketplace: marketplacePubkey,
        reward: sellerReward,
        rewardMint: rewardMint,
        rewardVault: sellerRewardVault,
      })
      .signers([seller])
      .rpc()
      .catch(console.error);

    const [buyerRewardVault] = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("reward_vault", "utf-8"), 
        buyer.publicKey.toBuffer(),
        marketplacePubkey.toBuffer(),
        rewardMint.toBuffer(),
      ],
      program.programId
    );
    buyerRewardVaults.push([buyerRewardVault, 0]);

    await program.methods
      .initRewardVault()
      .accounts({
        systemProgram: SystemProgram.programId,
        tokenProgramV0: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        rent: SYSVAR_RENT_PUBKEY,
        signer: buyer.publicKey,
        marketplace: marketplacePubkey,
        reward: buyerReward,
        rewardMint: rewardMint,
        rewardVault: buyerRewardVault,
      })
      .signers([buyer])
      .rpc()
      .catch(console.error);

    await sleep(2000);
    const [paymentPubkey] = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("payment", "utf-8"), 
        buyer.publicKey.toBuffer(), 
        productPubkey.toBuffer(),
      ],
      program.programId
    );
    const registerRewardBuyAccounts = {
      systemProgram: SystemProgram.programId,
      tokenProgramV0: TOKEN_PROGRAM_ID,
      rent: SYSVAR_RENT_PUBKEY,
      signer: buyer.publicKey,
      seller: null,
      marketplaceAuth: null,
      marketplace: marketplacePubkey,
      product: productPubkey,
      payment: paymentPubkey,
      paymentMint: rewardMint,
      buyerTransferVault: buyerVaults[1][0],
      sellerTransferVault: sellerVaults[1][0],
      marketplaceTransferVault: marketplaceVaults[1][0],
      bountyVault: bountyVault,
      sellerReward: sellerReward,
      sellerRewardVault: sellerRewardVaults[1][0],
      buyerReward: buyerReward,
      buyerRewardVault: buyerRewardVaults[1][0],
    };

    await program.methods
      .registerBuy(1)
      .accounts(registerRewardBuyAccounts)
      .signers([buyer])
      .rpc(confirmOptions)
      .catch(console.error);

    // check reward vaults
    const oldsellerPromo = 20;
    const expectedSellerReward = Math.floor(Number(productPrice) * oldsellerPromo / 10000);
    const sellerRewardFunds = await getAccount(provider.connection, sellerRewardVaults[1][0]);
    assert.equal(Number(sellerRewardFunds.amount), expectedSellerReward);

    const oldBuyerPromo = 20;
    const expectedBuyerReward = Math.floor(Number(productPrice) * oldBuyerPromo / 10000);
    const buyerRewardFunds = await getAccount(provider.connection, buyerRewardVaults[1][0]);
    assert.equal(Number(buyerRewardFunds.amount), expectedBuyerReward);

    // second time doing the same to test another reward mint (first change marketplace reward mint to check if can be enforced to not get reward with a different mint)
    const newRewardMint = await createMint(provider, confirmOptions);
    await sleep(2000)
    marketplaceVaults.push([
      await createFundedAssociatedTokenAccount(
        provider,
        newRewardMint,
        vaultBalances,
        marketplaceAuth
      ),
      vaultBalances
    ]);
    sellerVaults.push([
      await createFundedAssociatedTokenAccount(
        provider,
        newRewardMint,
        vaultBalances,
        seller
      ),
      vaultBalances
    ]);
    buyerVaults.push([
      await createFundedAssociatedTokenAccount(
        provider,
        newRewardMint,
        vaultBalances,
        buyer
      ),
      vaultBalances
    ]);

    [fee, feeReduction, sellerRewardMarketplace, buyerRewardMarketplace] = [100, 20, 20, 20];
    rewardsEnabled = true;
    const newEditMarketplaceInfoParams = {
      fee: fee,
      feeReduction: feeReduction,
      sellerReward: sellerRewardMarketplace,
      buyerReward: buyerRewardMarketplace,
      useCnfts: useCnfts,
      deliverToken: deliverToken,
      transferable: transferable,
      chainCounter: chainCounter,
      permissionless: permissionless,
      rewardsEnabled: rewardsEnabled,
      accessMintBump: accessMintBump,
      feePayer: FeePayer.Seller,
    };
    const newEditMarketplaceInfoAccounts = {
      signer: marketplaceAuth.publicKey,
      marketplace: marketplacePubkey,
      rewardMint: newRewardMint,
      discountMint: discountMint,
    };
    await program.methods
      .editMarketplace(newEditMarketplaceInfoParams)
      .accounts(newEditMarketplaceInfoAccounts)
      .signers([marketplaceAuth])
      .rpc()
      .catch(console.error);

    await sleep(2000);
    const [newBountyVault] = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("bounty_vault", "utf-8"),
        marketplacePubkey.toBuffer(),
        newRewardMint.toBuffer()
      ],
      program.programId
    );
    await program.methods
      .initBounty()
      .accounts({
        systemProgram: SystemProgram.programId,
        tokenProgramV0: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        rent: SYSVAR_RENT_PUBKEY,
        signer: marketplaceAuth.publicKey,
        marketplace: marketplacePubkey,
        rewardMint: newRewardMint,
        bountyVault: newBountyVault,
      })
      .signers([marketplaceAuth])
      .rpc(confirmOptions)
      .catch(console.error);

    await provider.sendAndConfirm(
      new Transaction()
        .add(
          createTransferInstruction(
            marketplaceVaults[2][0],
            newBountyVault,
            marketplaceAuth.publicKey,
            5000
          )
        ),
      [marketplaceAuth as anchor.web3.Signer]
    );
    bountyVaults.push([bountyVault, 5000]);

    const [newSellerRewardVault] = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("reward_vault", "utf-8"), 
        seller.publicKey.toBuffer(),
        marketplacePubkey.toBuffer(),
        newRewardMint.toBuffer(),
      ],
      program.programId
    );
    sellerRewardVaults.push([newSellerRewardVault, 0]);
    await program.methods
      .initRewardVault()
      .accounts({
        systemProgram: SystemProgram.programId,
        tokenProgramV0: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        rent: SYSVAR_RENT_PUBKEY,
        signer: seller.publicKey,
        marketplace: marketplacePubkey,
        reward: sellerReward,
        rewardMint: newRewardMint,
        rewardVault: newSellerRewardVault,
      })
      .signers([seller])
      .rpc()
      .catch(console.error);

    const [newBuyerRewardVault] = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("reward_vault", "utf-8"), 
        buyer.publicKey.toBuffer(),
        marketplacePubkey.toBuffer(),
        newRewardMint.toBuffer(),
      ],
      program.programId
    );
    buyerRewardVaults.push([newBuyerRewardVault, 0]);

    await program.methods
      .initRewardVault()
      .accounts({
        systemProgram: SystemProgram.programId,
        tokenProgramV0: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        rent: SYSVAR_RENT_PUBKEY,
        signer: buyer.publicKey,
        marketplace: marketplacePubkey,
        reward: buyerReward,
        rewardMint: newRewardMint,
        rewardVault: newBuyerRewardVault,
      })
      .signers([buyer])
      .rpc()
      .catch(console.error);

    await sleep(2000)
    const registerNoRewardBuyAccounts = {
      systemProgram: SystemProgram.programId,
      tokenProgramV0: TOKEN_PROGRAM_ID,
      rent: SYSVAR_RENT_PUBKEY,
      signer: buyer.publicKey,
      seller: null,
      marketplaceAuth: null,
      marketplace: marketplacePubkey,
      product: productPubkey,
      payment: paymentPubkey,
      paymentMint: rewardMint,
      buyerTransferVault: buyerVaults[1][0],
      sellerTransferVault: sellerVaults[1][0],
      marketplaceTransferVault: marketplaceVaults[1][0],
      bountyVault: bountyVault,
      sellerReward: sellerReward,
      sellerRewardVault: sellerRewardVaults[1][0],
      buyerReward: buyerReward,
      buyerRewardVault: buyerRewardVaults[1][0],
    };

    await program.methods
      .registerBuy(1)
      .accounts(registerNoRewardBuyAccounts)
      .signers([buyer])
      .rpc(confirmOptions)
      .catch(console.error);

    // check reward vaults
    const noSellerRewardFunds = await getAccount(provider.connection, sellerRewardVaults[2][0]);
    assert.equal(Number(noSellerRewardFunds.amount), 0);

    const noBuyerRewardFunds = await getAccount(provider.connection, buyerRewardVaults[2][0]);
    assert.equal(Number(noBuyerRewardFunds.amount), 0);

    // now change the product mint to be able to give rewards with that new mint
    await program.methods
      .editProduct(productPrice)
      .accounts({
        signer: seller.publicKey,
        product: productPubkey,
        paymentMint: newRewardMint,
      })
      .signers([seller])
      .rpc()
      .catch(console.error);

    const newRegisterRewardBuyAccounts = {
      systemProgram: SystemProgram.programId,
      tokenProgramV0: TOKEN_PROGRAM_ID,
      rent: SYSVAR_RENT_PUBKEY,
      signer: buyer.publicKey,
      seller: null,
      marketplaceAuth: null,
      marketplace: marketplacePubkey,
      product: productPubkey,
      payment: paymentPubkey,
      paymentMint: newRewardMint,
      buyerTransferVault: buyerVaults[2][0],
      sellerTransferVault: sellerVaults[2][0],
      marketplaceTransferVault: marketplaceVaults[2][0],
      bountyVault: newBountyVault,
      sellerReward: sellerReward,
      sellerRewardVault: sellerRewardVaults[2][0],
      buyerReward: buyerReward,
      buyerRewardVault: buyerRewardVaults[2][0],
    };

    await program.methods
      .registerBuy(1)
      .accounts(newRegisterRewardBuyAccounts)
      .signers([buyer])
      .rpc(confirmOptions)
      .catch(console.error);

    // check reward vaults
    const newExpectedSellerReward = Math.floor(Number(productPrice) * oldsellerPromo / 10000);
    const newSellerRewardFunds = await getAccount(provider.connection, sellerRewardVaults[2][0]);
    assert.equal(Number(newSellerRewardFunds.amount), newExpectedSellerReward);

    const newExpectedBuyerReward = Math.floor(Number(productPrice) * oldBuyerPromo / 10000);
    const newBuyerRewardFunds = await getAccount(provider.connection, buyerRewardVaults[2][0]);
    assert.equal(Number(newBuyerRewardFunds.amount), newExpectedBuyerReward);
    // withdraw rewards (both mints done before)
    // promo is finished with rewardsEnable = false
    rewardsEnabled = false;
    const changeMarketplaceInfoParams = {
      fee: fee,
      feeReduction: feeReduction,
      sellerReward: sellerRewardMarketplace,
      buyerReward: buyerRewardMarketplace,
      useCnfts: useCnfts,
      deliverToken: deliverToken,
      transferable: transferable,
      chainCounter: chainCounter,
      permissionless: permissionless,
      rewardsEnabled: rewardsEnabled,
      accessMintBump: accessMintBump,
      feePayer: FeePayer.Seller,
    };
    const changeMarketplaceInfoAccounts = {
      signer: marketplaceAuth.publicKey,
      marketplace: marketplacePubkey,
      rewardMint: rewardMint,
      discountMint: discountMint,
    };

    await program.methods
      .editMarketplace(changeMarketplaceInfoParams)
      .accounts(changeMarketplaceInfoAccounts)
      .signers([marketplaceAuth])
      .rpc()
      .catch(console.error);

    await sleep(2000);

    const preBuyerVault1Funds = await getAccount(provider.connection, buyerVaults[1][0]);
    await program.methods
      .withdrawReward()
      .accounts({
        tokenProgramV0: TOKEN_PROGRAM_ID,
        signer: buyer.publicKey,
        marketplace: marketplacePubkey,
        reward: buyerReward,
        rewardMint: rewardMint,
        receiverVault: buyerVaults[1][0],
        rewardVault: buyerRewardVaults[1][0],
      })
      .signers([buyer])
      .rpc()
      .catch(console.error);

    await sleep(1000);
    const postBuyerVault1Funds = await getAccount(provider.connection, buyerVaults[1][0]);
    assert.equal(Number(postBuyerVault1Funds.amount - preBuyerVault1Funds.amount), Number(buyerRewardFunds.amount));

    const preSellerVault1Funds = await getAccount(provider.connection, sellerVaults[1][0]);
    await program.methods
      .withdrawReward()
      .accounts({
        tokenProgramV0: TOKEN_PROGRAM_ID,
        signer: seller.publicKey,
        marketplace: marketplacePubkey,
        reward: sellerReward,
        rewardMint: rewardMint,
        receiverVault: sellerVaults[1][0],
        rewardVault: sellerRewardVaults[1][0],
      })
      .signers([seller])
      .rpc()
      .catch(console.error);
    
    await sleep(1000);
    const postSellerVault1Funds = await getAccount(provider.connection, sellerVaults[1][0]);
    assert.equal(Number(postSellerVault1Funds.amount - preSellerVault1Funds.amount), Number(sellerRewardFunds.amount));

    const preBuyerVault2Funds = await getAccount(provider.connection, buyerVaults[2][0]);
    await program.methods
      .withdrawReward()
      .accounts({
        tokenProgramV0: TOKEN_PROGRAM_ID,
        signer: buyer.publicKey,
        marketplace: marketplacePubkey,
        reward: buyerReward,
        rewardMint: newRewardMint,
        receiverVault: buyerVaults[2][0],
        rewardVault: buyerRewardVaults[2][0],
      })
      .signers([buyer])
      .rpc()
      .catch(console.error);

    await sleep(1000);
    const postBuyerVault2Funds = await getAccount(provider.connection, buyerVaults[2][0]);
    assert.equal(Number(postBuyerVault2Funds.amount - preBuyerVault2Funds.amount), Number(newBuyerRewardFunds.amount));

    const preSellerVault2Funds = await getAccount(provider.connection, sellerVaults[2][0]);
    await program.methods
      .withdrawReward()
      .accounts({
        tokenProgramV0: TOKEN_PROGRAM_ID,
        signer: seller.publicKey,
        marketplace: marketplacePubkey,
        reward: sellerReward,
        rewardMint: newRewardMint,
        receiverVault: sellerVaults[2][0],
        rewardVault: sellerRewardVaults[2][0],
      })
      .signers([seller])
      .rpc()
      .catch(console.error);

    await sleep(1000);
    const postSellerVault2Funds = await getAccount(provider.connection, sellerVaults[2][0]);
    assert.equal(Number(postSellerVault2Funds.amount - preSellerVault2Funds.amount), Number(newSellerRewardFunds.amount));
  });

  it("Should handle correctly FeePayer.Buyer as config (FeePayer.Seller was used in other examples)", async () => {
    const newPaymentMintPubkey = NATIVE_MINT;
    const newPrice = new BN(1000);

    const editProductInfoAccounts = {
      signer: seller.publicKey,
      product: productPubkey,
      paymentMint: newPaymentMintPubkey,
    };
    await program.methods
      .editProduct(newPrice)
      .accounts(editProductInfoAccounts)
      .signers([seller])
      .rpc()
      .catch(console.error);

    const editMarketplaceInfoParams = {
      fee: 100,
      feeReduction: 0,
      sellerReward: 100,
      buyerReward: 100,
      useCnfts: false,
      deliverToken: false,
      transferable: false,
      chainCounter: true,
      permissionless: true,
      rewardsEnabled: false,
      feePayer: FeePayer.Buyer,
    };

    const editMarketplaceInfoAccounts = {
      signer: marketplaceAuth.publicKey,
      marketplace: marketplacePubkey,
      rewardMint: rewardMint,
      discountMint: discountMint,
    };

    await program.methods
      .editMarketplace(editMarketplaceInfoParams)
      .accounts(editMarketplaceInfoAccounts)
      .signers([marketplaceAuth])
      .rpc()
      .catch(console.error);
  
    const [paymentPubkey, bump] = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("payment", "utf-8"), 
        buyer.publicKey.toBuffer(), 
        productPubkey.toBuffer(),
      ],
      program.programId
    );

    const marketAuthBalance = await provider.connection.getBalance(marketplaceAuth.publicKey, confirmOptions);
    const sellerBalance = await provider.connection.getBalance(seller.publicKey, confirmOptions);
    const buyerBalance = await provider.connection.getBalance(buyer.publicKey, confirmOptions);

    const registerBuyAccounts = {
      systemProgram: SystemProgram.programId,
      tokenProgramV0: TOKEN_PROGRAM_ID,
      rent: SYSVAR_RENT_PUBKEY,
      signer: buyer.publicKey,
      seller: seller.publicKey,
      marketplaceAuth: marketplaceAuth.publicKey,
      marketplace: marketplacePubkey,
      product: productPubkey,
      payment: paymentPubkey,
      paymentMint: newPaymentMintPubkey,
      buyerTokenVault: null,
      buyerTransferVault: null,
      sellerTransferVault: null,
      marketplaceTransferVault: null,
      bountyVault: null,
      sellerReward: null,
      sellerRewardVault: null,
      buyerReward: null,
      buyerRewardVault: null,
    };

    await program.methods
      .registerBuy(1)
      .accounts(registerBuyAccounts)
      .signers([buyer])
      .rpc()
      .catch(console.error);

    await sleep(2000);
    const postMarketAuthBalance = await provider.connection.getBalance(marketplaceAuth.publicKey, confirmOptions);
    const postSellerBalance = await provider.connection.getBalance(seller.publicKey, confirmOptions);
    const postBuyerBalance = await provider.connection.getBalance(buyer.publicKey, confirmOptions);
    const marketplaceFee = Math.floor((Number(newPrice) * fee) / 10000);

    assert.equal(postMarketAuthBalance, marketAuthBalance + marketplaceFee);
    assert.equal(postSellerBalance, sellerBalance + Number(newPrice));
    assert.equal(postBuyerBalance, buyerBalance - Number(newPrice) - marketplaceFee);
  });

  it("Marketplace auth airdrops access token", async () => {
    const accounts = {
      systemProgram: SystemProgram.programId,
      tokenProgram2022: TOKEN_2022_PROGRAM_ID,
      associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      rent: SYSVAR_RENT_PUBKEY,
      signer: marketplaceAuth.publicKey,
      receiver: exploiter.publicKey,
      marketplace: marketplacePubkey,
      accessMint: accessMint,
      accessVault: getAssociatedTokenAddressSync(accessMint, exploiter.publicKey, false, TOKEN_2022_PROGRAM_ID),
    }

    const tx = await program.methods
      .airdropAccess()
      .accounts(accounts)
      .signers([marketplaceAuth])
      .rpc()
      .catch(console.error);

  });

  it("Should make the marketplace token-gated", async () => {
    permissionless = false;
    const editMarketplaceInfoParams = {
      fee: fee,
      feeReduction: feeReduction,
      sellerReward: sellerRewardMarketplace,
      buyerReward: buyerRewardMarketplace,
      useCnfts: useCnfts,
      deliverToken: deliverToken,
      transferable: transferable,
      chainCounter: chainCounter,
      permissionless: permissionless,
      rewardsEnabled: rewardsEnabled,
      accessMintBump: accessMintBump,
      feePayer: FeePayer.Buyer,
    };

    const editMarketplaceInfoAccounts = {
      signer: marketplaceAuth.publicKey,
      marketplace: marketplacePubkey,
      rewardMint: await createMint(provider, confirmOptions),
      discountMint: await createMint(provider, confirmOptions),
    };

    await program.methods
      .editMarketplace(editMarketplaceInfoParams)
      .accounts(editMarketplaceInfoAccounts)
      .signers([marketplaceAuth])
      .rpc()
      .catch(console.error);

    const receiverVault = getAssociatedTokenAddressSync(accessMint, seller.publicKey, false, TOKEN_2022_PROGRAM_ID);
    const [request] = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("request", "utf-8"),
        seller.publicKey.toBuffer(),
        marketplacePubkey.toBuffer()
      ],
      program.programId
    );
    const initRequestAccounts = {
      systemProgram: SystemProgram.programId,
      rent: SYSVAR_RENT_PUBKEY,
      signer: seller.publicKey,
      marketplace: marketplacePubkey,
      request: request,
    };
    
    await program.methods
      .requestAccess()
      .accounts(initRequestAccounts)
      .signers([seller])
      .rpc()
      .catch(console.error);

    const acceptRequestAccounts = {
      systemProgram: SystemProgram.programId,
      tokenProgram2022: TOKEN_2022_PROGRAM_ID,
      associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      rent: SYSVAR_RENT_PUBKEY,
      signer: marketplaceAuth.publicKey,
      receiver: seller.publicKey,
      marketplace: marketplacePubkey,
      request: request,
      accessMint: accessMint,
      accessVault: receiverVault,
    };

    await program.methods
      .acceptAccess()
      .accounts(acceptRequestAccounts)
      .signers([marketplaceAuth])
      .rpc()
      .catch(console.error);

    [firstId, secondId] = getSplitId(uuid());
    const balance = 1000;
    seller = await createFundedWallet(provider, balance);
    [productPubkey] = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("product", "utf-8"), 
        firstId, 
        secondId,
        marketplacePubkey.toBuffer()
      ],
      program.programId
    );
    [productMint, mintBump] = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("product_mint", "utf-8"), 
        productPubkey.toBuffer()
      ],
      program.programId
    );
    productPrice = new BN(100);
    const initProductParams = {
      firstId: [...firstId],
      secondId: [...secondId],
      productPrice: productPrice,
      productMintBump: mintBump,
    };
    const accessVault = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      seller as anchor.web3.Signer,
      accessMint,
      seller.publicKey,
      false,
      "confirmed",
      confirmOptions,
      TOKEN_2022_PROGRAM_ID,
    );
    const initProductAccounts = {
      systemProgram: SystemProgram.programId,
      tokenProgram2022: TOKEN_2022_PROGRAM_ID,
      rent: SYSVAR_RENT_PUBKEY,
      signer: seller.publicKey,
      marketplace: marketplacePubkey,
      product: productPubkey,
      productMint: productMint,
      paymentMint: paymentMints[0],
      accessMint: accessMint,
      accessVault: accessVault.address,
    };
    await program.methods
      .initProduct(initProductParams)
      .accounts(initProductAccounts)
      .signers([seller])
      .rpc()
      .catch(console.error);

    [firstId, secondId] = getSplitId(uuid());
    [productPubkey] = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("product", "utf-8"), 
        firstId, 
        secondId,
        marketplacePubkey.toBuffer()
      ],
      program.programId
    );
    [productMint, mintBump] = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("product_mint", "utf-8"), 
        productPubkey.toBuffer()
      ],
      program.programId
    );
    productPrice = new BN(100);
    const buyerVault = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      buyer as anchor.web3.Signer,
      accessMint,
      buyer.publicKey,
      false,
      "confirmed",
      confirmOptions,
      TOKEN_2022_PROGRAM_ID,
    );
    const initErrorProductParams = {
      firstId: [...firstId],
      secondId: [...secondId],
      productPrice: productPrice,
      productMintBump: mintBump,
    };
    const initErrorProductAccounts = {
      systemProgram: SystemProgram.programId,
      tokenProgram2022: TOKEN_2022_PROGRAM_ID,
      rent: SYSVAR_RENT_PUBKEY,
      signer: buyer.publicKey,
      marketplace: marketplacePubkey,
      product: productPubkey,
      productMint: productMint,
      paymentMint: paymentMints[0],
      accessMint: accessMint,
      accessVault: buyerVault.address,
    };
    try {
      await program.methods
        .initProduct(initErrorProductParams)
        .accounts(initErrorProductAccounts)
        .signers([buyer])
        .rpc();
    } catch (e) {
      if (e as anchor.AnchorError)
        assert.equal(e.error.errorCode.code, "NotInWithelist");
    }

    try {
      await provider.sendAndConfirm(
        new Transaction()
          .add(
            createTransferInstruction(
              receiverVault,
              buyerVault.address,
              seller.publicKey,
              1,
              [],
              TOKEN_2022_PROGRAM_ID
            )
          ),
        [seller]
      );
    } catch(e) {
      // the decimal equivalent of hexadecimal 0x25, it's 37 in decimal 
      // ie NonTransferable error in the t2022 program 
      assert.isTrue(e.toString().includes("0x25"));
    }
  });
})

function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}