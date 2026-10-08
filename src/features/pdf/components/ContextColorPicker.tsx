import { useEffect, useRef, useState } from "react";
import { Palette } from "lucide-react";

export type ContextColorOption = { label: string; value: string };

const BASIC_COLORS: ContextColorOption[] = [
  { label: "主题色", value: "#26765A" },
  { label: "蓝色", value: "#2563EB" },
  { label: "红色", value: "#DC2626" },
  { label: "橙色", value: "#D97706" },
];

export const ANNOTATION_CONTEXT_COLORS: ContextColorOption[] = [
  ...BASIC_COLORS,
  { label: "紫色", value: "#7C3AED" },
  { label: "粉色", value: "#DB2777" },
];

export function ContextColorPicker({ value, onChange, scopeLabel, colors = BASIC_COLORS, showLabels = true, dense = false }: {
  value: string;
  onChange: (color: string) => void;
  scopeLabel: string;
  colors?: ContextColorOption[];
  showLabels?: boolean;
  dense?: boolean;
}) {
  const [customColor, setCustomColor] = useState(value);
  const inputRef = useRef<HTMLInputElement | null>(null);

  useEffect(() => setCustomColor(value), [value]);
  useEffect(() => {
    const input = inputRef.current;
    if (!input) return;
    const commit = () => onChange(input.value.toUpperCase());
    input.addEventListener("change", commit);
    return () => input.removeEventListener("change", commit);
  }, [onChange]);

  return <div className={`context-color-picker ${dense ? "dense" : ""}`} onPointerDown={(event) => event.stopPropagation()}>
    <div className="context-color-presets" aria-label={`${scopeLabel}基础颜色`}
      style={{ gridTemplateColumns: `repeat(${colors.length}, minmax(0, 1fr))` }}>
      {colors.map((color) => <button key={color.value}
        className={color.value.toLowerCase() === value.toLowerCase() ? "active" : ""}
        onClick={() => onChange(color.value)} aria-label={color.label} data-tooltip={color.label}>
        <span style={{ background: color.value }} />
        {showLabels && <small>{color.label}</small>}
      </button>)}
    </div>
    <label className="context-custom-color" data-tooltip="打开自定义色盘">
      <input ref={inputRef} type="color" value={customColor}
        onInput={(event) => setCustomColor(event.currentTarget.value.toUpperCase())} />
      <Palette size={16} /><span>色盘</span><i style={{ background: customColor }} />
    </label>
  </div>;
}
