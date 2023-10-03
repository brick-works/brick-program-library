import { PROGRAM_ID as METADATA_PROGRAM } from "@metaplex-foundation/mpl-token-metadata";
import { Tender } from "../../target/types/tender";
import * as anchor from "@coral-xyz/anchor";
import { v4 as uuid, parse } from "uuid";
import {
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  getAssociatedTokenAddressSync,
} from "@solana/spl-token";
import { 
    ComputeBudgetProgram,
    GetTransactionConfig, 
    SYSVAR_RENT_PUBKEY, 
    SystemProgram, 
} from "@solana/web3.js";
import { 
    createFundedAssociatedTokenAccount, 
    createFundedWallet, 
    createMint,
    delay,
} from "../utils";

describe("tender", () => {
    const provider = anchor.AnchorProvider.env();
    anchor.setProvider(provider);
    const program = anchor.workspace.Tender as anchor.Program<Tender>;
    const confirmOptions: GetTransactionConfig = { commitment: "confirmed" };

    let networkCreator: anchor.web3.Keypair;
    let proposalCreator: anchor.web3.Keypair;
    let depositor: anchor.web3.Keypair;

    let network: anchor.web3.PublicKey;
    let networkMint: anchor.web3.PublicKey;
    let councilCollection: anchor.web3.PublicKey;
    let councilCollectionVault: anchor.web3.PublicKey;
    let councilCollectionMetadata: anchor.web3.PublicKey;
    let councilCollectionMasterEdition: anchor.web3.PublicKey;
    let serviceCollection: anchor.web3.PublicKey;
    let serviceCollectionVault: anchor.web3.PublicKey;
    let serviceCollectionMetadata: anchor.web3.PublicKey;
    let serviceCollectionMasterEdition: anchor.web3.PublicKey;
    let proposalCollection: anchor.web3.PublicKey;
    let proposalCollectionVault: anchor.web3.PublicKey;
    let proposalCollectionMetadata: anchor.web3.PublicKey;
    let proposalCollectionMasterEdition: anchor.web3.PublicKey;

    let proposal: anchor.web3.PublicKey;
    let proposalVault: anchor.web3.PublicKey;
    let proposalMint: anchor.web3.PublicKey;
    let proposalMetadata: anchor.web3.PublicKey;
    let proposalMasterEdition: anchor.web3.PublicKey;
    let userVault: anchor.web3.PublicKey;
    let paymentMint: anchor.web3.PublicKey;

    let serviceCollectionUri: string = 'https://shdw-drive.genesysgo.net/E8yeCMMgZCimwwh16LFMbWF4VDfQSzGWLcbXqFnrXMT6/DNzw56KHzDVqmfeKGSG8xut9JDTAUBZD64idjo2G9oJf.json?ts=1689256340';
    let councilCollectionUri: string = 'https://shdw-drive.genesysgo.net/E8yeCMMgZCimwwh16LFMbWF4VDfQSzGWLcbXqFnrXMT6/7QFMZM7hwxgQTAkfpbKyMxZ57ZRBpNpfPddbEdzoYxzx.json?ts=1689254939';
    let proposalCollectionUri: string = 'https://shdw-drive.genesysgo.net/E8yeCMMgZCimwwh16LFMbWF4VDfQSzGWLcbXqFnrXMT6/CCdeiLHULvrHsP1qzKbXEkrM7duvBGUgQ4uy8fRCcwK7.json?ts=1689256192';
    let proposalUri: string = 'https://shdw-drive.genesysgo.net/E8yeCMMgZCimwwh16LFMbWF4VDfQSzGWLcbXqFnrXMT6/CCdeiLHULvrHsP1qzKbXEkrM7duvBGUgQ4uy8fRCcwK7.json?ts=1689256192';

    let depositorVault: [anchor.web3.PublicKey, number];

    it("Should create a network", async () => {
        networkCreator = await createFundedWallet(provider, 10000);

        [network] = anchor.web3.PublicKey.findProgramAddressSync(
            [Buffer.from("network", "utf-8")],
            program.programId
        );
        [networkMint] = anchor.web3.PublicKey.findProgramAddressSync(
            [
                Buffer.from("network_mint", "utf-8"),
                network.toBuffer(),
            ],
            program.programId
        );
        [councilCollection] = anchor.web3.PublicKey.findProgramAddressSync(
            [
                Buffer.from("council", "utf-8"),
                network.toBuffer(),
            ],
            program.programId
        );
        councilCollectionVault = getAssociatedTokenAddressSync(
            councilCollection,
            network,
            true,
        );
        [councilCollectionMetadata] = anchor.web3.PublicKey.findProgramAddressSync(
            [
              Buffer.from("metadata", "utf-8"), 
              METADATA_PROGRAM.toBuffer(),
              councilCollection.toBuffer()
            ],
            METADATA_PROGRAM
        );
        [councilCollectionMasterEdition] = anchor.web3.PublicKey.findProgramAddressSync(
            [
              Buffer.from("metadata", "utf-8"), 
              METADATA_PROGRAM.toBuffer(),
              councilCollection.toBuffer(),
              Buffer.from("edition", "utf-8"), 
            ],
            METADATA_PROGRAM
        );
        [serviceCollection] = anchor.web3.PublicKey.findProgramAddressSync(
            [
                Buffer.from("service", "utf-8"),
                network.toBuffer(),
            ],
            program.programId
        );
        serviceCollectionVault = getAssociatedTokenAddressSync(
            serviceCollection,
            network,
            true,
        );
        [serviceCollectionMetadata] = anchor.web3.PublicKey.findProgramAddressSync(
            [
              Buffer.from("metadata", "utf-8"), 
              METADATA_PROGRAM.toBuffer(),
              serviceCollection.toBuffer()
            ],
            METADATA_PROGRAM
        );
        [serviceCollectionMasterEdition] = anchor.web3.PublicKey.findProgramAddressSync(
            [
              Buffer.from("metadata", "utf-8"), 
              METADATA_PROGRAM.toBuffer(),
              serviceCollection.toBuffer(),
              Buffer.from("edition", "utf-8"), 
            ],
            METADATA_PROGRAM
        );
        [proposalCollection] = anchor.web3.PublicKey.findProgramAddressSync(
            [
                Buffer.from("proposal", "utf-8"),
                network.toBuffer(),
            ],
            program.programId
        );
        proposalCollectionVault = getAssociatedTokenAddressSync(
            proposalCollection,
            network,
            true,
        );
        [proposalCollectionMetadata] = anchor.web3.PublicKey.findProgramAddressSync(
            [
              Buffer.from("metadata", "utf-8"), 
              METADATA_PROGRAM.toBuffer(),
              proposalCollection.toBuffer()
            ],
            METADATA_PROGRAM
        );
        [proposalCollectionMasterEdition] = anchor.web3.PublicKey.findProgramAddressSync(
            [
              Buffer.from("metadata", "utf-8"), 
              METADATA_PROGRAM.toBuffer(),
              proposalCollection.toBuffer(),
              Buffer.from("edition", "utf-8"), 
            ],
            METADATA_PROGRAM
        );

        const networkAccounts = {
            signer: networkCreator.publicKey,
            network,
            networkMint,
            councilCollection,
            serviceCollection,
            proposalCollection,
            proposalCollectionVault,
            proposalMetadata: proposalCollectionMetadata,
            proposalCollectionMasterEdition,        
            rent: SYSVAR_RENT_PUBKEY,
            SystemProgram: SystemProgram.programId,
            tokenProgram: TOKEN_PROGRAM_ID,
            associatedTokenProgram:ASSOCIATED_TOKEN_PROGRAM_ID,
            tokenMetadataProgram: METADATA_PROGRAM,
        }

        await program.methods
            .initNetwork(proposalCollectionUri)
            .accounts(networkAccounts)
            .preInstructions([ComputeBudgetProgram.setComputeUnitLimit({ units: 250000 })])
            .signers([networkCreator])
            .rpc(confirmOptions)
            .catch(console.error);

        const rolesAccounts = {
            signer: networkCreator.publicKey,
            network,
            networkMint,
            councilCollection,
            councilCollectionVault,
            councilCollectionMetadata,
            councilCollectionMasterEdition,
            serviceCollection,
            serviceCollectionVault,
            serviceCollectionMetadata,
            serviceCollectionMasterEdition,
            rent: SYSVAR_RENT_PUBKEY,
            SystemProgram: SystemProgram.programId,
            tokenProgram: TOKEN_PROGRAM_ID,
            associatedTokenProgram:ASSOCIATED_TOKEN_PROGRAM_ID,
            tokenMetadataProgram: METADATA_PROGRAM,
        }

        const rolesParams = {
            serviceCollectionUri,
            councilCollectionUri,
        }

        await program.methods
            .initRoles(rolesParams)
            .accounts(rolesAccounts)
            .preInstructions([ComputeBudgetProgram.setComputeUnitLimit({ units: 350000 })])
            .signers([networkCreator])
            .rpc(confirmOptions)
            .catch(console.error);
    })

    it("Should create a proposal", async () => {
        proposalCreator = await createFundedWallet(provider, 10000);
        paymentMint = await createMint(provider, confirmOptions);
        const proposalId = parse(uuid());
        [proposal] = anchor.web3.PublicKey.findProgramAddressSync(
            [
                Buffer.from("proposal", "utf-8"),
                proposalCreator.publicKey.toBuffer(),
                proposalId,
            ],
            program.programId
        );
        [proposalVault] = anchor.web3.PublicKey.findProgramAddressSync(
            [
                Buffer.from("vault", "utf-8"),
                proposal.toBuffer(),
            ],
            program.programId
        );
        [proposalMint] = anchor.web3.PublicKey.findProgramAddressSync(
            [
                Buffer.from("proposal_mint", "utf-8"), 
                proposal.toBuffer(),
            ],
            program.programId
        );
        [proposalMetadata] = anchor.web3.PublicKey.findProgramAddressSync(
            [
                Buffer.from("metadata", "utf-8"), 
                METADATA_PROGRAM.toBuffer(),
                proposalMint.toBuffer(),
            ],
            METADATA_PROGRAM
        );
        [proposalMasterEdition] = anchor.web3.PublicKey.findProgramAddressSync(
            [
                Buffer.from("metadata", "utf-8"), 
                METADATA_PROGRAM.toBuffer(),
                proposalMint.toBuffer(),
                Buffer.from("edition", "utf-8"), 
            ],
            METADATA_PROGRAM
        );
        userVault = getAssociatedTokenAddressSync(
            proposalMint,
            proposalCreator.publicKey,
            true,
        );
        const proposalAccounts = {
            signer: proposalCreator.publicKey,
            networkOrigin: networkCreator.publicKey,
            network,
            proposal,
            vault: proposalVault,
            paymentMint,
            proposalCollection,
            proposalCollectionMetadata,
            proposalCollectionMasterEdition,
            proposalMint,
            proposalMetadata,
            proposalMasterEdition,
            userVault,
            rent: SYSVAR_RENT_PUBKEY,
            systemProgram: SystemProgram.programId,
            tokenProgram: TOKEN_PROGRAM_ID,
            associatedTokenProgram:ASSOCIATED_TOKEN_PROGRAM_ID,
            tokenMetadataProgram: METADATA_PROGRAM,
        };
        const proposalParams = {
            id: [...proposalId],
            name: "Build a football pitch",
            description: "Proposal to build a football pitch for community benefit",
            proposalUri,
        };
        await program.methods
            .initProposal(proposalParams)
            .accounts(proposalAccounts)
            .preInstructions([ComputeBudgetProgram.setComputeUnitLimit({ units: 300000 })])
            .signers([proposalCreator])
            .rpc(confirmOptions)
            .catch(console.error);
    })

    it("Should do a deposit", async () => {
        depositor = await createFundedWallet(provider, 10000);
        depositorVault = [
            await createFundedAssociatedTokenAccount(
              provider,
              paymentMint,
              10,
              depositor
            ),
            10
        ];
        const receiverVault = getAssociatedTokenAddressSync(networkMint, depositor.publicKey, false);
        const depositAccounts = {
            signer: depositor.publicKey,
            network,
            proposal,
            proposalVault,
            depositVault: depositorVault[0],
            receiverVault,
            paymentMint,
            networkMint,
            tokenProgram: TOKEN_PROGRAM_ID,
        };

        await program.methods
            .deposit(new anchor.BN(10))
            .accounts(depositAccounts)
            .signers([depositor])
            .rpc(confirmOptions)
            .catch(console.error);
    })
})