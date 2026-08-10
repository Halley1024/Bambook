import { createContext, useMemo, type ReactNode } from "react";
import type { PdfDocument } from "../../documents";

type PdfSessionValue = {
  document: PdfDocument | null;
};

export const PdfSessionContext = createContext<PdfSessionValue | null>(null);

export function PdfSessionProvider({ document, children }: { document: PdfDocument | null; children: ReactNode }) {
  const value = useMemo(() => ({ document }), [document]);
  return <PdfSessionContext.Provider value={value}>{children}</PdfSessionContext.Provider>;
}
