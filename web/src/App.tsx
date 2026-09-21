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

  const [pubKey, setPubKey] = useState<string | null>(null);
  const [token, setToken] = useState<string | null>(null);
  const [blindingResult, setBlindingResult] = useState<BlindingResult | null>(null);
  const [blindTokenSign, setBlindTokenSign] = useState<string | null>(null);
  const [tokenSign, setTokenSign] = useState<string | null>(null);
  const [verification, setVerification] = useState<boolean | null>(null);

  const disabled = !ready || pending !== null;

  useEffect(() => {
    let active = true;
    crypto.init().then(
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

  async function handleAcquire() {
    setPending("acquire-pubkey");
    setError(null);
    try {
      const pubKey = await api.acquirePubKey();
      setPubKey(pubKey);
      // トークンは公開鍵に依存しない。ブラインド化以降だけやり直す。
      setBlindingResult(null);
      setBlindTokenSign(null);
      setTokenSign(null);
      setVerification(null);
    } catch (error) {
      setError(error instanceof Error ? error.message : String(error));
    } finally {
      setPending(null);
    }
  }

  async function handleGenerate() {
    setPending("generate");
    setError(null);
    try {
      const newToken = await crypto.generate_token();
      setToken(newToken);
      // 新しいトークンには、以前のブラインド化結果や署名を使えない。
      setBlindingResult(null);
      setBlindTokenSign(null);
      setTokenSign(null);
      setVerification(null);
    } catch (error) {
      setError(error instanceof Error ? error.message : String(error));
    } finally {
      setPending(null);
    }
  }

  async function handleBlind() {
    if (!pubKey || !token) return;

    setPending("blind");
    setError(null);
    try {
      const blindingResult = await crypto.blind(token, pubKey);
      setBlindingResult(blindingResult);
      setBlindTokenSign(null);
      setTokenSign(null);
      setVerification(null);
    } catch (error) {
      setError(error instanceof Error ? error.message : String(error));
    } finally {
      setPending(null);
    }
  }

  async function handleCertificate() {
    if (!pubKey || !blindingResult) return;

    setPending("certificate");
    setError(null);
    try {
      const blindTokenSign = await api.certificate({
        pub_key: pubKey,
        blind_token: blindingResult.blind_token,
      });
      setBlindTokenSign(blindTokenSign);
      setTokenSign(null);
      setVerification(null);
    } catch (error) {
      setError(error instanceof Error ? error.message : String(error));
    } finally {
      setPending(null);
    }
  }

  async function handleFinalize() {
    if (!pubKey || !token || !blindingResult || !blindTokenSign) return;

    setPending("finalize");
    setError(null);
    try {
      const tokenSign = await crypto.finalize(pubKey, blindTokenSign, blindingResult, token);
      setTokenSign(tokenSign);
      setVerification(null);
    } catch (error) {
      setError(error instanceof Error ? error.message : String(error));
    } finally {
      setPending(null);
    }
  }

  async function handleVerify() {
    if (!pubKey || !token || !tokenSign) return;

    setPending("verify");
    setError(null);
    try {
      const verification = await crypto.verify(pubKey, tokenSign, token);
      setVerification(verification);
    } catch (error) {
      setError(error instanceof Error ? error.message : String(error));
    } finally {
      setPending(null);
    }
  }

  function handleReset() {
    setPubKey(null);
    setToken(null);
    setBlindingResult(null);
    setBlindTokenSign(null);
    setTokenSign(null);
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
            id="acquire-pubkey"
            onClick={handleAcquire}
            aria-busy={pending === "acquire-pubkey"}
          >
            {pending === "acquire-pubkey" ? "処理中…" : "公開鍵を取得"}
          </button>
          <Output id="pubkey" label="Public Key" value={pubKey} />
        </Step>
        <Step number={2} title="トークン" location="ブラウザ">
          <button
            type="button"
            disabled={disabled}
            id="generate"
            onClick={handleGenerate}
            aria-busy={pending === "generate"}
          >
            {pending === "generate" ? "処理中…" : "トークンを生成"}
          </button>
          <Output id="token" label="Token" value={token} />
        </Step>
        <Step number={3} title="ブラインド化" location="ブラウザ">
          <button
            type="button"
            disabled={disabled || !pubKey || !token}
            id="blind"
            onClick={handleBlind}
            aria-busy={pending === "blind"}
          >
            {pending === "blind" ? "処理中…" : "トークンをブラインド化"}
          </button>
          <Output id="blind-token" label="Blind Token" value={blindingResult?.blind_token} />
          <Output id="secret" label="Secret" value={blindingResult?.secret} />
        </Step>
        <Step number={4} title="署名の取得" location="サーバー">
          <button
            type="button"
            disabled={disabled || !pubKey || !blindingResult}
            id="certificate"
            onClick={handleCertificate}
            aria-busy={pending === "certificate"}
          >
            {pending === "certificate" ? "処理中…" : "署名を取得"}
          </button>
          <Output id="blind-token-sign" label="Blind Token Sign" value={blindTokenSign} />
        </Step>
        <Step number={5} title="署名の復元" location="ブラウザ">
          <button
            type="button"
            disabled={disabled || !pubKey || !token || !blindingResult || !blindTokenSign}
            id="finalize"
            onClick={handleFinalize}
            aria-busy={pending === "finalize"}
          >
            {pending === "finalize" ? "処理中…" : "署名を復元"}
          </button>
          <Output id="token-sign" label="Token Sign" value={tokenSign} />
        </Step>
        <Step number={6} title="署名の検証" location="ブラウザ">
          <button
            type="button"
            disabled={disabled || !pubKey || !token || !tokenSign}
            id="verify"
            onClick={handleVerify}
            aria-busy={pending === "verify"}
          >
            {pending === "verify" ? "処理中…" : "署名を検証"}
          </button>
          <p
            id="verification"
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
