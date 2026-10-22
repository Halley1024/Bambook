export type PdfNavigationTarget = {
  page: number;
  requestId: number;
  /** Normalized horizontal position within the PDF page. */
  pageX?: number;
  /** Normalized vertical position within the PDF page. Omit to navigate to the page top. */
  pageY?: number;
};
