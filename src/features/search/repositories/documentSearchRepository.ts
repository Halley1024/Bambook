import { invokeCommand } from "../../../platform/tauriClient";
import type { SearchLocator, SearchOptions, SearchResponse, SearchTarget } from "../types/search";

export const documentSearchRepository = {
  search<TLocator extends SearchLocator>(requestId: number, target: SearchTarget, query: string, options: SearchOptions) {
    return invokeCommand<SearchResponse<TLocator>>("search_document", { request: { requestId, target, query, options } });
  },
  cancel(requestId: number, target: SearchTarget) {
    return invokeCommand<void>("cancel_document_search", { request: { requestId, target } });
  },
};
