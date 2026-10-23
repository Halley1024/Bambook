import { invokeCommand } from "../../../platform/tauriClient";
import type { DocumentKind, ReaderDocument } from "../types";

const recentStorageKey = "bambook.recent-documents.v1";
const closedStorageKey = "bambook.closed-documents.v1";

export type RecentDocument = {
  documentId?: string;
  path: string;
  title: string;
  kind: DocumentKind;
  lastClosedAt: number;
  availability: DocumentAvailability;
};

export type DocumentAvailability = "available" | "missing" | "inaccessible" | "notFile" | "unknown";

export type DocumentHistory = {
  recent: RecentDocument[];
  closed: RecentDocument[];
};

let legacyMigration: Promise<void> | undefined;

export const documentHistoryRepository = {
  async load() {
    await ensureLegacyHistoryMigrated();
    return invokeCommand<DocumentHistory>("load_document_history");
  },

  async recordClosed(document: ReaderDocument) {
    await ensureLegacyHistoryMigrated();
    return invokeCommand<DocumentHistory>("record_document_closed", {
      document: { documentId: document.id, path: document.path, title: document.title, kind: document.kind },
    });
  },

  async recordAccessed(document: ReaderDocument) {
    await ensureLegacyHistoryMigrated();
    return invokeCommand<DocumentHistory>("record_document_accessed", {
      document: { documentId: document.id, path: document.path, title: document.title, kind: document.kind },
    });
  },

  async confirmReopened(path: string) {
    await ensureLegacyHistoryMigrated();
    return invokeCommand<DocumentHistory>("confirm_reopened_closed_document", { path });
  },

  async clear() {
    await ensureLegacyHistoryMigrated();
    return invokeCommand<DocumentHistory>("clear_document_history");
  },
};

function ensureLegacyHistoryMigrated() {
  legacyMigration ??= migrateLegacyHistory().catch((error) => {
    legacyMigration = undefined;
    throw error;
  });
  return legacyMigration;
}

async function migrateLegacyHistory() {
  const legacy = readLegacyHistory();
  if (!legacy) return;
  await invokeCommand<DocumentHistory>("import_legacy_document_history", { history: legacy });
  removeLegacyHistory();
}

function readLegacyHistory(): DocumentHistory | null {
  const recent = readLegacyList(recentStorageKey);
  const closed = readLegacyList(closedStorageKey);
  return recent.length || closed.length ? { recent, closed } : null;
}

function readLegacyList(key: string): RecentDocument[] {
  try {
    const value: unknown = JSON.parse(window.localStorage.getItem(key) ?? "[]");
    if (!Array.isArray(value)) return [];
    return value.flatMap((item) => {
      if (!isLegacyDocument(item)) return [];
      return [{
        path: item.path,
        title: item.title,
        kind: item.kind,
        lastClosedAt: item.openedAt,
        availability: "unknown",
      }];
    });
  } catch {
    return [];
  }
}

function removeLegacyHistory() {
  window.localStorage.removeItem(recentStorageKey);
  window.localStorage.removeItem(closedStorageKey);
}

function isLegacyDocument(value: unknown): value is {
  path: string;
  title: string;
  kind: DocumentKind;
  openedAt: number;
} {
  if (!value || typeof value !== "object") return false;
  const candidate = value as Record<string, unknown>;
  return typeof candidate.path === "string"
    && typeof candidate.title === "string"
    && (candidate.kind === "pdf" || candidate.kind === "markdown")
    && typeof candidate.openedAt === "number";
}
