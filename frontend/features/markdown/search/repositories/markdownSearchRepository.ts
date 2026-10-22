import { documentSearchRepository, type MarkdownSearchLocator, type SearchRepository } from "../../../search";

export function createMarkdownSearchRepository(sessionId: string, revision: number): SearchRepository<MarkdownSearchLocator> {
  const target = { kind: "markdown" as const, sessionId, revision };
  return {
    documentKey: `markdown:${sessionId}`,
    requestKey: `markdown:${sessionId}:${revision}`,
    search: (requestId, query, options) =>
      documentSearchRepository.search<MarkdownSearchLocator>(requestId, target, query, options),
    cancel: (requestId) => documentSearchRepository.cancel(requestId, target),
  };
}
