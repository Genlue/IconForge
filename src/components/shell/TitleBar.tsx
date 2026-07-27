export function TitleBar(): JSX.Element {
  return (
    <header
      data-tauri-drag-region="true"
      className="flex h-[52px] items-center justify-between px-4"
    >
      <span className="text-sm font-semibold text-[var(--text-primary)]">
        IconForge
      </span>
    </header>
  );
}
