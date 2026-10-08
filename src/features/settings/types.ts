export type ThemePreference = "system" | "light" | "dark";
export type StartupBehavior = "restoreWorkspace" | "newMarkdown" | "startPage";
export type DocumentImportMode = "copy" | "link";
export type WorkspacePanelLayout = { leftWidth: number; rightWidth: number };
export type TranslationProvider = "baidu";

export type AppSettings = {
  annotationStorageDir?: string;
  exportDir?: string;
  pdfStorageDir?: string;
  markdownStorageDir?: string;
  pdfImportMode: DocumentImportMode;
  markdownImportMode: DocumentImportMode;
  theme: ThemePreference;
  animationsEnabled: boolean;
  confirmationRemindersEnabled: boolean;
  startupBehavior: StartupBehavior;
  pdfWorkspaceLayout: WorkspacePanelLayout;
  markdownWorkspaceLayout: WorkspacePanelLayout;
  translationProvider: TranslationProvider;
  translationApiUrl: string;
  translationApiKey: string;
  translationSecretId: string;
  translationSecretKey: string;
  translationTargetLanguage: string;
  allowThirdPartyTranslation: boolean;
  llmBaseUrl: string;
  llmModel: string;
  llmApiKey: string;
  llmTimeoutSeconds: number;
  llmTemperature: number;
};

export const defaultSettings: AppSettings = {
  pdfImportMode: "copy",
  markdownImportMode: "copy",
  theme: "system",
  animationsEnabled: true,
  confirmationRemindersEnabled: true,
  startupBehavior: "restoreWorkspace",
  pdfWorkspaceLayout: { leftWidth: 280, rightWidth: 340 },
  markdownWorkspaceLayout: { leftWidth: 280, rightWidth: 310 },
  translationProvider: "baidu",
  translationApiUrl: "https://fanyi-api.baidu.com/api/trans/vip/translate",
  translationApiKey: "",
  translationSecretId: "",
  translationSecretKey: "",
  translationTargetLanguage: "zh",
  allowThirdPartyTranslation: false,
  llmBaseUrl: "http://127.0.0.1:8080",
  llmModel: "local-model",
  llmApiKey: "",
  llmTimeoutSeconds: 120,
  llmTemperature: 0.2,
};
