import type { BlindingResult } from "./lib/crypto.ts";

export type Operation = "publicKey" | "token" | "blinding" | "blindSignature" | "signature" | "verification";

export interface VerificationState {
  ready: boolean;
  pending: Operation | null;
  error: string | null;
  publicKey: string | null;
  token: string | null;
  blinding: BlindingResult | null;
  blindSignature: string | null;
  signature: string | null;
  verification: boolean | null;
}

export type Action =
  | { type: "ready" }
  | { type: "start"; operation: Operation }
  | { type: "error"; message: string }
  | { type: "reset" }
  | { type: "publicKey" | "token" | "blindSignature" | "signature"; value: string }
  | { type: "blinding"; value: BlindingResult }
  | { type: "verification"; value: boolean };

const clearedSignatures = { blindSignature: null, signature: null, verification: null };
const clearedBlinding = { blinding: null, ...clearedSignatures };

export const initialState: VerificationState = {
  ready: false,
  pending: null,
  error: null,
  publicKey: null,
  token: null,
  ...clearedBlinding,
};

export function verificationReducer(state: VerificationState, action: Action): VerificationState {
  switch (action.type) {
    case "ready":
      return { ...state, ready: true, error: null };
    case "start":
      return { ...state, pending: action.operation, error: null };
    case "error":
      return { ...state, pending: null, error: action.message };
    case "reset":
      return { ...initialState, ready: state.ready };
    case "publicKey":
    case "token":
      return { ...state, ...clearedBlinding, [action.type]: action.value, pending: null, error: null };
    case "blinding":
      return { ...state, ...clearedSignatures, blinding: action.value, pending: null, error: null };
    case "blindSignature":
      return {
        ...state,
        blindSignature: action.value,
        signature: null,
        verification: null,
        pending: null,
        error: null,
      };
    case "signature":
      return { ...state, signature: action.value, verification: null, pending: null, error: null };
    case "verification":
      return { ...state, verification: action.value, pending: null, error: null };
  }
}
