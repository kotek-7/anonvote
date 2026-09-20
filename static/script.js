//@ts-check
import init, { blind, generate_token } from "./pkg/client.js";

await init();
const acquireButton = document.getElementById("acquire-pubkey");
const generateButton = document.getElementById("generate");
const publicKeyOutput = document.getElementById("pubkey");
const tokenOutput = document.getElementById("token");
const blindedTokenOutput = document.getElementById("blinded-token");

acquireButton?.addEventListener("click", async (event) => {
  event.preventDefault();
  const response = await fetch("/pubkey");
  if (!response.ok) {
    console.error("Failed to acquire public key:", response.statusText);
    return;
  }
  const pubKey = await response.text();
  if (publicKeyOutput instanceof HTMLSpanElement) {
    publicKeyOutput.textContent = pubKey;
  }
});

generateButton?.addEventListener("click", async (event) => {
  event.preventDefault();

  const pubKey = `-----BEGIN PUBLIC KEY-----
MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAtSw86N20TdzZZdMFJ5Yi
XpLEu4drQMEHNb9TL8klsK1qZ5nSiNFgDeQy/3OpaOuFasx3eykstHEPALNp++rv
zxZFs8zbUR27ulM4GErqVUOnqvqPTpI8YqNiiAxmQ9T+YtUNVPHMV3sE1Dzxxa63
djXLZbzsT9PDWbnq/ruKGJlHi0x73YKq/iDvssEXO9e59Xrpz9swyH+jbeaaZAiZ
/5wlofJZfzLqA3rVZO363z29RlVPtW+pg2bjJs0Q7Idx5VBQqVr9RKTwtkI9SQOB
7+9pNJsSXrwzGKQrLcUjIsCBQaZk2VerDguNOwoXeeY0X4hYR0MpPlz/0jnQsQ3q
jwIDAQAB
-----END PUBLIC KEY-----`;
  const token = generate_token();
  const blindedToken = blind(token, pubKey);
  if (blindedTokenOutput instanceof HTMLSpanElement) {
    blindedTokenOutput.textContent = blindedToken;
  }
  if (publicKeyOutput instanceof HTMLSpanElement) {
    publicKeyOutput.textContent = pubKey;
  }
  if (tokenOutput instanceof HTMLSpanElement) {
    tokenOutput.textContent = token;
  }
});
