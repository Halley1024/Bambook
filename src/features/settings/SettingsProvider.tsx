import { createContext, useCallback, useEffect, useMemo, useRef, useState, type ReactNode } from "react";
import { useNotification } from "../notifications";
import { settingsRepository } from "./repositories/settingsRepository";
import { defaultSettings, type AppSettings, type ThemePreference, type WorkspacePanelLayout } from "./types";

type SettingsContextValue = {
  settings: AppSettings;
  ready: boolean;
  panelOpen: boolean;
  openPanel: () => void;
  closePanel: () => void;
  updateSettings: (patch: Partial<AppSettings>) => Promise<void>;
  updateWorkspaceLayout: (kind: "pdf" | "markdown", layout: WorkspacePanelLayout) => void;
  persistSettings: () => Promise<void>;
  setTheme: (theme: ThemePreference) => Promise<void>;
  toggleAnimations: () => Promise<void>;
  openDataDirectory: () => Promise<void>;
  resetSettings: () => Promise<void>;
};

export const SettingsContext = createContext<SettingsContextValue | null>(null);

export function SettingsProvider({ children }: { children: ReactNode }) {
  const [settings, setSettings] = useState<AppSettings>(defaultSettings);
  const [ready, setReady] = useState(false);
  const [panelOpen, setPanelOpen] = useState(false);
  const settingsRef = useRef(settings);
  const persistedLayoutsRef = useRef({
    pdfWorkspaceLayout: settings.pdfWorkspaceLayout,
    markdownWorkspaceLayout: settings.markdownWorkspaceLayout,
  });
  const saveChainRef = useRef<Promise<void>>(Promise.resolve());
  const notification = useNotification();

  const enqueueSettingsSave = useCallback((snapshot: AppSettings) => {
    const operation = saveChainRef.current
      .catch(() => undefined)
      .then(() => settingsRepository.save(snapshot));
    saveChainRef.current = operation;
    return operation;
  }, []);

  useEffect(() => {
    settingsRepository
      .load()
      .then((loaded) => {
        const next = { ...defaultSettings, ...loaded };
        settingsRef.current = next;
        persistedLayoutsRef.current = {
          pdfWorkspaceLayout: next.pdfWorkspaceLayout,
          markdownWorkspaceLayout: next.markdownWorkspaceLayout,
        };
        setSettings(next);
      })
      .catch((error) => notification.error(`读取设置失败：${String(error)}`))
      .finally(() => setReady(true));
  }, []);

  useEffect(() => {
    const media = window.matchMedia("(prefers-color-scheme: dark)");
    const applyTheme = () => {
      const resolved = settings.theme === "system" ? (media.matches ? "dark" : "light") : settings.theme;
      document.documentElement.dataset.theme = resolved;
      document.documentElement.dataset.themePreference = settings.theme;
    };
    applyTheme();
    media.addEventListener("change", applyTheme);
    return () => media.removeEventListener("change", applyTheme);
  }, [settings.theme]);

  useEffect(() => {
    document.documentElement.dataset.animations = settings.animationsEnabled ? "enabled" : "disabled";
  }, [settings.animationsEnabled]);

  const updateSettings = useCallback(
    async (patch: Partial<AppSettings>) => {
      const next = { ...settingsRef.current, ...patch };
      settingsRef.current = next;
      setSettings(next);
      try {
        await enqueueSettingsSave({ ...next, ...persistedLayoutsRef.current });
      } catch (error) {
        notification.error(`保存设置失败：${String(error)}`);
      }
    },
    [enqueueSettingsSave, notification],
  );

  const updateWorkspaceLayout = useCallback((kind: "pdf" | "markdown", layout: WorkspacePanelLayout) => {
    const key = kind === "pdf" ? "pdfWorkspaceLayout" : "markdownWorkspaceLayout";
    const next = { ...settingsRef.current, [key]: layout };
    settingsRef.current = next;
    setSettings(next);
  }, []);

  const persistSettings = useCallback(async () => {
    const snapshot = settingsRef.current;
    persistedLayoutsRef.current = {
      pdfWorkspaceLayout: snapshot.pdfWorkspaceLayout,
      markdownWorkspaceLayout: snapshot.markdownWorkspaceLayout,
    };
    try {
      await enqueueSettingsSave(snapshot);
    } catch (error) {
      notification.error(`\u4fdd\u5b58\u5e03\u5c40\u8bbe\u7f6e\u5931\u8d25\uff1a${String(error)}`);
      throw error;
    }
  }, [enqueueSettingsSave, notification]);

  const openDataDirectory = useCallback(async () => {
    try {
      await settingsRepository.openDataDirectory();
    } catch (error) {
      notification.error(`打开应用数据文件夹失败：${String(error)}`);
    }
  }, [notification]);

  const resetSettings = useCallback(async () => {
    const next = {
      ...defaultSettings,
      annotationStorageDir: settings.annotationStorageDir,
      exportDir: settings.exportDir,
    };
    settingsRef.current = next;
    persistedLayoutsRef.current = {
      pdfWorkspaceLayout: next.pdfWorkspaceLayout,
      markdownWorkspaceLayout: next.markdownWorkspaceLayout,
    };
    setSettings(next);
    try {
      await enqueueSettingsSave(next);
      notification.success("已恢复默认设置");
    } catch (error) {
      notification.error(`恢复默认设置失败：${String(error)}`);
    }
  }, [enqueueSettingsSave, notification, settings.annotationStorageDir, settings.exportDir]);

  const value = useMemo(
    () => ({
      settings,
      ready,
      panelOpen,
      openPanel: () => setPanelOpen(true),
      closePanel: () => setPanelOpen(false),
      updateSettings,
      updateWorkspaceLayout,
      persistSettings,
      setTheme: (theme: ThemePreference) => updateSettings({ theme }),
      toggleAnimations: () => updateSettings({ animationsEnabled: !settings.animationsEnabled }),
      openDataDirectory,
      resetSettings,
    }),
    [openDataDirectory, panelOpen, persistSettings, ready, resetSettings, settings, updateSettings, updateWorkspaceLayout],
  );
  return <SettingsContext.Provider value={value}>{children}</SettingsContext.Provider>;
}
