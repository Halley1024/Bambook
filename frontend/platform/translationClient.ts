import { invokeCommand } from "./tauriClient";
import type { AppSettings } from "../features/settings";

export type TranslationResult = { translatedText: string; detectedLanguage?: string; characterCount: number; usageCount: number; provider: string };
export type TranslationConnectionStatus = { provider: string; message: string };

function config(settings: AppSettings) {
  return { provider: settings.translationProvider, apiUrl: settings.translationApiUrl, appId: settings.translationApiKey,
    secretKey: settings.translationSecretKey, allowThirdParty: settings.allowThirdPartyTranslation };
}

export const translationClient = {
  translate(settings: AppSettings, text: string, targetLanguage: string) {
    return invokeCommand<TranslationResult>("translate_text", { config: config(settings),
      request: { text, sourceLanguage: "auto", targetLanguage } });
  },
  testConnection(settings: AppSettings) {
    return invokeCommand<TranslationConnectionStatus>("test_translation_connection", { config: config(settings) });
  },
};
