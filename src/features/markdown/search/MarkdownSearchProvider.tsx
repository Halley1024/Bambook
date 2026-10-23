import { useMemo, type ReactNode } from "react";
import { useDocumentSearch, type MarkdownSearchLocator } from "../../search";
import { useMarkdownWorkspace } from "../hooks/useMarkdownWorkspace";
import { navigateToMarkdownSearchLocator } from "./adapters/markdownSearchAdapter";
import { createMarkdownSearchRepository } from "./repositories/markdownSearchRepository";
import { MarkdownSearchContext } from "./state/MarkdownSearchContext";

export function MarkdownSearchProvider({ children }: { children: ReactNode }) {
  const workspace = useMarkdownWorkspace();
  const document = workspace.activeDocument;
  const repository = useMemo(
    () => document ? createMarkdownSearchRepository(document.sessionId, workspace.revision) : null,
    [document?.sessionId, workspace.revision],
  );
  const search = useDocumentSearch<MarkdownSearchLocator>({
    repository,
    onNavigate: (match) => navigateToMarkdownSearchLocator(
      match.locator,
      workspace.revision,
      workspace.navigateToSearchResult,
    ),
  });

  return <MarkdownSearchContext.Provider value={search}>{children}</MarkdownSearchContext.Provider>;
}
