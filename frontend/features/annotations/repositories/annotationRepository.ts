import { invokeCommand } from "../../../platform/tauriClient";
import type { ReaderDocument } from "../../documents";
import type { AppSettings } from "../../settings";
import type { Annotation } from "../types";

type AnnotationStorageSettings = Pick<AppSettings, "annotationStorageDir">;

export const annotationRepository = {
  load(documentId: string, sourcePath: string | undefined, settings: AnnotationStorageSettings) {
    return invokeCommand<Annotation[]>("load_annotations", { documentId, sourcePath, settings });
  },

  async save(documentId: string, sourcePath: string | undefined, annotations: Annotation[], settings: AnnotationStorageSettings) {
    await invokeCommand<void>("save_annotations", { documentId, sourcePath, annotations, settings });
  },

  async migrate(fromDocumentId: string, toDocumentId: string, settings: AnnotationStorageSettings) {
    await invokeCommand<void>("migrate_pdf_annotations", {
      fromDocumentId,
      toDocumentId,
      settings,
    });
  },

  exportMarkdown(document: ReaderDocument, annotations: Annotation[], settings: AppSettings) {
    const exportableDocument = { ...document, bytes: undefined };
    return invokeCommand<string>("export_annotations_markdown", {
      document: exportableDocument,
      annotations,
      settings,
    });
  },
};
