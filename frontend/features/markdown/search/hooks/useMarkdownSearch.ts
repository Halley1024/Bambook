import { useContext } from "react";
import { MarkdownSearchContext } from "../state/MarkdownSearchContext";

export function useMarkdownSearch() {
  const context = useContext(MarkdownSearchContext);
  if (!context) throw new Error("useMarkdownSearch must be used inside MarkdownSearchProvider");
  return context;
}
