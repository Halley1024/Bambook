import { invokeCommand } from "../../../platform/tauriClient";
import type { AppSettings } from "../types";

export const settingsRepository = {
  load() {
    return invokeCommand<AppSettings>("load_settings");
  },

  async save(settings: AppSettings) {
    await invokeCommand<void>("save_settings", { settings });
  },
  openDataDirectory() {
    return invokeCommand<void>("open_app_data_directory");
  },
};
