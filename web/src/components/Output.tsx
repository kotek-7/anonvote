export function Output({ label, value }: { label: string; value: string | null | undefined }) {
  return (
    <div className="output">
      <span>{label}</span>
      <pre>{value ?? "未生成"}</pre>
    </div>
  );
}
