export function Output({ id, label, value }: { id: string; label: string; value: string | null | undefined }) {
  return (
    <div className="output">
      <span>{label}</span>
      <pre id={id}>{value ?? "未生成"}</pre>
    </div>
  );
}
