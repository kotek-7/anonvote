import { useEffect, useReducer, useRef } from "react";
import { acquireCertificate, acquirePublicKey } from "./lib/api";
import { blindToken, finalizeToken, generateToken, initializeCrypto, verifyToken } from "./lib/crypto";
import { initialState, verificationReducer } from "./verification";
import type { Action, Operation } from "./verification";

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

export function useVerification() {
  const [state, dispatch] = useReducer(verificationReducer, initialState);
  const inFlight = useRef(false);

  useEffect(() => {
    let active = true;
    initializeCrypto().then(
      () => {
        if (active) dispatch({ type: "ready" });
      },
      (error: unknown) => {
        if (active)
          dispatch({
            type: "error",
            message: `Wasm の読み込みに失敗しました。ページを再読み込みしてください。 ${errorMessage(error)}`,
          });
      },
    );
    return () => {
      active = false;
    };
  }, []);

  async function perform(operation: Operation, task: () => Promise<Action>) {
    if (!state.ready || inFlight.current) return;
    inFlight.current = true;
    dispatch({ type: "start", operation });
    try {
      dispatch(await task());
    } catch (error) {
      dispatch({ type: "error", message: errorMessage(error) });
    } finally {
      inFlight.current = false;
    }
  }

  return {
    state,
    reset: () => {
      if (!inFlight.current) dispatch({ type: "reset" });
    },
    acquirePublicKey: () => perform("publicKey", async () => ({ type: "publicKey", value: await acquirePublicKey() })),
    generateToken: () => perform("token", async () => ({ type: "token", value: await generateToken() })),
    blind: () =>
      perform("blinding", async () => {
        if (!state.token || !state.publicKey) throw new Error("公開鍵とトークンが必要です。");
        return { type: "blinding", value: await blindToken(state.token, state.publicKey) };
      }),
    acquireCertificate: () =>
      perform("blindSignature", async () => {
        if (!state.blinding || !state.publicKey) throw new Error("ブラインド化が必要です。");
        return {
          type: "blindSignature",
          value: await acquireCertificate({ blind_token: state.blinding.blind_token, pub_key: state.publicKey }),
        };
      }),
    finalize: () =>
      perform("signature", async () => {
        if (!state.publicKey || !state.blindSignature || !state.blinding || !state.token)
          throw new Error("署名の取得が必要です。");
        return {
          type: "signature",
          value: await finalizeToken(state.publicKey, state.blindSignature, state.blinding, state.token),
        };
      }),
    verify: () =>
      perform("verification", async () => {
        if (!state.publicKey || !state.signature || !state.token) throw new Error("署名の復元が必要です。");
        return { type: "verification", value: await verifyToken(state.publicKey, state.signature, state.token) };
      }),
  };
}
