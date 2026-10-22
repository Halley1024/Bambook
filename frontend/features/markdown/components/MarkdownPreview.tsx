import { useEffect, useRef, type RefObject, type UIEventHandler } from "react";
import type { MarkdownNode } from "../types/markdownAst";
import { useMarkdownSearch } from "../search";
import { MarkdownAstRenderer } from "./MarkdownAstRenderer";

export function MarkdownPreview({ nodes, scrollRef, onScroll }: {
  nodes: MarkdownNode[];
  scrollRef: RefObject<HTMLDivElement>;
  onScroll: UIEventHandler<HTMLDivElement>;
}) {
  const search = useMarkdownSearch();
  const activeMatchId = search.matches[search.activeIndex]?.id ?? null;
  const previewRef = useRef<HTMLElement | null>(null);

  useEffect(() => {
    if (!activeMatchId) return;
    const match = previewRef.current?.querySelector<HTMLElement>(`[data-search-match="${activeMatchId}"]`);
    match?.scrollIntoView({ block: "center", behavior: "smooth" });
  }, [activeMatchId]);

  return (
    <div ref={scrollRef} className="markdown-page" onScroll={onScroll}>
      <article ref={previewRef} className="markdown-body" data-search={search.query}>
        <MarkdownAstRenderer nodes={nodes} matches={search.matches} activeMatchId={activeMatchId} />
      </article>
    </div>
  );
}
