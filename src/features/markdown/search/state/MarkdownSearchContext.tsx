import { createContext } from "react";
import type { MarkdownSearchLocator, SearchController } from "../../../search";

export const MarkdownSearchContext = createContext<SearchController<MarkdownSearchLocator> | null>(null);
