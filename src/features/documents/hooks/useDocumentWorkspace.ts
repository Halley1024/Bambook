import { useContext } from "react";
import { DocumentWorkspaceContext } from "../state/DocumentWorkspaceContext";

export function useDocumentWorkspace() {
  const context = useContext(DocumentWorkspaceContext);
  if (!context) throw new Error("useDocumentWorkspace 必须在 DocumentWorkspaceProvider 中使用");
  return context;
}
