import { createContext, useCallback, useEffect, useMemo, useRef, useState, type ReactNode } from "react";
import type { AppNotification, NotificationKind } from "./types";

type NotificationContextValue = {
  notifications: AppNotification[];
  notify: (kind: NotificationKind, message: string) => void;
  dismiss: (id: string) => void;
};

export const NotificationContext = createContext<NotificationContextValue | null>(null);

export function NotificationProvider({ children }: { children: ReactNode }) {
  const [notifications, setNotifications] = useState<AppNotification[]>([]);
  const timersRef = useRef(new Map<string, number>());

  useEffect(
    () => () => {
      timersRef.current.forEach((timer) => window.clearTimeout(timer));
      timersRef.current.clear();
    },
    [],
  );

  const dismiss = useCallback((id: string) => {
    const timer = timersRef.current.get(id);
    if (timer) window.clearTimeout(timer);
    timersRef.current.delete(id);
    setNotifications((current) => current.filter((item) => item.id !== id));
  }, []);

  const notify = useCallback(
    (kind: NotificationKind, message: string) => {
      const id = crypto.randomUUID();
      setNotifications((current) => [...current, { id, kind, message }]);
      timersRef.current.set(id, window.setTimeout(() => dismiss(id), kind === "error" ? 6000 : 3200));
    },
    [dismiss],
  );

  const value = useMemo(() => ({ notifications, notify, dismiss }), [dismiss, notifications, notify]);
  return <NotificationContext.Provider value={value}>{children}</NotificationContext.Provider>;
}
