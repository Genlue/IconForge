import type { OuterShadowConfig } from "../../types/domain";

export interface ShadowSectionProps {
  value: OuterShadowConfig;
  onChange(patch: Partial<OuterShadowConfig>): void;
}

export function ShadowSection(_props: ShadowSectionProps): JSX.Element {
  return (
    <section className="space-y-3">
      <h3 className="text-xs font-semibold text-[var(--text-primary)]">阴影</h3>
    </section>
  );
}
