import { createContext } from "react";
import type { MarkdownDocument } from "../../documents";
import type {
  EditorSelection,
  EditorSelectionRequest,
  MarkdownCommand,
  MarkdownFileEntry,
  MarkdownOutlineItem,
  MarkdownOutlineNavigationRequest,
  MarkdownViewMode,
} from "../types/markdownWorkspace";
import type { MarkdownDiagnostic, MarkdownNode } from "../types/markdownAst";

export type MarkdownWorkspaceContextValue = {
  documents: MarkdownDocument[];
  activeDocument: MarkdownDocument | null;
  activeDocumentId: string | null;
  content: string;
  selection: EditorSelection;
  revision: number;
  nodes: MarkdownNode[];
  diagnostics: MarkdownDiagnostic[];
  dirty: boolean;
  saving: boolean;
  canUndo: boolean;
  canRedo: boolean;
  viewMode: MarkdownViewMode;
  fileTree: MarkdownFileEntry[];
  fileTreeLoading: boolean;
  outline: MarkdownOutlineItem[];
  activeOutlineId: string | null;
  selectionRequest: EditorSelectionRequest | null;
  outlineNavigationRequest: MarkdownOutlineNavigationRequest | null;
  sidebarCollapsed: boolean;
  helpPanelCollapsed: boolean;
  openFile: () => Promise<void>;
  openFileFromPath: (path: string) => Promise<void>;
  saveActiveDocument: () => Promise<void>;
  saveActiveDocumentAs: () => Promise<void>;
  exportHtml: () => Promise<void>;
  exportPdf: () => Promise<void>;
  activateDocument: (id: string) => void;
  closeDocument: (id: string) => void;
  closeOtherDocuments: (id: string) => void;
  reorderDocuments: (sourceId: string, targetId: string) => void;
  copyDocumentPath: (id: string) => void;
  setViewMode: (mode: MarkdownViewMode) => void;
  updateContent: (content: string) => void;
  updateSelection: (selection: EditorSelection) => void;
  executeCommand: (command: MarkdownCommand, payload?: string) => void;
  undo: () => void;
  redo: () => void;
  navigateToOutline: (item: MarkdownOutlineItem) => void;
  navigateToSearchResult: (startUtf16: number, endUtf16: number) => void;
  toggleSidebar: () => void;
  toggleHelpPanel: () => void;
};

export const MarkdownWorkspaceContext = createContext<MarkdownWorkspaceContextValue | null>(null);
