//@ts-check
import init, { blind, finalize, generate_token, verify } from "./pkg/client.js";

await init();

const acquireButton = document.getElementById("acquire-pubkey");
const publicKeyOutput = document.getElementById("pubkey");

let pubKey = "";

acquireButton?.addEventListener("click", async (event) => {
  event.preventDefault();

  const response = await fetch("/api/pubkey", { method: "POST" });
  if (!response.ok) {
    console.error("Failed to acquire public key:", response.statusText);
    return;
  }
  pubKey = await response.text();
  if (publicKeyOutput instanceof HTMLSpanElement) {
    publicKeyOutput.textContent = pubKey;
  }
});

const generateButton = document.getElementById("generate");
const tokenOutput = document.getElementById("token");

let token = "";

generateButton?.addEventListener("click", async (event) => {
  event.preventDefault();

  if (tokenOutput instanceof HTMLSpanElement) {
    token = generate_token();
    tokenOutput.textContent = token;
  }
});

const blindButton = document.getElementById("blind");
const blindTokenOutput = document.getElementById("blind-token");
const secretOutput = document.getElementById("secret");

let blindToken = "";
let secret = "";

blindButton?.addEventListener("click", async (event) => {
  event.preventDefault();

  /** @type {{blind_token: string, secret: string }} */
  const blindingResult = blind(token, pubKey);

  blindToken = blindingResult.blind_token;
  if (blindTokenOutput instanceof HTMLSpanElement) {
    blindTokenOutput.textContent = blindToken;
  }
  secret = blindingResult.secret;
  if (secretOutput instanceof HTMLSpanElement) {
    secretOutput.textContent = secret;
  }
});

const certificateButton = document.getElementById("certificate");
const blindTokenSignOutput = document.getElementById("blind-token-sign");

let blindTokenSign = "";

certificateButton?.addEventListener("click", async (event) => {
  event.preventDefault();

  const response = await fetch("/api/certificate", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      blind_token: blindToken,
      pub_key: pubKey,
    }),
  });

  if (!response.ok) {
    console.error("Failed to acquire public key:", response.statusText);
    return;
  }

  blindTokenSign = await response.text();
  if (blindTokenSignOutput instanceof HTMLSpanElement) {
    blindTokenSignOutput.textContent = blindTokenSign;
  }
});

const finalizeButton = document.getElementById("finalize");
const tokenSignOutput = document.getElementById("token-sign");

let tokenSign = "";

finalizeButton?.addEventListener("click", async (event) => {
  event.preventDefault();

  const blindingResult = {
    blind_token: blindToken,
    secret: secret,
  };

  tokenSign = finalize(pubKey, blindTokenSign, blindingResult, token);
  if (tokenSignOutput instanceof HTMLSpanElement) {
    tokenSignOutput.textContent = tokenSign;
  }
});

const verifyButton = document.getElementById("verify");
const verificationOutput = document.getElementById("verification");

let verification = "";

verifyButton?.addEventListener("click", async (event) => {
  event.preventDefault();

  verification = verify(pubKey, tokenSign, token) ? "ok" : "failed";
  if (verificationOutput instanceof HTMLSpanElement) {
    verificationOutput.textContent = verification;
  }
})