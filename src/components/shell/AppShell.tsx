export function AppShell(): JSX.Element {
  return (
    <div
      className="app-shell"
      style={{
        display: "grid",
        gridTemplateColumns: "minmax(520px, 1fr) 360px",
        gridTemplateRows: "52px minmax(420px, 1fr) 184px",
        gridTemplateAreas: `
          "title title"
          "preview controls"
          "batch controls"
        `,
        width: "100vw",
        height: "100vh",
        minWidth: 960,
        minHeight: 700,
        overflow: "hidden",
      }}
    >
      <div style={{ gridArea: "title" }}>TitleBar</div>
      <div style={{ gridArea: "preview" }}>PreviewPane</div>
      <div style={{ gridArea: "controls" }}>ParameterPanel</div>
      <div style={{ gridArea: "batch" }}>BatchTray</div>
    </div>
  );
}
