export type EditorDirection = "up" | "down" | "left" | "right";

export type EditorGroup = {
  id: string;
  documentIds: string[];
  activeDocumentId: string | null;
};

export type EditorLayoutNode =
  | { type: "group"; groupId: string }
  | { type: "split"; id: string; axis: "horizontal" | "vertical"; first: EditorLayoutNode; second: EditorLayoutNode };

export type EditorGroupsState = {
  root: EditorLayoutNode;
  groups: EditorGroup[];
  focusedGroupId: string;
};

export function createEditorGroupsState(): EditorGroupsState {
  const id = "editor-group-main";
  return { root: { type: "group", groupId: id }, groups: [{ id, documentIds: [], activeDocumentId: null }], focusedGroupId: id };
}

export function addDocumentToFocusedGroup(state: EditorGroupsState, documentId: string): EditorGroupsState {
  return updateGroup(state, state.focusedGroupId, (group) => ({
    ...group,
    documentIds: group.documentIds.includes(documentId) ? group.documentIds : [...group.documentIds, documentId],
    activeDocumentId: documentId,
  }));
}

export function activateGroupDocument(state: EditorGroupsState, groupId: string, documentId: string): EditorGroupsState {
  return {
    ...updateGroup(state, groupId, (group) => group.documentIds.includes(documentId)
      ? { ...group, activeDocumentId: documentId }
      : group),
    focusedGroupId: groupId,
  };
}

export function splitEditorGroup(
  state: EditorGroupsState,
  groupId: string,
  documentId: string,
  direction: EditorDirection,
): EditorGroupsState {
  const source = state.groups.find((group) => group.id === groupId);
  if (!source?.documentIds.includes(documentId)) return state;
  const newGroupId = `editor-group-${crypto.randomUUID()}`;
  const newGroup: EditorGroup = { id: newGroupId, documentIds: [documentId], activeDocumentId: documentId };
  const newLeaf: EditorLayoutNode = { type: "group", groupId: newGroupId };
  const oldLeaf: EditorLayoutNode = { type: "group", groupId };
  const newFirst = direction === "up" || direction === "left";
  const replacement: EditorLayoutNode = {
    type: "split",
    id: `editor-split-${crypto.randomUUID()}`,
    axis: direction === "up" || direction === "down" ? "vertical" : "horizontal",
    first: newFirst ? newLeaf : oldLeaf,
    second: newFirst ? oldLeaf : newLeaf,
  };
  return {
    root: replaceGroupLeaf(state.root, groupId, replacement),
    groups: [...state.groups, newGroup],
    focusedGroupId: newGroupId,
  };
}

export function moveDocumentToDirection(
  state: EditorGroupsState,
  sourceGroupId: string,
  documentId: string,
  direction: EditorDirection,
): EditorGroupsState {
  const targetGroupId = findDirectionalGroup(state, sourceGroupId, direction);
  if (!targetGroupId) return state;
  const source = state.groups.find((group) => group.id === sourceGroupId);
  if (!source?.documentIds.includes(documentId)) return state;
  const groups = state.groups.map((group) => {
    if (group.id === sourceGroupId) {
      const index = group.documentIds.indexOf(documentId);
      const documentIds = group.documentIds.filter((id) => id !== documentId);
      return { ...group, documentIds, activeDocumentId: group.activeDocumentId === documentId
        ? documentIds[Math.min(index, documentIds.length - 1)] ?? null
        : group.activeDocumentId };
    }
    if (group.id === targetGroupId) return {
      ...group,
      documentIds: group.documentIds.includes(documentId) ? group.documentIds : [...group.documentIds, documentId],
      activeDocumentId: documentId,
    };
    return group;
  });
  return collapseEmptyGroup({ ...state, groups, focusedGroupId: targetGroupId }, sourceGroupId, targetGroupId);
}

export function removeDocumentView(state: EditorGroupsState, groupId: string, documentId: string): EditorGroupsState {
  const next = updateGroup(state, groupId, (group) => {
    const index = group.documentIds.indexOf(documentId);
    const documentIds = group.documentIds.filter((id) => id !== documentId);
    return {
      ...group,
      documentIds,
      activeDocumentId: group.activeDocumentId === documentId
        ? documentIds[Math.min(index, documentIds.length - 1)] ?? null
        : group.activeDocumentId,
    };
  });
  return collapseEmptyGroup(next, groupId);
}

export function removeDocumentEverywhere(state: EditorGroupsState, documentId: string): EditorGroupsState {
  return state.groups.reduce((current, group) => removeDocumentView(current, group.id, documentId), state);
}

export function findDirectionalGroup(state: EditorGroupsState, sourceGroupId: string, direction: EditorDirection) {
  const rects = new Map<string, Rect>();
  collectRects(state.root, { left: 0, top: 0, right: 1, bottom: 1 }, rects);
  const source = rects.get(sourceGroupId);
  if (!source) return null;
  const sourceX = (source.left + source.right) / 2;
  const sourceY = (source.top + source.bottom) / 2;
  let bestId: string | null = null;
  let bestScore = Number.POSITIVE_INFINITY;
  rects.forEach((rect, id) => {
    if (id === sourceGroupId) return;
    const x = (rect.left + rect.right) / 2;
    const y = (rect.top + rect.bottom) / 2;
    const primary = direction === "left" ? sourceX - x
      : direction === "right" ? x - sourceX
        : direction === "up" ? sourceY - y : y - sourceY;
    if (primary <= 0) return;
    const secondary = direction === "left" || direction === "right" ? Math.abs(y - sourceY) : Math.abs(x - sourceX);
    const score = primary + secondary * 2;
    if (score < bestScore) {
      bestId = id;
      bestScore = score;
    }
  });
  return bestId;
}

function updateGroup(state: EditorGroupsState, groupId: string, updater: (group: EditorGroup) => EditorGroup) {
  return { ...state, groups: state.groups.map((group) => group.id === groupId ? updater(group) : group) };
}

function replaceGroupLeaf(node: EditorLayoutNode, groupId: string, replacement: EditorLayoutNode): EditorLayoutNode {
  if (node.type === "group") return node.groupId === groupId ? replacement : node;
  return { ...node, first: replaceGroupLeaf(node.first, groupId, replacement), second: replaceGroupLeaf(node.second, groupId, replacement) };
}

function collapseEmptyGroup(state: EditorGroupsState, groupId: string, preferredFocusId?: string): EditorGroupsState {
  const group = state.groups.find((item) => item.id === groupId);
  if (!group || group.documentIds.length > 0 || state.groups.length <= 1) return state;
  const root = removeGroupLeaf(state.root, groupId);
  if (!root) return state;
  const groups = state.groups.filter((item) => item.id !== groupId);
  const focusedGroupId = preferredFocusId && groups.some((item) => item.id === preferredFocusId)
    ? preferredFocusId
    : state.focusedGroupId === groupId
      ? firstGroupId(root)
      : state.focusedGroupId;
  return { root, groups, focusedGroupId };
}

function removeGroupLeaf(node: EditorLayoutNode, groupId: string): EditorLayoutNode | null {
  if (node.type === "group") return node.groupId === groupId ? null : node;
  const first = removeGroupLeaf(node.first, groupId);
  const second = removeGroupLeaf(node.second, groupId);
  if (!first) return second;
  if (!second) return first;
  return { ...node, first, second };
}

function firstGroupId(node: EditorLayoutNode): string {
  return node.type === "group" ? node.groupId : firstGroupId(node.first);
}

type Rect = { left: number; top: number; right: number; bottom: number };

function collectRects(node: EditorLayoutNode, rect: Rect, output: Map<string, Rect>) {
  if (node.type === "group") {
    output.set(node.groupId, rect);
    return;
  }
  if (node.axis === "horizontal") {
    const middle = (rect.left + rect.right) / 2;
    collectRects(node.first, { ...rect, right: middle }, output);
    collectRects(node.second, { ...rect, left: middle }, output);
  } else {
    const middle = (rect.top + rect.bottom) / 2;
    collectRects(node.first, { ...rect, bottom: middle }, output);
    collectRects(node.second, { ...rect, top: middle }, output);
  }
}
