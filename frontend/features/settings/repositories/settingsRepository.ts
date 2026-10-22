import { invokeCommand } from "../../../platform/tauriClient";
import type { AppSettings } from "../types";

export const settingsRepository = {
  load() {
    return invokeCommand<AppSettings>("load_settings");
  },

  async save(settings: AppSettings) {
    await invokeCommand<void>("save_settings", { settings: normalizeSettingsForStorage(settings) });
  },
  openDataDirectory() {
    return invokeCommand<void>("open_app_data_directory");
  },
};

function normalizeSettingsForStorage(settings: AppSettings): AppSettings {
  return {
    ...settings,
    pdfWorkspaceLayout: normalizeLayout(settings.pdfWorkspaceLayout, 280, 340),
    markdownWorkspaceLayout: normalizeLayout(settings.markdownWorkspaceLayout, 280, 310),
  };
}

function normalizeLayout(layout: AppSettings["pdfWorkspaceLayout"], defaultLeft: number, defaultRight: number) {
  return {
    leftWidth: normalizeUnsignedInteger(layout.leftWidth, defaultLeft),
    rightWidth: normalizeUnsignedInteger(layout.rightWidth, defaultRight),
  };
}

function normalizeUnsignedInteger(value: number, fallback: number) {
  return Number.isFinite(value) && value >= 0 ? Math.round(value) : fallback;
}
