import { Channel } from "@tauri-apps/api/core";
import { invokeCommand } from "./tauriClient";
import type { AppSettings } from "../features/settings";

export type LlmMessage = {
  role: "system" | "user" | "assistant";
  content: string;
};

export type LlmResponse = {
  content: string;
  model: string;
  promptTokens?: number | null;
  completionTokens?: number | null;
  totalTokens?: number | null;
};

export type LlmConnectionStatus = {
  provider: string;
  message: string;
};

export type LlmStreamEvent =
  | { type: "delta"; content: string }
  | { type: "reasoning"; content: string }
  | { type: "complete" };

function config(settings: AppSettings, model = settings.llmModel) {
  return {
    baseUrl: settings.llmBaseUrl,
    model,
    apiKey: settings.llmApiKey,
    timeoutSeconds: settings.llmTimeoutSeconds,
  };
}

export const llmClient = {
  chat(settings: AppSettings, messages: LlmMessage[], handlers: { onDelta: (content: string) => void; onReasoning?: (content: string) => void; onComplete: () => void }, model = settings.llmModel) {
    const onEvent = new Channel<LlmStreamEvent>();
    onEvent.onmessage = (event) => {
      if (event.type === "delta") handlers.onDelta(event.content);
      else if (event.type === "reasoning") handlers.onReasoning?.(event.content);
      else if (event.type === "complete") handlers.onComplete();
    };
    return invokeCommand<LlmResponse>("ask_llm", {
      config: config(settings, model),
      request: { messages, temperature: settings.llmTemperature },
      onEvent,
    });
  },
  testConnection(settings: AppSettings) {
    return invokeCommand<LlmConnectionStatus>("test_llm_connection", { config: config(settings) });
  },
};
