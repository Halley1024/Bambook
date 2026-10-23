import { ChevronDown, ChevronUp, Search, X } from "lucide-react";
import { useEffect, useRef } from "react";
import type { SearchController, SearchLocator } from "../types/search";

export function DocumentSearchPanel<TLocator extends SearchLocator>({
  search,
  placeholder = "搜索当前文档",
}: {
  search: SearchController<TLocator>;
  placeholder?: string;
}) {
  const inputRef = useRef<HTMLInputElement | null>(null);

  useEffect(() => {
    inputRef.current?.focus();
  }, []);

  return (
    <div className="document-sidebar-search">
      <div className="document-sidebar-search-controls">
        <label className="document-sidebar-search-input">
          <Search size={16} aria-hidden="true" />
          <input
            ref={inputRef}
            value={search.query}
            onChange={(event) => search.setQuery(event.target.value)}
            placeholder={placeholder}
            spellCheck={false}
          />
          {search.query && (
            <button type="button" onClick={search.clear} aria-label="清空搜索" data-tooltip="清空搜索">
              <X size={15} />
            </button>
          )}
        </label>

        <div className="document-sidebar-search-actions">
          <div className="document-sidebar-search-options">
            <SearchOption label="区分大小写" checked={search.options.caseSensitive}
              onChange={(value) => search.setOption("caseSensitive", value)} />
            <SearchOption label="全词匹配" checked={search.options.wholeWord}
              onChange={(value) => search.setOption("wholeWord", value)} />
          </div>
          <span>{search.status === "searching" ? "搜索中" : search.total ? `${search.activeIndex + 1}/${search.total}` : "0/0"}</span>
          <button type="button" disabled={!search.matches.length} onClick={search.previous}
            aria-label="上一个搜索结果" data-tooltip="上一个结果"><ChevronUp size={17} /></button>
          <button type="button" disabled={!search.matches.length} onClick={search.next}
            aria-label="下一个搜索结果" data-tooltip="下一个结果"><ChevronDown size={17} /></button>
        </div>
      </div>

      <SidebarSearchResults search={search} />
    </div>
  );
}

function SidebarSearchResults<TLocator extends SearchLocator>({ search }: { search: SearchController<TLocator> }) {
  if (search.status === "idle") return <div className="document-sidebar-search-message">输入关键词搜索当前文档</div>;
  if (search.status === "error") return <div className="document-sidebar-search-message error">搜索失败：{search.error}</div>;
  if ((search.status === "searching" || search.status === "debouncing") && !search.matches.length) {
    return <div className="document-sidebar-search-message">正在搜索全文…</div>;
  }
  if (search.status === "complete" && !search.matches.length) {
    return <div className="document-sidebar-search-message">没有找到匹配内容</div>;
  }

  return (
    <div className="document-sidebar-search-results">
      {search.matches.map((match, index) => (
        <button key={match.id} type="button"
          className={index === search.activeIndex ? "active" : ""}
          onClick={() => search.selectResult(index)}>
          <span>{match.locationLabel}</span>
          <p>{match.beforeText}<strong>{match.matchedText}</strong>{match.afterText}</p>
        </button>
      ))}
      {search.truncated && <div className="document-sidebar-search-message">结果较多，仅显示前 {search.matches.length} 项</div>}
    </div>
  );
}

function SearchOption({ label, checked, onChange }: { label: string; checked: boolean; onChange: (checked: boolean) => void }) {
  return (
    <label>
      <input type="checkbox" checked={checked} onChange={(event) => onChange(event.target.checked)} />
      <span>{label}</span>
    </label>
  );
}
