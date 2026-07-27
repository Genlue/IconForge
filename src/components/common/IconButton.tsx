import React from "react";

export interface IconButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  label: string;
}

export function IconButton(props: IconButtonProps): JSX.Element {
  const { label, children, ...rest } = props;
  return (
    <button
      aria-label={label}
      title={label}
      className="flex items-center justify-center rounded-lg p-1.5 text-[var(--text-secondary)] transition-colors hover:bg-white/60 hover:text-[var(--text-primary)]"
      {...rest}
    >
      {children}
    </button>
  );
}
