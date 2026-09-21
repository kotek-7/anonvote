import init, { blind, finalize, generate_token, verify } from "../../wasm/wasm.js";

export interface BlindingResult {
  blind_token: string;
  secret: string;
}

let initialization: ReturnType<typeof init> | undefined;

export function initializeCrypto() {
  initialization ??= init().catch((error: unknown) => {
    initialization = undefined;
    throw error;
  });
  return initialization;
}

export async function generateToken(): Promise<string> {
  await initializeCrypto();
  return generate_token();
}

export async function blindToken(token: string, publicKey: string): Promise<BlindingResult> {
  await initializeCrypto();
  const result: unknown = blind(token, publicKey);
  if (
    typeof result !== "object" ||
    result === null ||
    !("blind_token" in result) ||
    typeof result.blind_token !== "string" ||
    !("secret" in result) ||
    typeof result.secret !== "string"
  ) {
    throw new Error("Wasm から不正なブラインド化結果が返されました。");
  }
  return { blind_token: result.blind_token, secret: result.secret };
}

export async function finalizeToken(
  publicKey: string,
  blindSignature: string,
  blinding: BlindingResult,
  token: string,
): Promise<string> {
  await initializeCrypto();
  return finalize(publicKey, blindSignature, blinding, token);
}

export async function verifyToken(publicKey: string, signature: string, token: string): Promise<boolean> {
  await initializeCrypto();
  return verify(publicKey, signature, token);
}
