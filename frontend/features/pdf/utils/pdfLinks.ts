import type { PdfPageLink } from "../types/pdfStructure";

export function isAllowedExternalPdfLink(uri: string) {
  return /^(https?:\/\/|mailto:)/i.test(uri.trim());
}

export function pdfLinkTooltip(link: PdfPageLink) {
  if (link.targetPage) return `转到第 ${link.targetPage} 页`;
  if (isAllowedExternalPdfLink(link.uri)) return `打开外部链接：${link.uri}`;
  return `已拦截不安全链接：${link.uri || "未知地址"}`;
}
