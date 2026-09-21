import assert from "node:assert/strict";
import { test } from "node:test";
import { certificate, acquirePubKey } from "../src/lib/api.ts";

test("public key retrieval uses the existing POST endpoint and text response", async (t) => {
  t.mock.method(globalThis, "fetch", async (path: string, options: RequestInit) => {
    assert.equal(path, "/api/pubkey");
    assert.equal(options.method, "POST");
    assert.equal(options.body, undefined);
    return new Response("public-key");
  });
  assert.equal(await acquirePubKey(), "public-key");
});

test("certificate request sends only the blinded token and public key", async (t) => {
  t.mock.method(globalThis, "fetch", async (path: string, options: RequestInit) => {
    assert.equal(path, "/api/certificate");
    assert.equal(options.method, "POST");
    assert.deepEqual(options.headers, { "Content-Type": "application/json" });
    assert.deepEqual(JSON.parse(String(options.body)), { blind_token: "blinded-token", pub_key: "public-key" });
    return new Response("blind-signature");
  });
  assert.equal(await certificate({ blind_token: "blinded-token", pub_key: "public-key" }), "blind-signature");
});

test("a failed response is reported instead of being used as a signature", async (t) => {
  t.mock.method(globalThis, "fetch", async () => new Response("", { status: 400 }));
  await assert.rejects(certificate({ blind_token: "bad-token", pub_key: "public-key" }), /400.*\/api\/certificate/);
});
