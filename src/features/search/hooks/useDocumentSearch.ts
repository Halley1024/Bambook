import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import type { SearchController, SearchLocator, SearchMatch, SearchOptions, SearchRepository, SearchStatus } from "../types/search";
import type { SearchResponse } from "../types/search";

const initialOptions: SearchOptions = { caseSensitive: false, wholeWord: false, resultLimit: 500 };
type FrontendSearchCacheEntry = { key: string; response: SearchResponse };
const frontendSearchHistory: FrontendSearchCacheEntry[] = [];

export function useDocumentSearch<TLocator extends SearchLocator>({ repository, onNavigate, cacheSize = 0 }: {
  repository: SearchRepository<TLocator> | null;
  onNavigate: (match: SearchMatch<TLocator>) => void;
  cacheSize?: number;
}): SearchController<TLocator> {
  const [query, setQuery] = useState("");
  const [options, setOptions] = useState(initialOptions);
  const [status, setStatus] = useState<SearchStatus>("idle");
  const [matches, setMatches] = useState<SearchMatch<TLocator>[]>([]);
  const [total, setTotal] = useState(0);
  const [truncated, setTruncated] = useState(false);
  const [activeIndex, setActiveIndex] = useState(-1);
  const [error, setError] = useState<string | null>(null);
  const requestIdRef = useRef(0);
  const activeRequestRef = useRef<{ id: number; repository: SearchRepository<TLocator> } | null>(null);
  const navigateRef = useRef(onNavigate);
  navigateRef.current = onNavigate;
  const requestKey = repository?.requestKey ?? "none";
  const documentKey = repository?.documentKey ?? "none";

  useEffect(() => {
    const active = activeRequestRef.current;
    if (active) void active.repository.cancel(active.id).catch(() => undefined);
    activeRequestRef.current = null;
    requestIdRef.current += 1;
    setQuery(""); setMatches([]); setTotal(0); setActiveIndex(-1); setStatus("idle"); setError(null);
  }, [documentKey]);

  useEffect(() => {
    const active = activeRequestRef.current;
    if (active) void active.repository.cancel(active.id).catch(() => undefined);
    activeRequestRef.current = null;
    requestIdRef.current += 1;
    const trimmed = query.trim();
    if (!repository || !trimmed) {
      setMatches([]); setTotal(0); setActiveIndex(-1); setStatus("idle"); setError(null);
      return;
    }
    const cacheKey = buildSearchCacheKey(repository.documentKey, trimmed, options);
    const cached = cacheSize > 0 ? readSearchCache<TLocator>(cacheKey) : null;
    if (cached) {
      setMatches(cached.matches);
      setTotal(cached.total);
      setTruncated(cached.truncated);
      setActiveIndex(cached.matches.length ? 0 : -1);
      setStatus("complete");
      setError(null);
      return;
    }
    setStatus("debouncing");
    const requestId = requestIdRef.current;
    const timer = window.setTimeout(() => {
      activeRequestRef.current = { id: requestId, repository };
      setStatus("searching"); setError(null);
      repository.search(requestId, trimmed, options)
        .then((response) => {
          if (requestId !== requestIdRef.current || response.cancelled) return;
          if (cacheSize > 0) rememberSearch(cacheKey, response, cacheSize);
          setMatches(response.matches); setTotal(response.total); setTruncated(response.truncated);
          setActiveIndex(response.matches.length ? 0 : -1); setStatus("complete");
        })
        .catch((reason) => {
          if (requestId !== requestIdRef.current) return;
          setMatches([]); setTotal(0); setActiveIndex(-1); setError(String(reason)); setStatus("error");
        })
        .finally(() => { if (activeRequestRef.current?.id === requestId) activeRequestRef.current = null; });
    }, 250);
    return () => window.clearTimeout(timer);
  }, [cacheSize, options, query, requestKey]);

  useEffect(() => () => {
    const active = activeRequestRef.current;
    if (active) void active.repository.cancel(active.id).catch(() => undefined);
  }, []);

  const navigate = useCallback((index: number) => {
    setActiveIndex(index);
    const match = matches[index];
    if (match) navigateRef.current(match);
  }, [matches]);

  return useMemo(() => ({
    query, options, status, matches, total, truncated, activeIndex, error,
    setQuery,
    setOption: (key, value) => setOptions((current) => ({ ...current, [key]: value })),
    selectResult: navigate,
    previous: () => { if (matches.length) navigate((activeIndex <= 0 ? matches.length : activeIndex) - 1); },
    next: () => { if (matches.length) navigate((activeIndex + 1) % matches.length); },
    clear: () => {
      const active = activeRequestRef.current;
      if (active) void active.repository.cancel(active.id).catch(() => undefined);
      activeRequestRef.current = null;
      requestIdRef.current += 1;
      setQuery(""); setMatches([]); setTotal(0); setTruncated(false); setActiveIndex(-1); setStatus("idle"); setError(null);
    },
  }), [activeIndex, error, matches, navigate, options, query, status, total, truncated]);
}

function buildSearchCacheKey(documentKey: string, query: string, options: SearchOptions) {
  return `${documentKey}\u0000${query}\u0000${options.caseSensitive ? 1 : 0}\u0000${options.wholeWord ? 1 : 0}\u0000${options.resultLimit}`;
}

function readSearchCache<TLocator extends SearchLocator>(key: string) {
  const entry = frontendSearchHistory.find((item) => item.key === key);
  return entry?.response as SearchResponse<TLocator> | undefined;
}

function rememberSearch<TLocator extends SearchLocator>(key: string, response: SearchResponse<TLocator>, limit: number) {
  if (frontendSearchHistory.some((item) => item.key === key)) return;
  frontendSearchHistory.push({ key, response: response as SearchResponse });
  while (frontendSearchHistory.length > Math.max(1, limit)) frontendSearchHistory.shift();
}
