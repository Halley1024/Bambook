import { useEffect, useRef, useState } from "react";

export const ANNOTATION_COLORS = ["#26765a", "#2563eb", "#dc2626", "#d97706", "#7c3aed", "#db2777"];

export function AnnotationColorPicker({ value, onChange, onPreview, compact = false }: {
  value: string;
  onChange: (color: string) => void;
  onPreview?: (color: string) => void;
  compact?: boolean;
}) {
  const [draft, setDraft] = useState(value.toUpperCase());
  const [preview, setPreview] = useState(value.toUpperCase());
  const colorInputRef = useRef<HTMLInputElement | null>(null);
  const committedValueRef = useRef(value);
  const previewCallbackRef = useRef(onPreview);
  committedValueRef.current = value;
  previewCallbackRef.current = onPreview;
  useEffect(() => {
    setDraft(value.toUpperCase());
    setPreview(value.toUpperCase());
  }, [value]);

  useEffect(() => {
    const input = colorInputRef.current;
    if (!input) return;
    const commitNativeColor = () => commit(input.value);
    input.addEventListener("change", commitNativeColor);
    return () => input.removeEventListener("change", commitNativeColor);
  });

  useEffect(() => () => previewCallbackRef.current?.(committedValueRef.current), []);

  function commit(candidate: string) {
    const normalized = normalizeHex(candidate);
    if (!normalized) return;
    setDraft(normalized);
    setPreview(normalized);
    onPreview?.(normalized);
    onChange(normalized);
  }

  function previewColor(candidate: string) {
    const normalized = normalizeHex(candidate);
    if (!normalized) return;
    setDraft(normalized);
    setPreview(normalized);
    onPreview?.(normalized);
  }

  return (
    <div className={`annotation-color-picker ${compact ? "compact" : ""}`} onPointerDown={(event) => event.stopPropagation()}>
      <div className="annotation-color-presets" aria-label="预设颜色">
        {ANNOTATION_COLORS.map((color) => (
          <button key={color} className={`annotation-color-swatch ${color.toLowerCase() === value.toLowerCase() ? "active" : ""}`}
            style={{ background: color }} onClick={() => commit(color)} data-tooltip={`选择颜色：${color}`} aria-label={`选择颜色 ${color}`} />
        ))}
      </div>
      <div className="annotation-color-custom">
        <label title="打开调色盘">
          <input ref={colorInputRef} type="color" value={normalizeHex(preview) ?? ANNOTATION_COLORS[0]}
            onInput={(event) => previewColor(event.currentTarget.value)} />
          <span style={{ background: preview }} />
        </label>
        <input value={draft} maxLength={7} spellCheck={false} aria-label="颜色代码"
          onChange={(event) => setDraft(event.target.value.toUpperCase())}
          onBlur={() => commit(draft)}
          onKeyDown={(event) => { if (event.key === "Enter") commit(draft); }} />
      </div>
    </div>
  );
}

function normalizeHex(value: string): string | null {
  const candidate = value.trim();
  const expanded = /^#[0-9a-f]{3}$/i.test(candidate)
    ? `#${candidate.slice(1).split("").map((part) => part + part).join("")}`
    : candidate;
  return /^#[0-9a-f]{6}$/i.test(expanded) ? expanded.toUpperCase() : null;
}
