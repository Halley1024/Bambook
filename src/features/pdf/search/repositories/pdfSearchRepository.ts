import { documentSearchRepository, type PdfSearchLocator, type SearchRepository } from "../../../search";

export function createPdfSearchRepository(sessionId: string): SearchRepository<PdfSearchLocator> {
  const target = { kind: "pdf" as const, sessionId };
  return {
    documentKey: `pdf:${sessionId}`,
    requestKey: `pdf:${sessionId}`,
    search: (requestId, query, options) =>
      documentSearchRepository.search<PdfSearchLocator>(requestId, target, query, options),
    cancel: (requestId) => documentSearchRepository.cancel(requestId, target),
  };
}
