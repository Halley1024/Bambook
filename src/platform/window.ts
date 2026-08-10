import { WebviewWindow, getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";

export function openReaderWindow() {
  const label = `reader-${crypto.randomUUID()}`;
  return new Promise<void>((resolve, reject) => {
    const window = new WebviewWindow(label, {
      url: "/",
      title: "Bambook",
      width: 1280,
      height: 820,
      minWidth: 1080,
      minHeight: 640,
    });
    void window.once("tauri://created", () => resolve());
    void window.once<string>("tauri://error", (event) => reject(new Error(event.payload)));
  });
}

export function closeCurrentWindow() {
  return getCurrentWebviewWindow().close();
}

export function destroyCurrentWindow() {
  return getCurrentWebviewWindow().destroy();
}
