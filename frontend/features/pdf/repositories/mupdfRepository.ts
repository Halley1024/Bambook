import { invokeCommand } from "../../../platform/tauriClient";
import type { MupdfDocument, PdfPageStructure } from "../types/pdfStructure";
import { LruCache } from "../../../shared/cache/LruCache";

const structureCache = new LruCache<string, PdfPageStructure>(24);
const renderedPageCache = new LruCache<string, Uint8Array>(10, 64 * 1024 * 1024, (value) => value.byteLength);
const structureRequests = new Map<string, Promise<PdfPageStructure>>();
const renderRequests = new Map<string, Promise<Uint8Array>>();

export const mupdfRepository = {
  open(path: string, password?: string) {
    return invokeCommand<MupdfDocument>("open_pdf_document", { path, password });
  },
  authenticate(sessionId: string, password: string) {
    return invokeCommand<MupdfDocument>("authenticate_pdf_document", { sessionId, password });
  },
  pageStructure(sessionId: string, pageIndex: number) {
    const key = `${sessionId}:${pageIndex}`;
    const cached = structureCache.get(key);
    if (cached) return Promise.resolve(cached);
    const pending = structureRequests.get(key);
    if (pending) return pending;
    const request = invokeCommand<PdfPageStructure>("get_pdf_page_structure", { sessionId, pageIndex })
      .then((page) => { structureCache.set(key, page); return page; })
      .finally(() => structureRequests.delete(key));
    structureRequests.set(key, request);
    return request;
  },
  renderPage(sessionId: string, pageIndex: number, scale: number) {
    const normalizedScale = Math.round(scale * 1000) / 1000;
    const key = `${sessionId}:${pageIndex}:${normalizedScale}`;
    const cached = renderedPageCache.get(key);
    if (cached) return Promise.resolve(cached);
    const pending = renderRequests.get(key);
    if (pending) return pending;
    const request = invokeCommand<Uint8Array>("render_pdf_page", { sessionId, pageIndex, scale: normalizedScale })
      .then((bytes) => {
        const normalized = bytes instanceof Uint8Array ? bytes : new Uint8Array(bytes);
        renderedPageCache.set(key, normalized);
        return normalized;
      })
      .finally(() => renderRequests.delete(key));
    renderRequests.set(key, request);
    return request;
  },
  close(sessionId: string) {
    return invokeCommand<void>("close_pdf_document", { sessionId }).finally(() => {
      structureCache.deleteWhere((key) => key.startsWith(`${sessionId}:`));
      renderedPageCache.deleteWhere((key) => key.startsWith(`${sessionId}:`));
    });
  },
};
