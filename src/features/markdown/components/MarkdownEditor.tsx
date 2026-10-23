import { useLayoutEffect, type KeyboardEvent, type RefObject, type UIEventHandler } from "react";
import { useMarkdownWorkspace } from "../hooks/useMarkdownWorkspace";
import type { MarkdownCommand } from "../types/markdownWorkspace";

export function MarkdownEditor({ editorRef, onScroll }: {
  editorRef: RefObject<HTMLTextAreaElement>;
  onScroll: UIEventHandler<HTMLTextAreaElement>;
}) {
  const workspace = useMarkdownWorkspace();

  useLayoutEffect(() => {
    const editor = editorRef.current;
    const request = workspace.selectionRequest;
    if (!editor || !request) return;
    editor.focus();
    editor.setSelectionRange(request.start, request.end);
  }, [workspace.selectionRequest]);

  return (
    <div className="markdown-editor-pane">
      <textarea
        ref={editorRef}
        className="markdown-editor"
        value={workspace.content}
        placeholder="开始编写 Markdown…"
        spellCheck={false}
        aria-label="Markdown 编辑器"
        onScroll={onScroll}
        onChange={(event) => workspace.updateContent(event.target.value)}
        onSelect={(event) =>
          workspace.updateSelection({
            start: event.currentTarget.selectionStart,
            end: event.currentTarget.selectionEnd,
          })
        }
        onKeyDown={(event) => {
          const markdownCommand = commandForShortcut(event);
          if (markdownCommand) {
            event.preventDefault();
            workspace.executeCommand(markdownCommand);
            return;
          }
          if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "z") {
            event.preventDefault();
            if (event.shiftKey) workspace.redo();
            else workspace.undo();
            return;
          }
          if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "y") {
            event.preventDefault();
            workspace.redo();
            return;
          }
          if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "s") {
            event.preventDefault();
            void workspace.saveActiveDocument();
          }
        }}
      />
    </div>
  );
}

function commandForShortcut(event: KeyboardEvent<HTMLTextAreaElement>): MarkdownCommand | null {
  const control = event.ctrlKey || event.metaKey;
  const key = event.key.toLowerCase();
  if (control && event.altKey && /^Digit[1-5]$/.test(event.code)) return `heading${event.code.slice(-1)}` as MarkdownCommand;
  if (event.altKey && event.shiftKey && event.key === "ArrowLeft") return "promoteHeading";
  if (event.altKey && event.shiftKey && event.key === "ArrowRight") return "demoteHeading";
  if (event.altKey && event.shiftKey && event.code === "Digit5") return "strikethrough";
  if (event.altKey && event.shiftKey && key === "m") return "blockFormula";
  if (!control) return null;
  if (!event.shiftKey && key === "b") return "bold";
  if (!event.shiftKey && key === "i") return "italic";
  if (!event.shiftKey && key === "u") return "underline";
  if (!event.shiftKey && key === "k") return "link";
  if (!event.shiftKey && event.code === "Equal") return "subscript";
  if (event.shiftKey && event.code === "Equal") return "superscript";
  if (event.shiftKey && key === "h") return "highlight";
  if (event.shiftKey && event.code === "Digit7") return "orderedList";
  if (event.shiftKey && event.code === "Digit8") return "unorderedList";
  if (event.shiftKey && event.code === "Digit9") return "taskList";
  if (event.shiftKey && key === "m") return "inlineFormula";
  if (event.shiftKey && key === "q") return "quote";
  if (event.shiftKey && event.code === "Backquote") return "code";
  if (!event.shiftKey && event.code === "Backslash") return "clearFormatting";
  return null;
}
