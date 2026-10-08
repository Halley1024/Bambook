import { useMemo, type ReactNode } from "react";
import { useDocumentSearch, type PdfSearchLocator } from "../../search";
import { usePdfWorkspace } from "../hooks/usePdfWorkspace";
import { navigateToPdfSearchLocator } from "./adapters/pdfSearchAdapter";
import { createPdfSearchRepository } from "./repositories/pdfSearchRepository";
import { PdfSearchContext } from "./state/PdfSearchContext";

export function PdfSearchProvider({ children }: { children: ReactNode }) {
  const workspace = usePdfWorkspace();
  const document = workspace.activeDocument;
  const repository = useMemo(
    () => document ? createPdfSearchRepository(document.sessionId) : null,
    [document?.sessionId],
  );
  const search = useDocumentSearch<PdfSearchLocator>({
    repository,
    cacheSize: 7,
    onNavigate: (match) => navigateToPdfSearchLocator(match.locator, workspace.navigateToSearchResult),
  });
  return <PdfSearchContext.Provider value={search}>{children}</PdfSearchContext.Provider>;
}
