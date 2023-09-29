import { ProductManager } from "../../target/types/product_manager";
import * as anchor from "@coral-xyz/anchor";
import { v4 as uuid, parse } from "uuid";
import { assert } from "chai";
import {
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  AccountLayout,
} from "@solana/spl-token";
import { 
  createFundedAssociatedTokenAccount, 
  createFundedWallet, 
  createMint,
  delay,
} from "../utils";
import { 
  ConfirmOptions, 
  SYSVAR_RENT_PUBKEY, 
  SystemProgram, 
} from "@solana/web3.js";
import BN from "bn.js";

describe("product_manager", () => {
    const provider = anchor.AnchorProvider.env();
    anchor.setProvider(provider);
    const program = anchor.workspace.ProductManager as anchor.Program<ProductManager>;
    const confirmOptions: ConfirmOptions = { commitment: "confirmed" };

    // Keypairs:
    let seller: anchor.web3.Keypair;
    let buyer: anchor.web3.Keypair;

    // Mints, vaults and balances:
    let paymentMint: anchor.web3.PublicKey;
    let buyerVault: [anchor.web3.PublicKey, number];
    let sellerVault: [anchor.web3.PublicKey, number];
    let vaultsInitialBalance = 1000;

    // Program account addresses:
    let productPubkey: anchor.web3.PublicKey;
    let productBump: number;
    let escrowPubkey: anchor.web3.PublicKey;
    let escrowBump: number;
    let escrowVaultPubkey: anchor.web3.PublicKey;
    let escrowVaultBump: number;

    // Product properties
    let price: BN;
    let id: Uint8Array;

    // Escrow properties
    let expireTime: BN;

    let expectedExpirationTime: number;

    it("Should create a product", async () => {
        seller = await createFundedWallet(provider, vaultsInitialBalance);
        paymentMint = await createMint(provider, confirmOptions);

        id = parse(uuid());
        [productPubkey, productBump] = anchor.web3.PublicKey.findProgramAddressSync(
            [
                Buffer.from("product", "utf-8"),
                seller.publicKey.toBuffer(),
                id,
            ],
            program.programId
        );
        price = new BN(10);

        const initProductAccounts = {
            signer: seller.publicKey,
            product: productPubkey,
            paymentMint: paymentMint,
            systemProgram: SystemProgram.programId,
        };

        await program.methods
            .initProduct([...id], price)
            .accounts(initProductAccounts)
            .signers([seller])
            .rpc(confirmOptions)
            .catch(console.error);

        const productAccount = await program.account.product.fetch(productPubkey);
        assert.isDefined(productAccount);
        assert.equal(productAccount.id.toString(), id.toString());
        assert.equal(productAccount.authority.toString(), seller.publicKey.toString());
        assert.equal(productAccount.paymentMint.toString(), paymentMint.toString());
        assert.equal(Number(productAccount.price), Number(price));
        assert.equal(productAccount.bump, productBump);
    });

    it("Should pay a product, money sent to a escrow", async () => {
        buyer = await createFundedWallet(provider, vaultsInitialBalance);
        sellerVault = [
            await createFundedAssociatedTokenAccount(
                provider,
                paymentMint,
                0,
                seller
            ),
            0
        ];
        buyerVault = [
            await createFundedAssociatedTokenAccount(
                provider,
                paymentMint,
                vaultsInitialBalance,
                buyer
            ),
            vaultsInitialBalance
        ];
        [escrowPubkey, escrowBump] = anchor.web3.PublicKey.findProgramAddressSync(
            [
                Buffer.from("escrow", "utf-8"), 
                productPubkey.toBuffer(),
                buyer.publicKey.toBuffer()
            ],
            program.programId
        );
        [escrowVaultPubkey, escrowVaultBump] = anchor.web3.PublicKey.findProgramAddressSync(
            [
                Buffer.from("escrow_vault", "utf-8"), 
                productPubkey.toBuffer(),
                buyer.publicKey.toBuffer()
            ],
            program.programId
        );
        expireTime = new BN(5); // 5 seconds

        const payAccounts = {
            signer: buyer.publicKey,
            seller: seller.publicKey,
            product: productPubkey,
            escrow: escrowPubkey,
            escrowVault: escrowVaultPubkey,
            transferVault: buyerVault[0],
            paymentMint: paymentMint,
            rent: SYSVAR_RENT_PUBKEY,
            systemProgram: SystemProgram.programId,
            tokenProgram: TOKEN_PROGRAM_ID,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        };

        await program.methods
            .pay(expireTime)
            .accounts(payAccounts)
            .signers([buyer])
            .rpc(confirmOptions)
            .catch(console.error);

        const escrowAccount = await program.account.escrow.fetch(escrowPubkey);
        assert.isDefined(escrowAccount);
        assert.equal(escrowAccount.buyer.toString(), buyer.publicKey.toString());
        assert.equal(escrowAccount.seller.toString(), seller.publicKey.toString());
        assert.equal(escrowAccount.vaultBump, escrowVaultBump);
        assert.equal(escrowAccount.bump, escrowBump);

        const escrowExpirationDate = new Date(Number(escrowAccount.expireTime));
        const escrowExpirationHour = escrowExpirationDate.getHours();
        const escrowExpirationMinute = escrowExpirationDate.getMinutes();
        const escrowExpirationSeconds = escrowExpirationDate.getSeconds();
        const onChainDate = `${escrowExpirationHour}:${escrowExpirationMinute}:${escrowExpirationSeconds}`;

        const slot = await program.provider.connection.getSlot();
        expectedExpirationTime = await program.provider.connection.getBlockTime(slot) + Number(expireTime);
        const expectedExpirationDate = new Date(expectedExpirationTime);
        const expectedExpirationHour = expectedExpirationDate.getHours();
        const expectedExpirationMinute = expectedExpirationDate.getMinutes();
        const expectedExpirationSeconds = expectedExpirationDate.getSeconds();
        const offChainDate = `${expectedExpirationHour}:${expectedExpirationMinute}:${expectedExpirationSeconds}`;
        assert.equal(onChainDate, offChainDate);
        
        const buyerVaultInfo = await program.provider.connection.getAccountInfo(buyerVault[0]);
        const decodedBuyerATA = AccountLayout.decode(buyerVaultInfo.data);
        buyerVault[1] = vaultsInitialBalance - Number(price);
        assert.equal(Number(decodedBuyerATA.amount), buyerVault[1]);
        
        const escrowVaultInfo = await program.provider.connection.getAccountInfo(escrowVaultPubkey);
        const decodedEscrowData = AccountLayout.decode(escrowVaultInfo.data);
        assert.equal(Number(decodedEscrowData.amount), Number(price));
    });

    /* 
        buyer or any other keypairs cant withdraw the money with this instructions, only the seller.
        the context is where the condition is enforced, the signer of the instruction has to be seller,
        because is stored in the product and the product is also stored on the escrow.
        
            #[account(
                mut,
                seeds = [
                    b"escrow".as_ref(),
                    product.key().as_ref(),
                    buyer.key().as_ref()
                ],
                bump = escrow.bump,
                constraint = escrow.seller == signer.key()
                    @ ErrorCode::IncorrectAuthority,
                constraint = escrow.product == product.key()
                    @ ErrorCode::IncorrectProduct,
                close = buyer,
            )]
            pub escrow: Account<'info, Escrow>,
            #[account(
                mut,
                seeds = [
                    b"product".as_ref(),
                    signer.key().as_ref(),
                    product.id.as_ref()        
                ],
                bump = product.bump,
                constraint = signer.key() == product.authority 
                    @ ErrorCode::IncorrectAuthority,
                constraint = product.payment_mint == payment_mint.key()
                    @ ErrorCode::IncorrectMint
            )]
            pub product: Account<'info, Product>,
    */
    it("After expire time only the buyer can withdraw, before this time only the seller can", async () => {
        const buyerAccounts = {
            signer: buyer.publicKey,
            buyer: buyer.publicKey,
            product: productPubkey,
            escrow: escrowPubkey,
            escrowVault: escrowVaultPubkey,
            transferVault: buyerVault[0],
            paymentMint: paymentMint,
            systemProgram: SystemProgram.programId,
            tokenProgram: TOKEN_PROGRAM_ID,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        };

        try {
            await program.methods
                .accept()
                .accounts(buyerAccounts)
                .signers([buyer])
                .rpc(confirmOptions);
        } catch(e) {
            if (e as anchor.AnchorError) {
                assert.equal(e.error.errorCode.code, "IncorrectAuthority");
            }
        }

        try {
            await program.methods
                .deny()
                .accounts(buyerAccounts)
                .signers([buyer])
                .rpc(confirmOptions);
        } catch(e) {
            if (e as anchor.AnchorError) {
                assert.equal(e.error.errorCode.code, "IncorrectAuthority");
            }
        }

        const escrowAccount = await program.account.escrow.fetch(escrowPubkey);
        await awaitUntilTimestamp(Number(escrowAccount.expireTime));

        const sellerRecoverAccounts = {
            signer: seller.publicKey,
            seller: seller.publicKey,
            product: productPubkey,
            escrow: escrowPubkey,
            escrowVault: escrowVaultPubkey,
            transferVault: sellerVault[0],
            paymentMint: paymentMint,
            systemProgram: SystemProgram.programId,
            tokenProgram: TOKEN_PROGRAM_ID,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        };

        try {
            await program.methods
                .recoverFunds()
                .accounts(sellerRecoverAccounts)
                .signers([seller])
                .rpc(confirmOptions);
        } catch (e) {
            if (e as anchor.AnchorError) {
                assert.equal(e.error.errorCode.code, "ConstraintSeeds");
            }
        }

        const acceptAccounts = {
            signer: seller.publicKey,
            buyer: buyer.publicKey,
            product: productPubkey,
            escrow: escrowPubkey,
            escrowVault: escrowVaultPubkey,
            transferVault: sellerVault[0],
            paymentMint: paymentMint,
            systemProgram: SystemProgram.programId,
            tokenProgram: TOKEN_PROGRAM_ID,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        };

        try {
            await program.methods
                .accept()
                .accounts(acceptAccounts)
                .signers([seller])
                .rpc(confirmOptions);
        } catch (e) {
            if (e as anchor.AnchorError) {
                assert.equal(e.error.errorCode.code, "TimeExpired");
            }
        }

        const recoverAccounts = {
            signer: buyer.publicKey,
            seller: seller.publicKey,
            product: productPubkey,
            escrow: escrowPubkey,
            escrowVault: escrowVaultPubkey,
            transferVault: buyerVault[0],
            paymentMint: paymentMint,
            systemProgram: SystemProgram.programId,
            tokenProgram: TOKEN_PROGRAM_ID,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        };

        await program.methods
            .recoverFunds()
            .accounts(recoverAccounts)
            .signers([buyer])
            .rpc(confirmOptions)
            .catch(console.error);

        const buyerVaultInfo = await program.provider.connection.getAccountInfo(buyerVault[0]);
        const decodedBuyerATA = AccountLayout.decode(buyerVaultInfo.data);
        buyerVault[1] = vaultsInitialBalance;
        assert.equal(Number(decodedBuyerATA.amount), buyerVault[1]);
        
        const escrowVaultInfo = await program.provider.connection.getAccountInfo(escrowVaultPubkey);
        const escrowInfo = await program.provider.connection.getAccountInfo(escrowPubkey);

        if (escrowVaultInfo || escrowInfo) {
            throw new Error('Escrow and escrow vault should have been closed')
        } else {
            console.log('Escrow and escrow vault are closed after withdrawal')
        }
    });

    it("Should accept the delivery of the product/service, money sent to the seller", async () => {
        const payAccounts = {
            signer: buyer.publicKey,
            seller: seller.publicKey,
            product: productPubkey,
            escrow: escrowPubkey,
            escrowVault: escrowVaultPubkey,
            transferVault: buyerVault[0],
            paymentMint: paymentMint,
            rent: SYSVAR_RENT_PUBKEY,
            systemProgram: SystemProgram.programId,
            tokenProgram: TOKEN_PROGRAM_ID,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        };

        await program.methods
            .pay(expireTime)
            .accounts(payAccounts)
            .signers([buyer])
            .rpc(confirmOptions)
            .catch(console.error);

        buyerVault[1] = buyerVault[1] - Number(price);

        const acceptAccounts = {
            signer: seller.publicKey,
            buyer: buyer.publicKey,
            product: productPubkey,
            escrow: escrowPubkey,
            escrowVault: escrowVaultPubkey,
            transferVault: sellerVault[0],
            paymentMint: paymentMint,
            systemProgram: SystemProgram.programId,
            tokenProgram: TOKEN_PROGRAM_ID,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        };

        await program.methods
            .accept()
            .accounts(acceptAccounts)
            .signers([seller])
            .rpc(confirmOptions)
            .catch(console.error);

        const sellerVaultInfo = await program.provider.connection.getAccountInfo(sellerVault[0]);
        const decodedSellerATA = AccountLayout.decode(sellerVaultInfo.data);
        sellerVault[1] = Number(price);
        assert.equal(Number(decodedSellerATA.amount), sellerVault[1]);
        
        const escrowVaultInfo = await program.provider.connection.getAccountInfo(escrowVaultPubkey);
        const escrowInfo = await program.provider.connection.getAccountInfo(escrowPubkey);

        if (escrowVaultInfo || escrowInfo) {
            throw new Error('Escrow and escrow vault should have been closed')
        } else {
            console.log('Escrow and escrow vault are closed after withdrawal')
        }
    });

    it("Should deny the delivery of the product/service, money sent to the buyer", async () => {
        const payAccounts = {
            signer: buyer.publicKey,
            seller: seller.publicKey,
            product: productPubkey,
            escrow: escrowPubkey,
            escrowVault: escrowVaultPubkey,
            transferVault: buyerVault[0],
            paymentMint: paymentMint,
            rent: SYSVAR_RENT_PUBKEY,
            systemProgram: SystemProgram.programId,
            tokenProgram: TOKEN_PROGRAM_ID,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        };

        await program.methods
            .pay(expireTime)
            .accounts(payAccounts)
            .signers([buyer])
            .rpc(confirmOptions)
            .catch(console.error);

        const acceptAccounts = {
            signer: seller.publicKey,
            buyer: buyer.publicKey,
            product: productPubkey,
            escrow: escrowPubkey,
            escrowVault: escrowVaultPubkey,
            transferVault: buyerVault[0],
            paymentMint: paymentMint,
            systemProgram: SystemProgram.programId,
            tokenProgram: TOKEN_PROGRAM_ID,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        };

        await program.methods
            .deny()
            .accounts(acceptAccounts)
            .signers([seller])
            .rpc(confirmOptions)
            .catch(console.error);

        const buyerVaultInfo = await program.provider.connection.getAccountInfo(buyerVault[0]);
        const decodedBuyerATA = AccountLayout.decode(buyerVaultInfo.data);
        assert.equal(Number(decodedBuyerATA.amount), buyerVault[1]);
        
        const escrowVaultInfo = await program.provider.connection.getAccountInfo(escrowVaultPubkey);
        const escrowInfo = await program.provider.connection.getAccountInfo(escrowPubkey);

        if (escrowVaultInfo || escrowInfo) {
            throw new Error('Escrow and escrow vault should have been closed')
        } else {
            console.log('Escrow and escrow vault are closed after withdrawal')
        }
    });
})

async function awaitUntilTimestamp(targetTimestamp: number) {
    // all time variables in seconds
    const currentTimestamp = Date.now() / 1000;
    const tolerance = 5;
    const timeRemaining = targetTimestamp + tolerance - currentTimestamp;
  
    if (timeRemaining > 0) {
      console.log(`Waiting for ${timeRemaining} seconds...`);
      await delay(timeRemaining * 1000);
      console.log("Wait complete.");
    } else {
      console.log("No need to wait, the target timestamp has already passed.");
    }
}