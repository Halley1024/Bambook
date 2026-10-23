import { createContext } from "react";
import type { PdfSearchLocator, SearchController } from "../../../search";

export const PdfSearchContext = createContext<SearchController<PdfSearchLocator> | null>(null);
