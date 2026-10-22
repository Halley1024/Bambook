import { Move, Trash2 } from "lucide-react";
import type { Annotation, AreaBorderStyle, AreaCorner } from "../../annotations";
import { ANNOTATION_CONTEXT_COLORS, ContextColorPicker } from "./ContextColorPicker";

const BORDER_STYLES: Array<{ value: AreaBorderStyle; label: string }> = [
  { value: "dotted", label: "点线" },
  { value: "dashed", label: "虚线" },
  { value: "solid", label: "实线" },
  { value: "none", label: "无线框" },
];
const CORNERS: Array<{ value: AreaCorner; label: string }> = [
  { value: "topLeft", label: "左上" },
  { value: "topRight", label: "右上" },
  { value: "bottomRight", label: "右下" },
  { value: "bottomLeft", label: "左下" },
];

export function PdfAreaAnnotationMenu({ x, y, annotation, onUpdate, onMove, onDelete }: {
  x: number;
  y: number;
  annotation: Annotation;
  onUpdate: (annotation: Annotation) => void;
  onMove: () => void;
  onDelete: () => void;
}) {
  if (!annotation.areaStyle) return null;
  const updateStyle = (changes: Partial<NonNullable<Annotation["areaStyle"]>>) => onUpdate({
    ...annotation,
    areaStyle: { ...annotation.areaStyle!, ...changes },
    updatedAt: new Date().toISOString(),
  });
  return <div className="selection-context-menu area-annotation-menu" style={{ left: x, top: y }}
    onPointerDown={(event) => event.stopPropagation()}>
    <div className="context-menu-label">线条颜色</div>
    <ContextColorPicker value={annotation.color}
      onChange={(color) => onUpdate({ ...annotation, color, updatedAt: new Date().toISOString() })}
      scopeLabel="矩形线条" colors={ANNOTATION_CONTEXT_COLORS} showLabels={false} dense />

    <div className="context-menu-separator" />
    <div className="context-menu-label">区域背景</div>
    <ContextColorPicker value={annotation.areaStyle.backgroundColor}
      onChange={(backgroundColor) => updateStyle({ backgroundColor })}
      scopeLabel="矩形背景" colors={ANNOTATION_CONTEXT_COLORS} showLabels={false} dense />
    <label className="area-range-field area-opacity-field"><span>背景透明度</span><input type="range" min="0" max="100" step="1"
      value={Math.round((1 - (annotation.areaStyle.opacity ?? 0.22)) * 100)}
      onChange={(event) => updateStyle({ opacity: 1 - Number(event.target.value) / 100 })} />
      <output>{Math.round((1 - (annotation.areaStyle.opacity ?? 0.22)) * 100)}%</output></label>

    <div className="context-menu-separator" />
    <div className="area-style-section">
      <span>线条样式</span>
      <div className="area-border-style-options">
        {BORDER_STYLES.map((item) => <button type="button" key={item.value}
          className={annotation.areaStyle!.borderStyle === item.value ? "active" : ""}
          onClick={() => updateStyle({ borderStyle: item.value })}>
          <i style={{ borderTopStyle: item.value === "none" ? "none" : item.value }} />{item.label}
        </button>)}
      </div>
      <label className="area-range-field"><span>线条粗细</span><input type="range" min="1" max="8" step="1"
        value={annotation.areaStyle.borderWidth} onChange={(event) => updateStyle({ borderWidth: Number(event.target.value) })} />
        <output>{annotation.areaStyle.borderWidth}px</output></label>
    </div>

    <div className="context-menu-separator" />
    <div className="area-style-section">
      <span>圆角位置</span>
      <div className="area-corner-options">{CORNERS.map((corner) => <label key={corner.value}>
        <input type="checkbox" checked={annotation.areaStyle!.roundedCorners[corner.value]}
          onChange={(event) => updateStyle({ roundedCorners: {
            ...annotation.areaStyle!.roundedCorners, [corner.value]: event.target.checked,
          } })} />{corner.label}
      </label>)}</div>
      <label className="area-range-field"><span>R 角弧度</span><input type="range" min="0" max="36" step="1"
        value={annotation.areaStyle.radius} onChange={(event) => updateStyle({ radius: Number(event.target.value) })} />
        <output>{annotation.areaStyle.radius}px</output></label>
    </div>

    <div className="context-menu-separator" />
    <button type="button" onClick={onMove}><Move size={17} />进入移动与调整</button>
    <button type="button" className="danger" onClick={onDelete}><Trash2 size={17} />删除选择区域</button>
  </div>;
}
