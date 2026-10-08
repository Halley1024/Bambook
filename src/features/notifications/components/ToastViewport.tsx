import { AlertCircle, CheckCircle2, Info, TriangleAlert, X } from "lucide-react";
import { useNotification } from "../useNotification";
import type { NotificationKind } from "../types";

const icons: Record<NotificationKind, typeof Info> = {
  success: CheckCircle2,
  error: AlertCircle,
  warning: TriangleAlert,
  info: Info,
};

export function ToastViewport() {
  const { notifications, dismiss } = useNotification();

  return (
    <div className="toast-viewport" aria-live="polite" aria-label="应用通知">
      {notifications.map((notification) => {
        const Icon = icons[notification.kind];
        return (
          <div className={`toast toast-${notification.kind}`} key={notification.id} role="status">
            <Icon size={18} />
            <span className="toast-message">{notification.message}</span>
            <button onClick={() => dismiss(notification.id)} data-tooltip="关闭：移除这条通知" aria-label="关闭通知">
              <X size={15} />
            </button>
          </div>
        );
      })}
    </div>
  );
}
