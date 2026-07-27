import { AppShell } from "./components/shell/AppShell";
import { useGlobalFileDrop } from "./hooks/useGlobalFileDrop";
import { useKeyboardShortcuts } from "./hooks/useKeyboardShortcuts";

export default function App(): JSX.Element {
  useGlobalFileDrop();
  useKeyboardShortcuts();

  return <AppShell />;
}
