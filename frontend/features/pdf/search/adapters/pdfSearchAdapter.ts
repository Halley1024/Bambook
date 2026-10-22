import type { PdfSearchLocator } from "../../../search";

export function navigateToPdfSearchLocator(
  locator: PdfSearchLocator,
  navigate: (page: number, pageY: number) => void,
) {
  navigate(locator.page, locator.pageY);
}
