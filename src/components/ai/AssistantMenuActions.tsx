import { CircleHelp, Languages, Sparkles } from "lucide-react";
import type { AssistantTextAction } from "./assistantActions";

export function AssistantMenuActions({ onAction, disabled = false }: { onAction: (action: AssistantTextAction) => void; disabled?: boolean }) {
  return <>
    <button type="button" disabled={disabled} data-tooltip="总结这段文字" onClick={() => onAction("summarize")}><Sparkles size={17} />总结</button>
    <button type="button" disabled={disabled} data-tooltip="解释这段文字" onClick={() => onAction("explain")}><CircleHelp size={17} />解释</button>
    <button type="button" disabled={disabled} data-tooltip="翻译这段文字" onClick={() => onAction("translate")}><Languages size={17} />翻译</button>
  </>;
}
