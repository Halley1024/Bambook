import { useCallback, useMemo, useReducer } from "react";

type IdentifiableDocument = { id: string };

type TabsState<TDocument> = {
  documents: TDocument[];
  activeId: string | null;
};

type TabsAction<TDocument> =
  | { type: "open"; document: TDocument }
  | { type: "activate"; id: string }
  | { type: "deactivate" }
  | { type: "close"; id: string }
  | { type: "replace"; document: TDocument }
  | { type: "closeOthers"; id: string }
  | { type: "closeAll" }
  | { type: "reorder"; sourceId: string; targetId: string };

function tabsReducer<TDocument extends IdentifiableDocument>(
  state: TabsState<TDocument>,
  action: TabsAction<TDocument>,
): TabsState<TDocument> {
  switch (action.type) {
    case "open": {
      const exists = state.documents.some((item) => item.id === action.document.id);
      return {
        documents: exists ? state.documents : [...state.documents, action.document],
        activeId: action.document.id,
      };
    }
    case "activate":
      return state.documents.some((item) => item.id === action.id) ? { ...state, activeId: action.id } : state;
    case "deactivate":
      return { ...state, activeId: null };
    case "close": {
      const index = state.documents.findIndex((item) => item.id === action.id);
      if (index < 0) return state;
      const documents = state.documents.filter((item) => item.id !== action.id);
      const activeId =
        state.activeId === action.id ? (documents[Math.min(index, documents.length - 1)]?.id ?? null) : state.activeId;
      return { documents, activeId };
    }
    case "replace": {
      const index = state.documents.findIndex((item) => item.id === action.document.id);
      if (index < 0) return state;
      const documents = [...state.documents];
      documents[index] = action.document;
      return { ...state, documents };
    }
    case "closeOthers": {
      const document = state.documents.find((item) => item.id === action.id);
      return document ? { documents: [document], activeId: document.id } : state;
    }
    case "closeAll":
      return { documents: [], activeId: null };
    case "reorder": {
      const sourceIndex = state.documents.findIndex((item) => item.id === action.sourceId);
      const targetIndex = state.documents.findIndex((item) => item.id === action.targetId);
      if (sourceIndex < 0 || targetIndex < 0 || sourceIndex === targetIndex) return state;
      const documents = [...state.documents];
      const [source] = documents.splice(sourceIndex, 1);
      documents.splice(targetIndex, 0, source);
      return { ...state, documents };
    }
  }
}

export function useWorkspaceTabs<TDocument extends IdentifiableDocument>() {
  const [state, dispatch] = useReducer(tabsReducer<TDocument>, { documents: [], activeId: null });
  const activeDocument = useMemo(
    () => state.documents.find((item) => item.id === state.activeId) ?? null,
    [state.activeId, state.documents],
  );

  return {
    documents: state.documents,
    activeId: state.activeId,
    activeDocument,
    open: useCallback((document: TDocument) => dispatch({ type: "open", document }), []),
    activate: useCallback((id: string) => dispatch({ type: "activate", id }), []),
    deactivate: useCallback(() => dispatch({ type: "deactivate" }), []),
    close: useCallback((id: string) => dispatch({ type: "close", id }), []),
    replace: useCallback((document: TDocument) => dispatch({ type: "replace", document }), []),
    closeOthers: useCallback((id: string) => dispatch({ type: "closeOthers", id }), []),
    closeAll: useCallback(() => dispatch({ type: "closeAll" }), []),
    reorder: useCallback(
      (sourceId: string, targetId: string) => dispatch({ type: "reorder", sourceId, targetId }),
      [],
    ),
  };
}
