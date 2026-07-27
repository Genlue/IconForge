import type { RenderConfig } from "../../types/domain";

export interface ShapeSectionProps {
  config: RenderConfig;
  onChange(patch: Partial<RenderConfig>): void;
}

export function ShapeSection(_props: ShapeSectionProps): JSX.Element {
  return (
    <section className="space-y-3">
      <h3 className="text-xs font-semibold text-[var(--text-primary)]">形状</h3>
    </section>
  );
}
