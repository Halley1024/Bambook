import { useContext } from "react";
import { DocumentHistoryContext } from "../state/DocumentHistoryContext";

export function useDocumentHistory() {
  const context = useContext(DocumentHistoryContext);
  if (!context) throw new Error("useDocumentHistory must be used inside DocumentHistoryProvider");
  return context;
}
