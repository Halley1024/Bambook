import { useEffect, useRef, useState } from "react";
import { Eraser, Hand,
  MousePointer2, Palette, RotateCcw, RotateCw, SquareDashedMousePointer } from "lucide-react";
import type { MarkupType } from "../../annotations";
import { usePdfWorkspace } from "../hooks/usePdfWorkspace";
import { MarkupIcon } from "./MarkupIcon";
import { ANNOTATION_CONTEXT_COLORS, ContextColorPicker } from "./ContextColorPicker";

export function PdfToolbar() {
  const workspace = usePdfWorkspace();
  const [openPopover, setOpenPopover] = useState<"color" | null>(null);
  const toolbarRef = useRef<HTMLDivElement | null>(null);

  useEffect(() => {
    if (!openPopover) return;
    const close = (event: PointerEvent) => {
      if (!toolbarRef.current?.contains(event.target as Node)) setOpenPopover(null);
    };
    window.addEventListener("pointerdown", close);
    return () => window.removeEventListener("pointerdown", close);
  }, [openPopover]);

  return <div className="mode-toolbar pdf-mode-toolbar" aria-label="PDF 工具" ref={toolbarRef}>
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
