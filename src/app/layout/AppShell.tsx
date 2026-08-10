import { useCallback, useEffect, useRef } from "react";
import { ToastViewport } from "../../features/notifications";
import { useDocumentWorkspace } from "../../features/documents";
import { useNotification } from "../../features/notifications";
import { SettingsPanel, useSettings } from "../../features/settings";
import { destroyCurrentWindow } from "../../platform/window";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { TopToolbar } from "./TopToolbar";
import { WorkspaceHost } from "./WorkspaceHost";
import { GlobalTooltip } from "../../components/tooltips/GlobalTooltip";

const supportedDropExtensions = [".pdf", ".md", ".markdown", ".mdown", ".mkd"];

export function AppShell() {
  const { panelOpen } = useSettings();
  const documents = useDocumentWorkspace();
  const notification = useNotification();
  const closeInProgressRef = useRef(false);

  useEffect(() => {
    let disposed = false;
    let unlisten: (() => void) | undefined;
    void getCurrentWebviewWindow().onCloseRequested(async (event) => {
      event.preventDefault();
      if (closeInProgressRef.current) return;
      closeInProgressRef.current = true;
      try {
        if (await documents.prepareWindowClose()) await destroyCurrentWindow();
      } catch (error) {
        notification.error(`关闭窗口失败：${String(error)}`);
      } finally {
        closeInProgressRef.current = false;
      }
    }).then((dispose) => {
      if (disposed) dispose();
      else unlisten = dispose;
    }).catch((error) => notification.error(`注册窗口关闭事件失败：${String(error)}`));
    return () => {
      disposed = true;
      unlisten?.();
    };
  }, [documents.prepareWindowClose, notification.error]);

  const handleDragOver = useCallback((event: React.DragEvent) => {
    event.preventDefault();
    event.dataTransfer.dropEffect = "copy";
  }, []);

  const handleDrop = useCallback(async (event: React.DragEvent) => {
    event.preventDefault();
    const paths = event.dataTransfer.getData("text/uri-list") || event.dataTransfer.getData("text/plain");
    if (!paths) return;
    for (const raw of paths.split("\r\n").filter(Boolean)) {
      const decoded = decodeURIComponent(raw.replace(/^file:\/\/\//, "").replace(/^file:\/\//, ""));
      const sanitizedPath = decoded.replace(/\//g, "\\").replace(/^\\+/, "\\\\");
      const lower = sanitizedPath.toLowerCase();
      if (supportedDropExtensions.some((ext) => lower.endsWith(ext))) {
        void documents.openPath(sanitizedPath).catch((error) =>
          notification.error(`拖放打开失败：${sanitizedPath}：${String(error)}`),
        );
      }
    }
  }, [documents.openPath, notification.error]);

  return (
    <div className="app-shell" onDragOver={handleDragOver} onDrop={handleDrop}>
      <TopToolbar />
      {panelOpen && <SettingsPanel />}
      <WorkspaceHost />
      <ToastViewport />
      <GlobalTooltip />
    </div>
  );
}
