import type { RenderConfig } from "../../types/domain";

export interface ForegroundSectionProps {
  config: RenderConfig;
  onChange(patch: Partial<RenderConfig>): void;
}

export function ForegroundSection(_props: ForegroundSectionProps): JSX.Element {
  return (
    <section className="space-y-3">
      <h3 className="text-xs font-semibold text-[var(--text-primary)]">前景</h3>
    </section>
  );
}
