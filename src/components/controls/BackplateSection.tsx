import type { RenderConfig } from "../../types/domain";

export interface BackplateSectionProps {
  config: RenderConfig;
  onChange(patch: Partial<RenderConfig>): void;
}

export function BackplateSection(_props: BackplateSectionProps): JSX.Element {
  return (
    <section className="space-y-3">
      <h3 className="text-xs font-semibold text-[var(--text-primary)]">背板</h3>
    </section>
  );
}
