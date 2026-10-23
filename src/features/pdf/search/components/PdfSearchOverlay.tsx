import { usePdfSearch } from "../hooks/usePdfSearch";

export function PdfSearchOverlay({ pageNumber }: { pageNumber: number }) {
  const search = usePdfSearch();
  const activeId = search.matches[search.activeIndex]?.id;
  const matches = search.matches.filter((match) => match.locator.page === pageNumber);
  if (!matches.length) return null;
  return <div className="pdf-search-layer" aria-hidden="true">
    {matches.flatMap((match) => match.locator.rects.map((rect, index) => (
      <span key={`${match.id}-${index}`} className={match.id === activeId ? "active" : ""} style={{
        left: `${rect.left * 100}%`, top: `${rect.top * 100}%`, width: `${rect.width * 100}%`, height: `${rect.height * 100}%`,
      }} />
    )))}
  </div>;
}
