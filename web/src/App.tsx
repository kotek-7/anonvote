import { useEffect, useState } from "react";
import { Output } from "./components/Output";
import { Step } from "./components/Step";
import * as api from "./lib/api";
import * as crypto from "./lib/crypto";
import type { BlindingResult } from "./lib/crypto";
import "./App.css";

function App() {
  const [ready, setReady] = useState(false);
  const [pending, setPending] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const [publicKey, setPublicKey] = useState<string | null>(null);
  const [token, setToken] = useState<string | null>(null);
  const [blinding, setBlinding] = useState<BlindingResult | null>(null);
  const [blindSignature, setBlindSignature] = useState<string | null>(null);
  const [signature, setSignature] = useState<string | null>(null);
  const [verification, setVerification] = useState<boolean | null>(null);

  const disabled = !ready || pending !== null;

  useEffect(() => {
    let active = true;
    crypto.initializeCrypto().then(
      () => {
        if (active) setReady(true);
      },
      (error: unknown) => {
        if (active) {
          const message = error instanceof Error ? error.message : String(error);
          setError(`Wasm の読み込みに失敗しました。ページを再読み込みしてください。 ${message}`);
        }
      },
    );
    return () => {
      active = false;
    };
  }, []);

  async function handleAcquirePublicKey() {
    setPending("publicKey");
    setError(null);
    try {
      const key = await api.acquirePublicKey();
      setPublicKey(key);
      // トークンは公開鍵に依存しない。ブラインド化以降だけやり直す。
      setBlinding(null);
      setBlindSignature(null);
      setSignature(null);
      setVerification(null);
    } catch (error) {
      setError(error instanceof Error ? error.message : String(error));
    } finally {
      setPending(null);
    }
  }

  async function handleGenerateToken() {
    setPending("token");
    setError(null);
    try {
      const newToken = await crypto.generateToken();
      setToken(newToken);
      // 新しいトークンには、以前のブラインド化結果や署名を使えない。
      setBlinding(null);
      setBlindSignature(null);
      setSignature(null);
      setVerification(null);
    } catch (error) {
      setError(error instanceof Error ? error.message : String(error));
    } finally {
      setPending(null);
    }
  }

  async function handleBlindToken() {
    if (!publicKey || !token) return;

    setPending("blinding");
    setError(null);
    try {
      const result = await crypto.blindToken(token, publicKey);
      setBlinding(result);
      setBlindSignature(null);
      setSignature(null);
      setVerification(null);
    } catch (error) {
      setError(error instanceof Error ? error.message : String(error));
    } finally {
      setPending(null);
    }
  }

  async function handleAcquireCertificate() {
    if (!publicKey || !blinding) return;

    setPending("blindSignature");
    setError(null);
    try {
      const certificate = await api.acquireCertificate({
        pub_key: publicKey,
        blind_token: blinding.blind_token,
      });
      setBlindSignature(certificate);
      setSignature(null);
      setVerification(null);
    } catch (error) {
      setError(error instanceof Error ? error.message : String(error));
    } finally {
      setPending(null);
    }
  }

  async function handleFinalizeToken() {
    if (!publicKey || !token || !blinding || !blindSignature) return;

    setPending("signature");
    setError(null);
    try {
      const result = await crypto.finalizeToken(publicKey, blindSignature, blinding, token);
      setSignature(result);
      setVerification(null);
    } catch (error) {
      setError(error instanceof Error ? error.message : String(error));
    } finally {
      setPending(null);
    }
  }

  async function handleVerifyToken() {
    if (!publicKey || !token || !signature) return;

    setPending("verification");
    setError(null);
    try {
      const valid = await crypto.verifyToken(publicKey, signature, token);
      setVerification(valid);
    } catch (error) {
      setError(error instanceof Error ? error.message : String(error));
    } finally {
      setPending(null);
    }
  }

  function handleReset() {
    setPublicKey(null);
    setToken(null);
    setBlinding(null);
    setBlindSignature(null);
    setSignature(null);
    setVerification(null);
    setError(null);
  }

  return (
    <main>
      <header>
        <p className="eyebrow">ANONVOTE</p>
        <h1>ブラインド署名の検証</h1>
        <p>トークンを生成し、サーバーの署名を取得して、ブラウザで検証します。</p>
        <div className="toolbar">
          <p role="status">{ready ? "準備完了" : error ? "初期化に失敗しました" : "Wasm を読み込んでいます…"}</p>
          <button type="button" className="secondary" onClick={handleReset} disabled={disabled}>
            最初からやり直す
          </button>
        </div>
      </header>
      {error && (
        <p className="error" role="alert">
          {error}
        </p>
      )}
      <div className="steps">
        <Step number={1} title="公開鍵" location="サーバー">
          <button
            type="button"
            disabled={disabled}
            onClick={handleAcquirePublicKey}
            aria-busy={pending === "publicKey"}
          >
            {pending === "publicKey" ? "処理中…" : "公開鍵を取得"}
          </button>
          <Output label="Public Key" value={publicKey} />
        </Step>
        <Step number={2} title="トークン" location="ブラウザ">
          <button type="button" disabled={disabled} onClick={handleGenerateToken} aria-busy={pending === "token"}>
            {pending === "token" ? "処理中…" : "トークンを生成"}
          </button>
          <Output label="Token" value={token} />
        </Step>
        <Step number={3} title="ブラインド化" location="ブラウザ">
          <button
            type="button"
            disabled={disabled || !publicKey || !token}
            onClick={handleBlindToken}
            aria-busy={pending === "blinding"}
          >
            {pending === "blinding" ? "処理中…" : "トークンをブラインド化"}
          </button>
          <Output label="Blind Token" value={blinding?.blind_token} />
          <Output label="Secret" value={blinding?.secret} />
        </Step>
        <Step number={4} title="署名の取得" location="サーバー">
          <button
            type="button"
            disabled={disabled || !publicKey || !blinding}
            onClick={handleAcquireCertificate}
            aria-busy={pending === "blindSignature"}
          >
            {pending === "blindSignature" ? "処理中…" : "署名を取得"}
          </button>
          <Output label="Blind Token Signature" value={blindSignature} />
        </Step>
        <Step number={5} title="署名の復元" location="ブラウザ">
          <button
            type="button"
            disabled={disabled || !publicKey || !token || !blinding || !blindSignature}
            onClick={handleFinalizeToken}
            aria-busy={pending === "signature"}
          >
            {pending === "signature" ? "処理中…" : "署名を復元"}
          </button>
          <Output label="Token Signature" value={signature} />
        </Step>
        <Step number={6} title="署名の検証" location="ブラウザ">
          <button
            type="button"
            disabled={disabled || !publicKey || !token || !signature}
            onClick={handleVerifyToken}
            aria-busy={pending === "verification"}
          >
            {pending === "verification" ? "処理中…" : "署名を検証"}
          </button>
          <p
            role="status"
            className={verification === null ? "result" : verification ? "result success" : "result error"}
          >
            {verification === null ? "未検証" : verification ? "検証成功" : "検証失敗"}
          </p>
        </Step>
      </div>
    </main>
  );
}

export default App;
