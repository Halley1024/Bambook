import { useContext } from "react";
import { MarkdownWorkspaceContext } from "../state/MarkdownWorkspaceContext";

export function useMarkdownWorkspace() {
  const context = useContext(MarkdownWorkspaceContext);
  if (!context) throw new Error("useMarkdownWorkspace 必须在 MarkdownWorkspaceProvider 中使用");
  return context;
}
