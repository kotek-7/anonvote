import initWasm, * as wasm from "../../wasm/wasm.js";

export interface BlindingResult {
  blind_token: string;
  secret: string;
}

let initialization: ReturnType<typeof initWasm> | undefined;

export function init() {
  initialization ??= initWasm().catch((error: unknown) => {
    initialization = undefined;
    throw error;
  });
  return initialization;
}

export async function generate_token(): Promise<string> {
  await init();
  return wasm.generate_token();
}

export async function blind(token: string, pubKey: string): Promise<BlindingResult> {
  await init();
  const blindingResult: unknown = wasm.blind(token, pubKey);
  if (
    typeof blindingResult !== "object" ||
    blindingResult === null ||
    !("blind_token" in blindingResult) ||
    typeof blindingResult.blind_token !== "string" ||
    !("secret" in blindingResult) ||
    typeof blindingResult.secret !== "string"
  ) {
    throw new Error("Wasm から不正なブラインド化結果が返されました。");
  }
  return { blind_token: blindingResult.blind_token, secret: blindingResult.secret };
}

export async function finalize(
  pubKey: string,
  blindTokenSign: string,
  blindingResult: BlindingResult,
  token: string,
): Promise<string> {
  await init();
  return wasm.finalize(pubKey, blindTokenSign, blindingResult, token);
}

export async function verify(pubKey: string, tokenSign: string, token: string): Promise<boolean> {
  await init();
  return wasm.verify(pubKey, tokenSign, token);
}
