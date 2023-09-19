export * from "./createFundedWallet";
export * from "./createMint";
export * from "./createFundedAssociatedTokenAccount";
import { v4 as uuid } from 'uuid';

export function delay(ms: number) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}