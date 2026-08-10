import { useCallback, useEffect, useRef, useState, type PointerEvent as ReactPointerEvent } from "react";

type PanelSide = "left" | "right";
type WorkspacePanelWidths = { leftWidth: number; rightWidth: number };

const minimumContentWidth = 520;

export function useWorkspacePanelWidths(widths: WorkspacePanelWidths, onWidthsChange: (widths: WorkspacePanelWidths) => void) {
  const [resizing, setResizing] = useState<PanelSide | null>(null);
  const dragRef = useRef<{ side: PanelSide; startX: number; startWidth: number } | null>(null);
  const widthsRef = useRef(widths);
  const onWidthsChangeRef = useRef(onWidthsChange);
  widthsRef.current = widths;
  onWidthsChangeRef.current = onWidthsChange;

  const stopResize = useCallback(() => {
    dragRef.current = null;
    setResizing(null);
    document.body.classList.remove("workspace-panel-resizing");
  }, []);

  useEffect(() => {
    const handleMove = (event: PointerEvent) => {
      const drag = dragRef.current;
      if (!drag) return;
      const delta = event.clientX - drag.startX;
      if (drag.side === "left") {
        const maximum = Math.max(220, window.innerWidth - widthsRef.current.rightWidth - minimumContentWidth);
        onWidthsChangeRef.current({
          ...widthsRef.current,
          leftWidth: clamp(drag.startWidth + delta, 220, Math.min(420, maximum)),
        });
      } else {
        const maximum = Math.max(260, window.innerWidth - widthsRef.current.leftWidth - minimumContentWidth);
        onWidthsChangeRef.current({
          ...widthsRef.current,
          rightWidth: clamp(drag.startWidth - delta, 260, Math.min(480, maximum)),
        });
      }
    };
    const handleUp = () => stopResize();
    window.addEventListener("pointermove", handleMove);
    window.addEventListener("pointerup", handleUp);
    window.addEventListener("pointercancel", handleUp);
    return () => {
      window.removeEventListener("pointermove", handleMove);
      window.removeEventListener("pointerup", handleUp);
      window.removeEventListener("pointercancel", handleUp);
      document.body.classList.remove("workspace-panel-resizing");
    };
  }, [stopResize]);

  const beginResize = useCallback((side: PanelSide, event: ReactPointerEvent) => {
    if (event.button !== 0) return;
    event.preventDefault();
    dragRef.current = {
      side,
      startX: event.clientX,
      startWidth: side === "left" ? widthsRef.current.leftWidth : widthsRef.current.rightWidth,
    };
    setResizing(side);
    document.body.classList.add("workspace-panel-resizing");
  }, []);

  return {
    leftWidth: widths.leftWidth,
    rightWidth: widths.rightWidth,
    resizing,
    beginLeftResize: (event: ReactPointerEvent) => beginResize("left", event),
    beginRightResize: (event: ReactPointerEvent) => beginResize("right", event),
  };
}

function clamp(value: number, minimum: number, maximum: number) {
  return Math.min(Math.max(value, minimum), maximum);
}
