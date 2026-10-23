import { Highlighter, Strikethrough, Underline } from "lucide-react";
import type { MarkupType } from "../../annotations";

export function MarkupIcon({ type, size = 18 }: { type: MarkupType; size?: number }) {
  if (type === "highlight") return <Highlighter size={size} />;
  if (type === "underline") return <Underline size={size} />;
  if (type === "strikeout") return <Strikethrough size={size} />;

  return <svg width={size} height={size} viewBox="0 0 18 18" aria-hidden="true">
    <path d="M2 11c1.15-2.3 2.3-2.3 3.45 0s2.3 2.3 3.45 0 2.3-2.3 3.45 0 2.3 2.3 3.45 0"
      fill="none" stroke="currentColor" strokeWidth="1.7" strokeLinecap="round" />
  </svg>;
}
