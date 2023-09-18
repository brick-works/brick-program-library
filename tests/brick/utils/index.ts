export * from "./createFundedWallet";
export * from "./createMint";
export * from "./createFundedAssociatedTokenAccount";

export function delay(ms: number) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}


export function getSplitId(str: string): [Buffer, Buffer]{
  const bytes = new TextEncoder().encode(str);

  const data = new Uint8Array(64);
  data.fill(32);
  data.set(bytes);

  const firstId = Buffer.from(data.slice(0, 32));
  const secondId = Buffer.from(data.slice(32));

  return [firstId, secondId];
}
