import { useContext } from "react";
import { PdfWorkspaceContext } from "../state/PdfWorkspaceContext";

export function usePdfWorkspace() {
  const context = useContext(PdfWorkspaceContext);
  if (!context) throw new Error("usePdfWorkspace 必须在 PdfWorkspaceProvider 中使用");
  return context;
}
