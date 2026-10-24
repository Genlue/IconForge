import { useState } from "react";
import type { ReactNode } from "react";

export interface CollapsibleSectionProps {
  title: string;
  defaultOpen?: boolean;
  /** Nested blocks render with reduced emphasis to show hierarchy. */
  variant?: "primary" | "sub";
  children: ReactNode;
}

function Chevron(): JSX.Element {
  return (
    <svg
      width="9"
      height="9"
      viewBox="0 0 12 12"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.8"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
    >
      <path d="M4.5 2.5 L8 6 L4.5 9.5" />
    </svg>
  );
}

/**
 * 检查器分组：标题行整体可点，右侧 chevron 指示状态。
 * 层级靠 hairline 分隔与缩进表达，不再使用嵌套圆角色块；
 * `variant="sub"` 用于二级分组（更小的标题 + 缩进）。
 */
export function CollapsibleSection(props: CollapsibleSectionProps): JSX.Element {
  const [open, setOpen] = useState(props.defaultOpen ?? false);
  const sub = props.variant === "sub";

  if (sub) {
    return (
      <div>
        <button
          type="button"
          aria-expanded={open}
          onClick={() => setOpen((v) => !v)}
          className="inspector-subsection__header flex w-full items-center gap-1.5 text-left transition-colors hover:text-[var(--text-primary)]"
        >
          <span className={`inline-flex transition-transform ${open ? "rotate-90" : ""}`}>
            <Chevron />
          </span>
          {props.title}
        </button>
        {open && (
          <div className="mt-1.5 flex flex-col gap-2.5 pl-3.5">{props.children}</div>
        )}
      </div>
    );
  }

  return (
    <section className={`inspector-section${open ? " inspector-section--open" : ""}`}>
      <button
        type="button"
        aria-expanded={open}
        onClick={() => setOpen((v) => !v)}
        className="inspector-section__header"
      >
        <span className="inspector-section__chevron">
          <Chevron />
        </span>
        {props.title}
      </button>
      {open && <div className="inspector-section__body">{props.children}</div>}
    </section>
  );
}
