import { useContext } from "react";
import { PdfSessionContext } from "../state/PdfSessionContext";

export function usePdfSession() {
  const value = useContext(PdfSessionContext);
  if (!value) throw new Error("usePdfSession 必须在 PdfSessionProvider 中使用");
  return value;
}
