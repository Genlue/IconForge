export function PreviewPane(): JSX.Element {
  return (
    <div className="flex h-full items-center justify-center p-8">
      <div className="glass-panel flex aspect-square max-h-[420px] min-h-[256px] w-full max-w-[420px] items-center justify-center">
        <span className="text-[var(--text-secondary)]">拖入文件开始</span>
      </div>
    </div>
  );
}
