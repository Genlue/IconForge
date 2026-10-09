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
 * default; no arrow indicator. While open the header and its content sit inside
 * one rounded tinted block so the active region reads as a single group. Use
 * `variant="sub"` for nested sections so the hierarchy stays visible.
 */
export function CollapsibleSection(props: CollapsibleSectionProps): JSX.Element {
  const [open, setOpen] = useState(props.defaultOpen ?? false);
  const sub = props.variant === "sub";

  const blockClass = open
    ? sub
      ? "section-block section-block--sub"
      : "section-block"
    : sub
      ? "section-rail"
      : undefined;

  return (
    <section className={blockClass}>
      <button
        type="button"
        aria-expanded={open}
        onClick={() => setOpen((v) => !v)}
        className={`block w-full text-left transition-colors ${
          open
            ? sub
              ? "pb-0.5 pt-0.5 text-[11px] font-medium text-[var(--text-primary)]"
              : "pb-1 pt-0.5 text-xs font-semibold text-[var(--text-primary)]"
            : `rounded-md py-1 hover:bg-white/40 ${
                sub
                  ? "text-[11px] font-medium text-[var(--text-secondary)]"
                  : "px-1 text-xs font-semibold text-[var(--text-primary)]"
              }`
        }`}
      >
        {props.title}
      </button>
      {open && <div className="mt-2 space-y-3">{props.children}</div>}
    </section>
  );
}
