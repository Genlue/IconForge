export function TitleBar(): JSX.Element {
  return (
    <header
      data-tauri-drag-region="true"
      className="flex h-[52px] items-center gap-3 px-4"
      style={{ WebkitUserSelect: "none", userSelect: "none" }}
    >
      {/* macOS-style traffic light buttons */}
      <div className="flex items-center gap-[6px]" data-tauri-drag-region="false">
        <div className="h-3 w-3 rounded-full bg-[#EC6A5E]" />
        <div className="h-3 w-3 rounded-full bg-[#F5BF4F]" />
        <div className="h-3 w-3 rounded-full bg-[#62C554]" />
      </div>
      <span className="ml-2 text-[13px] font-semibold text-[var(--text-primary)]">
        IconForge
      </span>
    </header>
  );
}
