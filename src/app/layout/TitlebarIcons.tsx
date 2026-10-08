import { Bell, ChevronDown, Copy, Home, Maximize2, Minus, Plus, X, type LucideProps } from "lucide-react";

export type TitlebarIconSlot =
  | "navigation.home"
  | "tabs.create"
  | "tabs.list"
  | "notifications.updates"
  | "window.minimize"
  | "window.maximize"
  | "window.restore"
  | "window.close";

const icons: Record<TitlebarIconSlot, React.ComponentType<LucideProps>> = {
  "navigation.home": Home,
  "tabs.create": Plus,
  "tabs.list": ChevronDown,
  "notifications.updates": Bell,
  "window.minimize": Minus,
  "window.maximize": Maximize2,
  "window.restore": Copy,
  "window.close": X,
};

/** Semantic icon boundary reserved for future theme/icon plugins. */
export function TitlebarIcon({ slot, ...props }: LucideProps & { slot: TitlebarIconSlot }) {
  const Icon = icons[slot];
  return <Icon {...props} />;
}
