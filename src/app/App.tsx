import { AppProviders } from "./AppProviders";
import { AppShell } from "./layout/AppShell";

export default function App() {
  return (
    <AppProviders>
      <AppShell />
    </AppProviders>
  );
}
