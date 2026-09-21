import type { ReactNode } from "react";
import { useVerification } from "./useVerification";
import "./App.css";

function Output({ label, value }: { label: string; value: string | null | undefined }) {
  return (
    <div className="output">
      <span>{label}</span>
      <pre>{value ?? "未生成"}</pre>
    </div>
  );
}

function Step({
  number,
  title,
  location,
  button,
  disabled,
  busy,
  onClick,
  children,
}: {
  number: number;
  title: string;
  location: string;
  button: string;
  disabled: boolean;
  busy: boolean;
  onClick: () => Promise<void>;
  children: ReactNode;
}) {
  return (
    <section className="step" aria-labelledby={`step-${number}`}>
      <div className="step-heading">
        <span className="step-number">{number}</span>
        <h2 id={`step-${number}`}>{title}</h2>
        <span className="location">{location}</span>
      </div>
      <button type="button" disabled={disabled} onClick={onClick} aria-busy={busy}>
        {busy ? "処理中…" : button}
      </button>
      {children}
    </section>
  );
}

function App() {
  const workflow = useVerification();
  const { state } = workflow;
  const disabled = !state.ready || state.pending !== null;

  return (
    <main>
      <header>
        <p className="eyebrow">ANONVOTE</p>
        <h1>ブラインド署名の検証</h1>
        <p>トークンを生成し、サーバーの署名を取得して、ブラウザで検証します。</p>
        <div className="toolbar">
          <p role="status">
            {state.ready ? "準備完了" : state.error ? "初期化に失敗しました" : "Wasm を読み込んでいます…"}
          </p>
          <button type="button" className="secondary" onClick={workflow.reset} disabled={disabled}>
            最初からやり直す
          </button>
        </div>
      </header>
      {state.error && (
        <p className="error" role="alert">
          {state.error}
        </p>
      )}
      <div className="steps">
        <Step
          number={1}
          title="公開鍵"
          location="サーバー"
          button="公開鍵を取得"
          disabled={disabled}
          busy={state.pending === "publicKey"}
          onClick={workflow.acquirePublicKey}
        >
          <Output label="Public Key" value={state.publicKey} />
        </Step>
        <Step
          number={2}
          title="トークン"
          location="ブラウザ"
          button="トークンを生成"
          disabled={disabled}
          busy={state.pending === "token"}
          onClick={workflow.generateToken}
        >
          <Output label="Token" value={state.token} />
        </Step>
        <Step
          number={3}
          title="ブラインド化"
          location="ブラウザ"
          button="トークンをブラインド化"
          disabled={disabled || !state.publicKey || !state.token}
          busy={state.pending === "blinding"}
          onClick={workflow.blind}
        >
          <Output label="Blind Token" value={state.blinding?.blind_token} />
          <Output label="Secret" value={state.blinding?.secret} />
        </Step>
        <Step
          number={4}
          title="署名の取得"
          location="サーバー"
          button="署名を取得"
          disabled={disabled || !state.blinding}
          busy={state.pending === "blindSignature"}
          onClick={workflow.acquireCertificate}
        >
          <Output label="Blind Token Signature" value={state.blindSignature} />
        </Step>
        <Step
          number={5}
          title="署名の復元"
          location="ブラウザ"
          button="署名を復元"
          disabled={disabled || !state.blindSignature}
          busy={state.pending === "signature"}
          onClick={workflow.finalize}
        >
          <Output label="Token Signature" value={state.signature} />
        </Step>
        <Step
          number={6}
          title="署名の検証"
          location="ブラウザ"
          button="署名を検証"
          disabled={disabled || !state.signature}
          busy={state.pending === "verification"}
          onClick={workflow.verify}
        >
          <p
            role="status"
            className={state.verification === null ? "result" : state.verification ? "result success" : "result error"}
          >
            {state.verification === null ? "未検証" : state.verification ? "検証成功" : "検証失敗"}
          </p>
        </Step>
      </div>
    </main>
  );
}

export default App;
