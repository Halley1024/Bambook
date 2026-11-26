import type { LlmMessage } from "../../platform/llmClient";

export const assistantRequestByteLimit = 96_000;
export const assistantRequestMessageLimit = 32;
export const assistantSystemPrompt = "你是 Bambook 的阅读助手。请使用简体中文，回答应准确、清晰，并优先依据用户提供的文档内容。";

export type ConversationMessage = {
  id: string;
  role: "user" | "assistant";
  content: string;
  reasoning?: string;
  model?: string;
  status?: "streaming" | "complete" | "error";
};

export function buildAssistantRequest(messages: ConversationMessage[], context: string, label: string, question: string): LlmMessage[] {
  const history: LlmMessage[] = [];
  // Failed or unfinished turns are kept visible, but are not sent as successful history.
  messages.forEach((message, index) => {
    const reply = messages[index + 1];
    if (message.role === "user" && reply?.role === "assistant" && reply.status === "complete" && reply.content.trim()) {
      history.push({ role: "user", content: message.content }, { role: "assistant", content: reply.content });
    }
  });
  const source = context.trim();
  return [
    { role: "system", content: assistantSystemPrompt },
    ...history,
    { role: "user", content: source ? `${question.trim()}\n\n参考文档内容（${label}）：\n---\n${source}\n---` : question.trim() },
  ];
}

export function measureAssistantRequest(messages: LlmMessage[]) {
  const encoder = new TextEncoder();
  const bytes = messages.reduce((sum, message) => sum + encoder.encode(message.content.trim()).byteLength, 0);
  return { bytes, count: messages.length, exceeded: bytes > assistantRequestByteLimit || messages.length > assistantRequestMessageLimit };
}
