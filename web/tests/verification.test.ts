import assert from "node:assert/strict";
import { test } from "node:test";
import { initialState, verificationReducer } from "../src/verification.ts";
import type { Action, VerificationState } from "../src/verification.ts";

function verifiedState(): VerificationState {
  const actions: Action[] = [
    { type: "ready" },
    { type: "publicKey", value: "public-key" },
    { type: "token", value: "original-token" },
    { type: "blinding", value: { blind_token: "blinded-token", secret: "secret" } },
    { type: "blindSignature", value: "blind-signature" },
    { type: "signature", value: "signature" },
    { type: "verification", value: true },
  ];
  return actions.reduce(verificationReducer, initialState);
}

test("a new token cannot reuse the previous blinding, signature, or verification", () => {
  const next = verificationReducer(verifiedState(), { type: "token", value: "new-token" });
  assert.equal(next.publicKey, "public-key");
  assert.equal(next.token, "new-token");
  assert.equal(next.blinding, null);
  assert.equal(next.blindSignature, null);
  assert.equal(next.signature, null);
  assert.equal(next.verification, null);
});

test("changing the public key keeps the independent token and invalidates its proof", () => {
  const next = verificationReducer(verifiedState(), { type: "publicKey", value: "new-key" });
  assert.equal(next.token, "original-token");
  assert.equal(next.publicKey, "new-key");
  assert.equal(next.blinding, null);
  assert.equal(next.signature, null);
  assert.equal(next.verification, null);
});

test("reissuing a certificate requires finalization and verification again", () => {
  const previous = verifiedState();
  const next = verificationReducer(previous, { type: "blindSignature", value: "new-certificate" });
  assert.equal(next.blinding, previous.blinding);
  assert.equal(next.blindSignature, "new-certificate");
  assert.equal(next.signature, null);
  assert.equal(next.verification, null);
});

test("an API failure releases the pending operation and preserves input for retry", () => {
  const prepared = verificationReducer(verifiedState(), {
    type: "blinding",
    value: { blind_token: "retry-token", secret: "retry-secret" },
  });
  const pending = verificationReducer(prepared, { type: "start", operation: "blindSignature" });
  const failed = verificationReducer(pending, { type: "error", message: "HTTP 500" });
  assert.equal(failed.pending, null);
  assert.equal(failed.blinding, prepared.blinding);
  assert.equal(failed.blindSignature, null);
  assert.equal(failed.error, "HTTP 500");
  const retried = verificationReducer(failed, { type: "start", operation: "blindSignature" });
  assert.equal(retried.error, null);
  assert.equal(retried.pending, "blindSignature");
});

test("reset removes all intermediate data while keeping Wasm ready", () => {
  assert.deepEqual(verificationReducer(verifiedState(), { type: "reset" }), { ...initialState, ready: true });
});
