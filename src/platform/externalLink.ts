import { invokeCommand } from "./tauriClient";

export function openExternalUrl(url: string) {
  return invokeCommand<void>("open_external_url", { url });
}
