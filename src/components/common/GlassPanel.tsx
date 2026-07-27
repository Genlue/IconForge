import React from "react";

export interface GlassPanelProps extends React.HTMLAttributes<HTMLDivElement> {}

export function GlassPanel(props: GlassPanelProps): JSX.Element {
  const { className = "", children, ...rest } = props;
  return (
    <div className={`glass-panel ${className}`} {...rest}>
      {children}
    </div>
  );
}
