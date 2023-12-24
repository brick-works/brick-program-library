import { AnchorProvider, web3 } from "@coral-xyz/anchor";
import { ConfirmOptions } from "@solana/web3.js";

export const createFundedWallet = async (
  provider: AnchorProvider,
  amount: number,
  confirmOptions: ConfirmOptions,
): Promise<web3.Keypair> => {
  const user = new web3.Keypair();

  await provider.sendAndConfirm(
    new web3.Transaction().add(
      web3.SystemProgram.transfer({
        fromPubkey: provider.wallet.publicKey,
        toPubkey: user.publicKey,
        lamports: amount * web3.LAMPORTS_PER_SOL,
      })
    ),
    null,
    confirmOptions
  );

  return user;
};
