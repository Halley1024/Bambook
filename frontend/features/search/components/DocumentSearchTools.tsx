import { ChevronDown, ChevronUp, Search, X } from "lucide-react";
import { useEffect, useRef, useState } from "react";
import type { SearchController, SearchLocator } from "../types/search";

export function DocumentSearchTools<TLocator extends SearchLocator>({ search, placeholder = "搜索当前文档", clearOnClose = false }: {
  search: SearchController<TLocator>;
  placeholder?: string;
  clearOnClose?: boolean;
}) {
  const [open, setOpen] = useState(false);
  const hostRef = useRef<HTMLDivElement | null>(null);
  const inputRef = useRef<HTMLInputElement | null>(null);

  function closeSearch() {
    setOpen(false);
    if (clearOnClose) search.clear();
  }

  useEffect(() => {
    if (!open) return;
    inputRef.current?.focus();
    const closeOutside = (event: PointerEvent) => {
      if (!hostRef.current?.contains(event.target as Node)) closeSearch();
    };
    const closeOnEscape = (event: KeyboardEvent) => {
      if (event.key === "Escape") closeSearch();
    };
    window.addEventListener("pointerdown", closeOutside);
    window.addEventListener("keydown", closeOnEscape);
    return () => {
      window.removeEventListener("pointerdown", closeOutside);
      window.removeEventListener("keydown", closeOnEscape);
    };
  }, [open, clearOnClose, search.clear]);

  return <div className="document-navigation-tools toolbar-group" aria-label="文档搜索">
    <div className="document-search-host" ref={hostRef}>
      <button className={`icon-button document-search-trigger ${open || search.query ? "active" : ""}`}
        onClick={() => { if (open) closeSearch(); else setOpen(true); }} data-tooltip="搜索：查找当前文档中的文字" aria-label="搜索文档" aria-expanded={open}>
        <Search size={18} />
      </button>
      {open && <div className="document-search-popover" role="dialog" aria-label="文档搜索">
        <div className="document-search-row">
          <label className="document-search-input">
            <Search size={16} aria-hidden="true" />
            <input ref={inputRef} value={search.query} onChange={(event) => search.setQuery(event.target.value)}
              placeholder={placeholder} spellCheck={false} />
          </label>
          <span className="document-search-count">{search.status === "searching" ? "搜索中" : search.total ? `${search.activeIndex + 1}/${search.total}` : "0/0"}</span>
          <button className="search-popover-button" disabled={!search.matches.length} onClick={search.previous}
            data-tooltip="上一个结果：跳转到前一处匹配" aria-label="上一个搜索结果"><ChevronUp size={17} /></button>
          <button className="search-popover-button" disabled={!search.matches.length} onClick={search.next}
            data-tooltip="下一个结果：跳转到后一处匹配" aria-label="下一个搜索结果"><ChevronDown size={17} /></button>
          <button className="search-popover-button" onClick={closeSearch}
            data-tooltip="关闭：清空结果并收起搜索面板" aria-label="关闭并清空搜索"><X size={17} /></button>
        </div>
        <div className="document-search-options">
          <SearchOption label="区分大小写" checked={search.options.caseSensitive} onChange={(value) => search.setOption("caseSensitive", value)} />
          <SearchOption label="整词" checked={search.options.wholeWord} onChange={(value) => search.setOption("wholeWord", value)} />
        </div>
        <SearchResults search={search} />
      </div>}
    </div>
  </div>;
}

function SearchResults<TLocator extends SearchLocator>({ search }: { search: SearchController<TLocator> }) {
  if (search.status === "error") return <div className="document-search-message">搜索失败：{search.error}</div>;
  if ((search.status === "searching" || search.status === "debouncing") && !search.matches.length) {
    return <div className="document-search-message">正在搜索全文…</div>;
  }
  if (search.status === "complete" && !search.matches.length) return <div className="document-search-message">没有找到匹配内容</div>;
  return <div className="document-search-results">
    {search.matches.map((match, index) => <button key={match.id}
      className={`document-search-result ${index === search.activeIndex ? "active" : ""}`}
      data-tooltip={`搜索结果：跳转到${match.locationLabel}`}
      onClick={() => search.selectResult(index)}>
      <span className="document-search-result-location">{match.locationLabel}</span>
      <span>{match.beforeText}<strong>{match.matchedText}</strong>{match.afterText}</span>
    </button>)}
    {search.truncated && <div className="document-search-message">结果过多，仅显示前 {search.matches.length} 项</div>}
  </div>;
}

function SearchOption({ label, checked, onChange }: { label: string; checked: boolean; onChange: (checked: boolean) => void }) {
  return <label className="document-search-option">
    <input type="checkbox" checked={checked} onChange={(event) => onChange(event.target.checked)} />
    <span>{label}</span>
  </label>;
}
