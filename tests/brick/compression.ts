import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { assert } from "chai";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  TOKEN_2022_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
  getAssociatedTokenAddressSync,
} from "@solana/spl-token";
import { 
  createFundedAssociatedTokenAccount,
  createFundedWallet, 
  createMint, 
  getSplitId 
} from "./utils";
import { 
  ComputeBudgetProgram,
  ConfirmOptions, 
  SYSVAR_RENT_PUBKEY, 
  SystemProgram, 
} from "@solana/web3.js";
import { Brick } from "../../target/types/brick";
import BN from "bn.js";
import { v4 as uuid } from "uuid";
import { 
  SPL_ACCOUNT_COMPRESSION_ADDRESS, 
  SPL_ACCOUNT_COMPRESSION_PROGRAM_ID, 
  SPL_NOOP_PROGRAM_ID, 
  getConcurrentMerkleTreeAccountSize 
} from "@solana/spl-account-compression";
import { PROGRAM_ID as BUBBLEGUM_PROGRAM } from "@metaplex-foundation/mpl-bubblegum";
import { PROGRAM_ID as METADATA_PROGRAM } from "@metaplex-foundation/mpl-token-metadata";

describe("brick compression", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.Brick as Program<Brick>;
  const confirmOptions: ConfirmOptions = { commitment: "confirmed" };

  // Keypairs:
  let marketplaceAuth: anchor.web3.Keypair;
  let seller: anchor.web3.Keypair;
  let buyer: anchor.web3.Keypair;

  // Mints, vaults and balances:
  let paymentMint: anchor.web3.PublicKey;
  let productMint: anchor.web3.PublicKey;
  let mintBump: number;
  let marketplaceVault: anchor.web3.PublicKey;
  let buyerVault: anchor.web3.PublicKey;
  let sellerVault: anchor.web3.PublicKey;
  let bountyVault: anchor.web3.PublicKey;

  // Program account addresses:
  let marketplacePubkey: anchor.web3.PublicKey;
  let productPubkey: anchor.web3.PublicKey;

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

  // Compression:
  let merkleTree: anchor.web3.Keypair;
  let metadata: anchor.web3.PublicKey;
  let masterEdition: anchor.web3.PublicKey;
  let treeAuthority: anchor.web3.PublicKey;
  let bubblegumSigner: anchor.web3.PublicKey;

  it("Should create marketplace with cNFTs config", async () => {
    rewardMint = discountMint = paymentMint = await createMint(provider, confirmOptions);

    const balance = 1000;
    marketplaceAuth = await createFundedWallet(provider, balance);

    [marketplacePubkey] = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("marketplace", "utf-8"),
        marketplaceAuth.publicKey.toBuffer()
      ],
      program.programId
    );

    [bountyVault] = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("bounty_vault", "utf-8"),
        marketplacePubkey.toBuffer(),
        rewardMint.toBuffer()
      ],
      program.programId
    );

    fee = feeReduction = sellerRewardMarketplace = buyerRewardMarketplace = 0;
    deliverToken = transferable = rewardsEnabled = false;
    chainCounter = permissionless = useCnfts = true;

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
    assert.equal(marketplaceAccount.rewardsConfig.bountyVaults[0].toString(), bountyVault.toString());
    assert.equal(marketplaceAccount.rewardsConfig.sellerReward, sellerRewardMarketplace);
    assert.equal(marketplaceAccount.rewardsConfig.buyerReward, buyerRewardMarketplace);
    assert.equal(marketplaceAccount.rewardsConfig.rewardsEnabled, rewardsEnabled);
  });

  it("Should create a product account (with a tree)", async () => {
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
    [masterEdition] = anchor.web3.PublicKey.findProgramAddressSync(
        [
          Buffer.from("metadata", "utf-8"),
          METADATA_PROGRAM.toBuffer(),
          productMint.toBuffer(),
          Buffer.from("edition", "utf-8"),
        ],
        METADATA_PROGRAM
    );
    [metadata] = anchor.web3.PublicKey.findProgramAddressSync(
        [
          Buffer.from("metadata", "utf-8"),
          METADATA_PROGRAM.toBuffer(),
          productMint.toBuffer(),
        ],
        METADATA_PROGRAM
    );
    merkleTree = anchor.web3.Keypair.generate();
    [treeAuthority] = anchor.web3.PublicKey.findProgramAddressSync(
        [merkleTree.publicKey.toBuffer()], BUBBLEGUM_PROGRAM
    );
    productPrice = new BN(10000);
    const [height, buffer, canopy] = [14, 64, 11];
    const space = getConcurrentMerkleTreeAccountSize(height, buffer, canopy);
    const cost = await provider.connection.getMinimumBalanceForRentExemption(space);
    await provider.sendAndConfirm(
        new anchor.web3.Transaction()
          .add(
            SystemProgram.createAccount({
                fromPubkey: provider.wallet.publicKey,
                newAccountPubkey: merkleTree.publicKey,
                lamports: cost,
                space: space,
                programId: SPL_ACCOUNT_COMPRESSION_PROGRAM_ID,
            }),
          ),
        [merkleTree]
    );
    const initProductParams = {
        firstId: [...firstId],
        secondId: [...secondId],
        productPrice: productPrice,
        maxDepth: height,
        maxBufferSize: buffer,
        name: "DATASET",
        metadataUrl: "test",
        feeBasisPoints: 0,
        productMintBump: mintBump,
    };
    const initProductAccounts = {
        tokenMetadataProgram: METADATA_PROGRAM,
        logWrapper: SPL_NOOP_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        bubblegumProgram: BUBBLEGUM_PROGRAM,
        compressionProgram: SPL_ACCOUNT_COMPRESSION_ADDRESS,
        tokenProgramV0: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        rent: SYSVAR_RENT_PUBKEY,
        signer: seller.publicKey,
        marketplace: marketplacePubkey,
        product: productPubkey,
        productMint: productMint,
        accessMint: null,
        paymentMint: paymentMint,
        accessVault: null,
        productMintVault: getAssociatedTokenAddressSync(productMint, productPubkey, true),
        masterEditrion: masterEdition,
        metadata: metadata,
        merkleTree: merkleTree.publicKey,
        treeAuthority: treeAuthority,
    };

    await program.methods
      .initProductTree(initProductParams)
      .accounts(initProductAccounts)
      .signers([seller])
      .preInstructions([
        ComputeBudgetProgram.setComputeUnitLimit({ units: 350000 }),
      ])
      .rpc(confirmOptions)
      .catch(console.error);

    const productAccount = await program.account.product.fetch(productPubkey);
    assert.isDefined(productAccount);
    assert.equal(productAccount.authority.toString(), seller.publicKey.toString());
    assert.equal(productAccount.firstId.toString(), [...firstId].toString());
    assert.equal(productAccount.secondId.toString(), [...secondId].toString());
    assert.equal(productAccount.marketplace.toString(), marketplacePubkey.toString());
    assert.equal(productAccount.productMint.toString(), productMint.toString());
    assert.equal(productAccount.sellerConfig.paymentMint.toString(), paymentMint.toString());
    assert.equal(Number(productAccount.sellerConfig.productPrice), Number(productPrice));
  });

  it("Should register a buy and mint a cNFT", async () => {
    const buyerSOLBalance = 1000;
    buyer = await createFundedWallet(provider, buyerSOLBalance);
    const vaultBalances = 1000000000;
    marketplaceVault = await createFundedAssociatedTokenAccount(
      provider,
      paymentMint,
      vaultBalances,
      marketplaceAuth
    );
    sellerVault = await createFundedAssociatedTokenAccount(
      provider,
      paymentMint,
      vaultBalances,
      seller
    );
    buyerVault = await createFundedAssociatedTokenAccount(
      provider,
      paymentMint,
      vaultBalances,
      buyer
    );
    [bubblegumSigner] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("collection_cpi", "utf-8")], BUBBLEGUM_PROGRAM
    );
    const registerNoRewardBuyAccounts = {
      systemProgram: SystemProgram.programId,
      tokenProgramV0: TOKEN_PROGRAM_ID,
      rent: SYSVAR_RENT_PUBKEY,
      logWrapper: SPL_NOOP_PROGRAM_ID,
      bubblegumProgram: BUBBLEGUM_PROGRAM,
      compressionProgram: SPL_ACCOUNT_COMPRESSION_ADDRESS,
      tokenMetadataProgram: METADATA_PROGRAM,
      signer: buyer.publicKey,
      seller: null,
      marketplaceAuth: null,
      marketplace: marketplacePubkey,
      product: productPubkey,
      paymentMint: paymentMint,
      productMint: productMint,
      buyerTransferVault: buyerVault,
      sellerTransferVault: sellerVault,
      marketplaceTransferVault: marketplaceVault,
      bountyVault: null,
      sellerReward: null,
      sellerRewardVault: null,
      buyerReward: null,
      buyerRewardVault: null,
      metadata: metadata,
      masterEdition: masterEdition,
      treeAuthority: treeAuthority,
      bubblegumSigner: bubblegumSigner,
      merkleTree: merkleTree.publicKey,
    };
    const registerBuyCnftsParams = {
      amount: 1,
      name: "DATASET",
      symbol: "BRICK",
      uri: "TEST"
    };

    await program.methods
      .registerBuyCnft(registerBuyCnftsParams)
      .accounts(registerNoRewardBuyAccounts)
      .signers([buyer])
      .rpc(confirmOptions)
      .catch(console.error);
  });
})