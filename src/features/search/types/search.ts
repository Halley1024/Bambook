export type SearchOptions = { caseSensitive: boolean; wholeWord: boolean; resultLimit: number };
export type SearchTarget =
  | { kind: "pdf"; sessionId: string }
  | { kind: "markdown"; sessionId: string; revision: number };
export type SearchRect = { left: number; top: number; width: number; height: number };
export type PdfSearchLocator = { kind: "pdf"; page: number; pageY: number; rects: SearchRect[] };
export type MarkdownSearchLocator = { kind: "markdown"; revision: number; startUtf16: number; endUtf16: number; line: number; column: number };
export type SearchLocator = PdfSearchLocator | MarkdownSearchLocator;
export type SearchMatch<TLocator extends SearchLocator = SearchLocator> = {
  id: string; matchedText: string; beforeText: string; afterText: string; locationLabel: string; locator: TLocator;
};
export type SearchResponse<TLocator extends SearchLocator = SearchLocator> = {
  requestId: number; total: number; truncated: boolean; cancelled: boolean; matches: SearchMatch<TLocator>[];
};
export type SearchStatus = "idle" | "debouncing" | "searching" | "complete" | "error";
export type SearchController<TLocator extends SearchLocator = SearchLocator> = {
  query: string;
  options: SearchOptions;
  status: SearchStatus;
  matches: SearchMatch<TLocator>[];
  total: number;
  truncated: boolean;
  activeIndex: number;
  error: string | null;
  setQuery: (query: string) => void;
  setOption: <K extends keyof SearchOptions>(key: K, value: SearchOptions[K]) => void;
  selectResult: (index: number) => void;
  previous: () => void;
  next: () => void;
  clear: () => void;
};

export type SearchRepository<TLocator extends SearchLocator> = {
  documentKey: string;
  requestKey: string;
  search: (requestId: number, query: string, options: SearchOptions) => Promise<SearchResponse<TLocator>>;
  cancel: (requestId: number) => Promise<void>;
};
