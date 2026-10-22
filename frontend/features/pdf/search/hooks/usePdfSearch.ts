import { useContext } from "react";
import { PdfSearchContext } from "../state/PdfSearchContext";

export function usePdfSearch() {
  const value = useContext(PdfSearchContext);
  if (!value) throw new Error("usePdfSearch 必须在 PdfSearchProvider 中使用");
  return value;
}
