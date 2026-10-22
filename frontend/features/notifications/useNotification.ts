import { useCallback, useContext, useMemo } from "react";
import { NotificationContext } from "./NotificationProvider";

export function useNotification() {
  const context = useContext(NotificationContext);
  if (!context) throw new Error("useNotification 必须在 NotificationProvider 中使用");

  const { dismiss, notifications, notify } = context;
  const success = useCallback((message: string) => notify("success", message), [notify]);
  const error = useCallback((message: string) => notify("error", message), [notify]);
  const warning = useCallback((message: string) => notify("warning", message), [notify]);
  const info = useCallback((message: string) => notify("info", message), [notify]);

  return useMemo(
    () => ({ notifications, dismiss, success, error, warning, info }),
    [dismiss, error, info, notifications, success, warning],
  );
}
