import { useEffect, useRef, useState } from "react";
import { ChevronDown, Minus, MoveHorizontal, Plus, Scan } from "lucide-react";
import { usePdfWorkspace } from "../hooks/usePdfWorkspace";
import { PDF_MAX_SCALE, PDF_MIN_SCALE, PDF_ZOOM_LEVELS, PDF_ZOOM_STEP, clampPdfScale } from "../pdfZoom";

export function PdfZoomControls() {
  const workspace = usePdfWorkspace();
  const [zoomValue, setZoomValue] = useState(String(Math.round(workspace.scale * 100)));
  const [menuOpen, setMenuOpen] = useState(false);
  const hostRef = useRef<HTMLDivElement>(null);
  const menuButtonRef = useRef<HTMLButtonElement>(null);
  const cancelCommit = useRef(false);
  const disabled = !workspace.activeDocument;

  useEffect(() => {
    setZoomValue(String(Math.round(workspace.scale * 100)));
    cancelCommit.current = false;
    setMenuOpen(false);
  }, [workspace.scale, workspace.activeDocumentId]);

  useEffect(() => {
    if (!menuOpen) return;
    const closeOutside = (event: PointerEvent) => {
      if (!hostRef.current?.contains(event.target as Node)) setMenuOpen(false);
    };
    const closeOnEscape = (event: KeyboardEvent) => {
      if (event.key === "Escape") { setMenuOpen(false); menuButtonRef.current?.focus(); }
    };
    window.addEventListener("pointerdown", closeOutside);
    window.addEventListener("keydown", closeOnEscape);
    return () => {
      window.removeEventListener("pointerdown", closeOutside);
      window.removeEventListener("keydown", closeOnEscape);
    };
  }, [menuOpen]);

  function commitZoom() {
    if (cancelCommit.current) { cancelCommit.current = false; return; }
    const percent = Number(zoomValue);
    if (!zoomValue || !Number.isFinite(percent)) {
      setZoomValue(String(Math.round(workspace.scale * 100)));
      return;
    }
    const scale = clampPdfScale(percent / 100);
    workspace.setScale(scale);
    setZoomValue(String(Math.round(scale * 100)));
  }

  return <div className="pdf-status-zoom" role="group" aria-label="PDF 缩放" ref={hostRef}>
    <button type="button" disabled={disabled || workspace.scale <= PDF_MIN_SCALE}
      onClick={() => workspace.setScale(workspace.scale - PDF_ZOOM_STEP)}
      data-tooltip="缩小：每次降低 10%" aria-label="缩小"><Minus size={15} /></button>
    <div className="pdf-status-scale-control">
      <input value={zoomValue} inputMode="decimal" disabled={disabled} aria-label="PDF 缩放百分比" title="输入 10 到 1000 之间的缩放比例"
        onFocus={(event) => event.currentTarget.select()}
        onChange={(event) => setZoomValue(event.target.value.replace(/[^0-9.]/g, ""))}
        onBlur={commitZoom}
        onKeyDown={(event) => {
          if (event.key === "Enter") { event.preventDefault(); event.currentTarget.blur(); }
          if (event.key === "Escape") {
            event.preventDefault();
            cancelCommit.current = true;
            setZoomValue(String(Math.round(workspace.scale * 100)));
            event.currentTarget.blur();
          }
        }} />
      <span>%</span>
      <button type="button" ref={menuButtonRef} disabled={disabled}
        onMouseDown={(event) => event.preventDefault()} onClick={() => setMenuOpen((value) => !value)}
        aria-label="选择预设缩放比例" data-tooltip="选择预设缩放比例" aria-expanded={menuOpen} aria-controls="pdf-status-zoom-presets"><ChevronDown size={12} /></button>
    </div>
    {menuOpen && <div className="pdf-status-zoom-presets" id="pdf-status-zoom-presets" role="group" aria-label="预设缩放比例">
      {PDF_ZOOM_LEVELS.map((level) => <button key={level} type="button" aria-pressed={Math.round(workspace.scale * 100) === level}
        onClick={() => { workspace.setScale(level / 100); setMenuOpen(false); menuButtonRef.current?.focus(); }}>{level}%</button>)}
    </div>}
    <button type="button" disabled={disabled || workspace.scale >= PDF_MAX_SCALE}
      onClick={() => workspace.setScale(workspace.scale + PDF_ZOOM_STEP)}
      data-tooltip="放大：每次提高 10%" aria-label="放大"><Plus size={15} /></button>
    <button type="button" disabled={disabled} aria-pressed={workspace.zoomMode === "fit-width"}
      onClick={workspace.fitToWidth} data-tooltip="适合宽度：页面宽度匹配阅读区" aria-label="适合宽度"><MoveHorizontal size={15} /></button>
    <button type="button" disabled={disabled} aria-pressed={workspace.zoomMode === "fit-page"}
      onClick={workspace.fitToPage} data-tooltip="适合页面：完整显示当前页面" aria-label="适合页面"><Scan size={15} /></button>
  </div>;
}
