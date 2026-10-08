import { useEffect, useRef, useState } from "react";
import type { PdfDocument } from "../../documents";
import { mupdfRepository } from "../repositories/mupdfRepository";
import { Bookmark } from "lucide-react";

export function PdfThumbnailList({ document, currentPage, onNavigate, bookmarkCounts }: {
  document: PdfDocument;
  currentPage: number;
  onNavigate: (page: number) => void;
  bookmarkCounts?: ReadonlyMap<number, number>;
}) {
  return (
    <div className="thumbnail-list" aria-label="PDF 页面视图">
      {Array.from({ length: document.pageCount }, (_, index) => {
        const pageNumber = index + 1;
        return (
          <PdfThumbnail key={pageNumber} document={document} pageNumber={pageNumber}
            bookmarkCount={bookmarkCounts?.get(pageNumber) ?? 0}
            active={pageNumber === currentPage} onClick={() => onNavigate(pageNumber)} />
        );
      })}
    </div>
  );
}

function PdfThumbnail({ document, pageNumber, active, bookmarkCount, onClick }: {
  document: PdfDocument;
  pageNumber: number;
  active: boolean;
  bookmarkCount: number;
  onClick: () => void;
}) {
  const hostRef = useRef<HTMLButtonElement | null>(null);
  const [visible, setVisible] = useState(pageNumber <= 3);
  const [imageUrl, setImageUrl] = useState<string | null>(null);
  const [aspectRatio, setAspectRatio] = useState(0.707);

  useEffect(() => {
    const host = hostRef.current;
    if (!host) return;
    const observer = new IntersectionObserver(([entry]) => {
      setVisible(entry.isIntersecting);
    }, { rootMargin: "240px 0px" });
    observer.observe(host);
    return () => observer.disconnect();
  }, []);

  useEffect(() => {
    if (!visible) { setImageUrl(null); return; }
    let cancelled = false;
    let url: string | null = null;
    Promise.all([
      mupdfRepository.pageStructure(document.sessionId, pageNumber - 1),
      mupdfRepository.renderPage(document.sessionId, pageNumber - 1, 0.5),
    ]).then(([page, bytes]) => {
      if (cancelled) return;
      setAspectRatio(page.width / page.height);
      url = URL.createObjectURL(new Blob([new Uint8Array(bytes).buffer], { type: "image/png" }));
      setImageUrl(url);
    }).catch(() => undefined);
    return () => { cancelled = true; if (url) URL.revokeObjectURL(url); };
  }, [document.sessionId, pageNumber, visible]);

  return (
    <button ref={hostRef} className={`page-thumbnail ${active ? "active" : ""}`} onClick={onClick}
      aria-current={active ? "page" : undefined} data-tooltip={`页面导航：跳转到第 ${pageNumber} 页`}>
      <span className="thumbnail-paper" style={{ aspectRatio }}>
        {imageUrl && <img src={imageUrl} alt="" />}
        {bookmarkCount > 0 && <span className="thumbnail-bookmark" title={`${bookmarkCount} 个书签`}>
          <Bookmark size={15} fill="currentColor" />
          {bookmarkCount > 1 && <small>{bookmarkCount}</small>}
        </span>}
      </span>
      <span className="thumbnail-number">{pageNumber}</span>
    </button>
  );
}
