import type { MarkdownSearchLocator } from "../../../search";

export function navigateToMarkdownSearchLocator(
  locator: MarkdownSearchLocator,
  currentRevision: number,
  navigate: (startUtf16: number, endUtf16: number) => void,
) {
  if (locator.revision !== currentRevision) return;
  navigate(locator.startUtf16, locator.endUtf16);
}
