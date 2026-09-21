export interface CertificateRequest {
  blind_token: string;
  pub_key: string;
}

async function post(path: string, body?: CertificateRequest): Promise<string> {
  const response = await fetch(path, {
    method: "POST",
    ...(body && {
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(body),
    }),
  });
  if (!response.ok) {
    throw new Error(`API の呼び出しに失敗しました (${response.status}): ${path}`);
  }
  return response.text();
}

export function acquirePublicKey(): Promise<string> {
  return post("/api/pubkey");
}

export function acquireCertificate(body: CertificateRequest): Promise<string> {
  return post("/api/certificate", body);
}
