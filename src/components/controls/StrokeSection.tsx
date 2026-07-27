import type { StrokeConfig } from "../../types/domain";

export interface StrokeSectionProps {
  value: StrokeConfig;
  onChange(patch: Partial<StrokeConfig>): void;
}

export function StrokeSection(_props: StrokeSectionProps): JSX.Element {
  return (
    <section className="space-y-3">
      <h3 className="text-xs font-semibold text-[var(--text-primary)]">描边</h3>
    </section>
  );
}
