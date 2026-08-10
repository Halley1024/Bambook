export type NotificationKind = "success" | "error" | "warning" | "info";

export type AppNotification = {
  id: string;
  kind: NotificationKind;
  message: string;
};
