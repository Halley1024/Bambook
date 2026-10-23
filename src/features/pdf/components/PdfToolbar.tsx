import { useEffect, useRef, useState } from "react";
import { ChevronDown, ChevronLeft, ChevronRight, Eraser, Hand, Minus,
  MousePointer2, MoveHorizontal, Palette, Plus, Redo2, RotateCcw, RotateCw, Scan, SquareDashedMousePointer, Undo2 } from "lucide-react";
import type { MarkupType } from "../../annotations";
import { usePdfWorkspace } from "../hooks/usePdfWorkspace";
import { MarkupIcon } from "./MarkupIcon";
import { ANNOTATION_CONTEXT_COLORS, ContextColorPicker } from "./ContextColorPicker";
import { PDF_MAX_SCALE, PDF_MIN_SCALE, PDF_ZOOM_LEVELS, PDF_ZOOM_STEP, clampPdfScale } from "../pdfZoom";

export function PdfToolbar() {
  const workspace = usePdfWorkspace();
  const pageCount = workspace.activeDocument?.pageCount ?? 0;
  const [pageValue, setPageValue] = useState(String(workspace.currentPage));
  const [zoomValue, setZoomValue] = useState(String(Math.round(workspace.scale * 100)));
  const [openPopover, setOpenPopover] = useState<"zoom" | "color" | null>(null);
  const toolbarRef = useRef<HTMLDivElement | null>(null);

  useEffect(() => setPageValue(String(workspace.currentPage)), [workspace.currentPage]);
  useEffect(() => setZoomValue(String(Math.round(workspace.scale * 100))), [workspace.scale]);
  useEffect(() => {
    if (!openPopover) return;
    const close = (event: PointerEvent) => {
      if (!toolbarRef.current?.contains(event.target as Node)) setOpenPopover(null);
    };
    window.addEventListener("pointerdown", close);
    return () => window.removeEventListener("pointerdown", close);
  }, [openPopover]);

  function commitPage() {
    const requested = Number.parseInt(pageValue, 10);
    if (!Number.isFinite(requested) || pageCount === 0) {
      setPageValue(String(workspace.currentPage));
      return;
    }
    const page = Math.max(1, Math.min(pageCount, requested));
    setPageValue(String(page));
    if (page !== workspace.currentPage) workspace.navigateToPage(page);
  }

  function commitZoom() {
    const requestedPercent = Number.parseFloat(zoomValue);
    if (!Number.isFinite(requestedPercent)) {
      setZoomValue(String(Math.round(workspace.scale * 100)));
      return;
    }
    const scale = clampPdfScale(requestedPercent / 100);
    workspace.setScale(scale);
    setZoomValue(String(Math.round(scale * 100)));
  }

  return <div className="mode-toolbar pdf-mode-toolbar" aria-label="PDF 工具" ref={toolbarRef}>
    <div className="toolbar-group pdf-page-navigation" aria-label="PDF 页面与阅读历史">
      <button className="icon-button" data-tooltip="上一页：跳转到前一页" aria-label="上一页" disabled={!workspace.activeDocument || workspace.currentPage <= 1}
        onClick={() => workspace.navigateToPage(workspace.currentPage - 1)}><ChevronLeft size={19} /></button>
      <label className="pdf-page-input" title="输入页码并按回车跳转">
        <input value={pageValue} inputMode="numeric" aria-label="当前页码"
          onChange={(event) => setPageValue(event.target.value.replace(/[^0-9]/g, ""))} onBlur={commitPage}
          onKeyDown={(event) => {
            if (event.key === "Enter") { commitPage(); event.currentTarget.blur(); }
            if (event.key === "Escape") { setPageValue(String(workspace.currentPage)); event.currentTarget.blur(); }
          }} />
        <span>/ {pageCount}</span>
      </label>
      <button className="icon-button" data-tooltip="下一页：跳转到后一页" aria-label="下一页" disabled={!workspace.activeDocument || workspace.currentPage >= pageCount}
        onClick={() => workspace.navigateToPage(workspace.currentPage + 1)}><ChevronRight size={19} /></button>
      <button className="icon-button" data-tooltip="历史后退：返回上一个阅读位置" aria-label="阅读历史后退"
        disabled={!workspace.canNavigateHistoryBack} onClick={workspace.navigateHistoryBack}><Undo2 size={18} /></button>
      <button className="icon-button" data-tooltip="历史前进：前往下一个阅读位置" aria-label="阅读历史前进"
        disabled={!workspace.canNavigateHistoryForward} onClick={workspace.navigateHistoryForward}><Redo2 size={18} /></button>
    </div>

    <div className="toolbar-group pdf-zoom-tools" aria-label="PDF 缩放">
      <button className="icon-button" disabled={workspace.scale <= PDF_MIN_SCALE}
        onClick={() => workspace.setScale(workspace.scale - PDF_ZOOM_STEP)}
        data-tooltip="缩小：每次降低 10%" aria-label="缩小"><Minus size={18} /></button>
      <div className="toolbar-popover-host">
        <div className={`pdf-scale-control ${openPopover === "zoom" ? "active" : ""}`}>
          <input value={zoomValue} inputMode="decimal" aria-label="PDF 缩放百分比" title="输入 10 到 1000 之间的缩放比例"
            onFocus={(event) => event.currentTarget.select()}
            onChange={(event) => setZoomValue(event.target.value.replace(/[^0-9.]/g, ""))}
            onBlur={commitZoom}
            onKeyDown={(event) => {
              if (event.key === "Enter") { commitZoom(); event.currentTarget.blur(); }
              if (event.key === "Escape") { setZoomValue(String(Math.round(workspace.scale * 100))); event.currentTarget.blur(); }
            }} />
          <span>%</span>
          <button type="button" onMouseDown={(event) => event.preventDefault()}
            onClick={() => setOpenPopover(openPopover === "zoom" ? null : "zoom")}
            data-tooltip="选择预设缩放比例" aria-label="选择预设缩放比例" aria-haspopup="menu" aria-expanded={openPopover === "zoom"}>
            <ChevronDown size={14} />
          </button>
        </div>
        {openPopover === "zoom" && <div className="toolbar-popover zoom-level-menu" role="menu">
          {PDF_ZOOM_LEVELS.map((level) => <button key={level} role="menuitemradio"
            data-tooltip={`缩放：将页面设置为 ${level}%`}
            aria-checked={Math.round(workspace.scale * 100) === level}
            onClick={() => { workspace.setScale(level / 100); setOpenPopover(null); }}>{level}%</button>)}
        </div>}
      </div>
      <button className="icon-button" disabled={workspace.scale >= PDF_MAX_SCALE}
        onClick={() => workspace.setScale(workspace.scale + PDF_ZOOM_STEP)}
        data-tooltip="放大：每次提高 10%" aria-label="放大"><Plus size={18} /></button>
      <button className={`icon-button fit-view-button ${workspace.zoomMode === "fit-width" ? "active" : ""}`}
        onClick={workspace.fitToWidth} data-tooltip="适合宽度：页面宽度匹配阅读区" aria-label="适合宽度"><MoveHorizontal size={18} /></button>
      <button className={`icon-button fit-view-button ${workspace.zoomMode === "fit-page" ? "active" : ""}`}
        onClick={workspace.fitToPage} data-tooltip="适合页面：完整显示当前页面" aria-label="适合页面"><Scan size={18} /></button>
    </div>

    <div className="toolbar-group pdf-annotation-tools" aria-label="PDF 文字批注工具">
      <button className="tool-button annotation-history-button" disabled={!workspace.canUndoAnnotations}
        onClick={workspace.undoAnnotations} data-tooltip="撤销批注：撤销最近一次修改（Ctrl+Z）">
        <RotateCcw size={17} /><span className="tool-label">撤销批注</span>
      </button>
      <button className="tool-button annotation-history-button" disabled={!workspace.canRedoAnnotations}
        onClick={workspace.redoAnnotations} data-tooltip="重做批注：恢复刚撤销的修改（Ctrl+Y）">
        <RotateCw size={17} /><span className="tool-label">重做批注</span>
      </button>
      <button className={`tool-button annotation-mode-button ${workspace.interactionMode === "pointer" ? "active" : ""}`}
        onClick={workspace.selectPointerTool} data-tooltip="点击：选择文字或操作页面内容" aria-label="点击工具">
        <MousePointer2 size={17} /><span className="tool-label">点击</span>
      </button>
      <button className={`tool-button annotation-mode-button ${workspace.interactionMode === "pan" ? "active" : ""}`}
        onClick={workspace.selectPanTool} data-tooltip="拖拽：按住鼠标移动 PDF 页面" aria-label="拖拽工具">
        <Hand size={17} /><span className="tool-label">拖拽</span>
      </button>
      {(["underline", "squiggly", "strikeout", "highlight"] as MarkupType[]).map((type) => <button key={type}
        className={`tool-button markup-tool-button ${workspace.interactionMode === "markup" && workspace.activeMarkupType === type ? "active" : ""}`}
        onClick={() => workspace.selectMarkupTool(type)} data-tooltip={`${markupLabel(type)}：选中文字后自动添加样式`} aria-label={markupLabel(type)}>
        <MarkupIcon type={type} size={17} /><span className="tool-label">{markupLabel(type)}</span>
      </button>)}
      <button className={`tool-button annotation-mode-button ${workspace.interactionMode === "area" ? "active" : ""}`}
        onClick={workspace.selectAreaTool} data-tooltip="选择区域：在 PDF 页面拖动绘制矩形区域" aria-label="选择区域工具">
        <SquareDashedMousePointer size={17} /><span className="tool-label">选择区域</span>
      </button>
      <div className="toolbar-popover-host">
        <button className={`tool-button annotation-palette-button ${openPopover === "color" ? "active" : ""}`}
          onClick={() => setOpenPopover(openPopover === "color" ? null : "color")}
          data-tooltip="调色盘：设置新批注或当前批注的颜色" aria-label="调色盘" aria-haspopup="dialog" aria-expanded={openPopover === "color"}>
          <Palette className="annotation-palette-icon" size={18} style={{ color: workspace.activeAnnotationColor }} />
          <span className="tool-label">调色盘</span>
        </button>
        {openPopover === "color" && <div className="toolbar-popover annotation-toolbar-color-popover">
          <ContextColorPicker value={workspace.activeAnnotationColor} onChange={workspace.setAnnotationColor}
            scopeLabel="批注" colors={ANNOTATION_CONTEXT_COLORS} />
        </div>}
      </div>
      <button className={`tool-button annotation-mode-button ${workspace.interactionMode === "eraser" ? "active" : ""}`}
        onClick={workspace.selectEraserTool} data-tooltip="橡皮：滑过批注以删除样式和笔记" aria-label="橡皮工具">
        <Eraser size={17} /><span className="tool-label">橡皮</span>
      </button>
    </div>

  </div>;
}

function markupLabel(type: MarkupType) {
  if (type === "highlight") return "高亮";
  if (type === "underline") return "下划线";
  if (type === "squiggly") return "波浪线";
  return "删除线";
}
