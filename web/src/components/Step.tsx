import type { ReactNode } from "react";

export function Step({
  number,
  title,
  location,
  children,
}: {
  number: number;
  title: string;
  location: string;
  children: ReactNode;
}) {
  return (
    <section className="step" aria-labelledby={`step-${number}`}>
      <div className="step-heading">
        <span className="step-number">{number}</span>
        <h2 id={`step-${number}`}>{title}</h2>
        <span className="location">{location}</span>
      </div>
      {children}
    </section>
  );
}
