export type ThemePreference = "system" | "light" | "dark";
export type StartupBehavior = "restoreWorkspace" | "newMarkdown" | "startPage";
export type DocumentImportMode = "copy" | "link";
export type WorkspacePanelLayout = { leftWidth: number; rightWidth: number };

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
};
