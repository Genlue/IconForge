import { useState } from "react";
import type { ReactNode } from "react";

export interface CollapsibleSectionProps {
  title: string;
  defaultOpen?: boolean;
  /** Nested blocks render with reduced emphasis to show hierarchy. */
  variant?: "primary" | "sub";
  children: ReactNode;
}

/**
 * Collapsible block: the whole title row is the toggle target. Closed by
 * default; no arrow indicator. Use `variant="sub"` for nested sections so the
 * hierarchy stays visible.
 */
export function CollapsibleSection(props: CollapsibleSectionProps): JSX.Element {
  const [open, setOpen] = useState(props.defaultOpen ?? false);
  const sub = props.variant === "sub";

  return (
    <section className={sub ? "border-l-2 border-[var(--border-hairline)] pl-2.5" : undefined}>
      <button
        type="button"
        onClick={() => setOpen((v) => !v)}
        className={`block w-full rounded-md py-1 text-left hover:bg-white/40 ${
          sub
            ? "text-[11px] font-medium text-[var(--text-secondary)]"
            : "px-1 text-xs font-semibold text-[var(--text-primary)]"
        }`}
      >
        {props.title}
      </button>
      {open && (
        <div className={`mt-2 space-y-3 pb-1 ${sub ? "" : "px-1"}`}>{props.children}</div>
      )}
    </section>
  );
}