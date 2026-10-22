import { useCallback, useEffect, useMemo, useState, type ReactNode } from "react";
import { useNotification } from "../notifications";
import {
  documentHistoryRepository,
  type DocumentHistory,
} from "./repositories/documentHistoryRepository";
import { DocumentHistoryContext } from "./state/DocumentHistoryContext";
import type { ReaderDocument } from "./types";

const emptyHistory: DocumentHistory = { recent: [], closed: [] };

export function DocumentHistoryProvider({ children }: { children: ReactNode }) {
  const [history, setHistory] = useState<DocumentHistory>(emptyHistory);
  const [loading, setLoading] = useState(true);
  const { error: notifyError } = useNotification();

  const refresh = useCallback(async () => {
    setLoading(true);
    try {
      setHistory(await documentHistoryRepository.load());
    } catch (error) {
      notifyError(`读取最近文件失败：${String(error)}`);
    } finally {
      setLoading(false);
    }
  }, [notifyError]);

  useEffect(() => { void refresh(); }, [refresh]);

  const recordClosed = useCallback(async (document: ReaderDocument) => {
    if (!document.path) return;
    try {
      setHistory(await documentHistoryRepository.recordClosed(document));
    } catch (error) {
      notifyError(`保存最近文件失败：${String(error)}`);
    }
  }, [notifyError]);

  const recordAccessed = useCallback(async (document: ReaderDocument) => {
    if (!document.path) return;
    try {
      setHistory(await documentHistoryRepository.recordAccessed(document));
    } catch (error) {
      notifyError(`更新最近访问记录失败：${String(error)}`);
    }
  }, [notifyError]);

  const confirmReopened = useCallback(async (path: string) => {
    try {
      setHistory(await documentHistoryRepository.confirmReopened(path));
    } catch (error) {
      notifyError(`更新关闭文件记录失败：${String(error)}`);
    }
  }, [notifyError]);

  const clear = useCallback(async () => {
    try {
      setHistory(await documentHistoryRepository.clear());
    } catch (error) {
      notifyError(`清除最近文件失败：${String(error)}`);
    }
  }, [notifyError]);

  const value = useMemo(() => ({ ...history, loading, refresh, recordClosed, recordAccessed, confirmReopened, clear }), [
    clear, confirmReopened, history, loading, recordAccessed, recordClosed, refresh,
  ]);
  return <DocumentHistoryContext.Provider value={value}>{children}</DocumentHistoryContext.Provider>;
}
