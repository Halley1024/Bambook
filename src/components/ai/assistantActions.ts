export type AssistantTextAction = "summarize" | "explain" | "translate";
export type AssistantActionRequest = { id: string; text: string; action?: AssistantTextAction };

export const assistantTextInstructions: Record<AssistantTextAction, string> = {
  summarize: "请概括这段文字的核心内容，并列出关键要点。",
  explain: "请解释这段文字中的核心概念、术语和论证关系。",
  translate: "请将这段文字翻译为简体中文，并保留关键术语。",
};
