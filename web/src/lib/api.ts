export interface CertificateRequest {
  blind_token: string;
  pub_key: string;
}

export async function acquirePublicKey(): Promise<string> {
  const response = await fetch("/api/pubkey", { method: "POST" });
  if (!response.ok) {
    throw new Error(`公開鍵の取得に失敗しました (${response.status}): /api/pubkey`);
  }
  return response.text();
}

export async function acquireCertificate(body: CertificateRequest): Promise<string> {
  const response = await fetch("/api/certificate", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body),
  });
  if (!response.ok) {
    throw new Error(`署名の取得に失敗しました (${response.status}): /api/certificate`);
  }
  return response.text();
}
