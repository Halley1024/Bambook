import { createPortal } from "react-dom";
import { useEffect, useLayoutEffect, useRef, useState } from "react";

type TooltipState = {
  target: HTMLElement;
  text: string;
  bounds: DOMRect;
};

export function GlobalTooltip() {
  const [tooltip, setTooltip] = useState<TooltipState | null>(null);
  const [position, setPosition] = useState({ left: 8, top: 8 });
  const tooltipRef = useRef<HTMLDivElement | null>(null);
  const timerRef = useRef<number | null>(null);

  useEffect(() => {
    const clearTimer = () => {
      if (timerRef.current !== null) window.clearTimeout(timerRef.current);
      timerRef.current = null;
    };
    const hide = () => { clearTimer(); setTooltip(null); };
    const findButton = (target: EventTarget | null) => target instanceof Element
      ? target.closest<HTMLElement>("[data-tooltip]")
      : null;
    const showFromTarget = (target: EventTarget | null, delay: number) => {
      const button = findButton(target);
      const rawText = button?.dataset.tooltip?.trim();
      if (!button || !rawText || button.getAttribute("aria-expanded") === "true") return;
      const text = resolveToolbarTooltip(button, rawText);
      clearTimer();
      timerRef.current = window.setTimeout(() => {
        if (!button.isConnected) return;
        setTooltip({ target: button, text, bounds: button.getBoundingClientRect() });
      }, delay);
    };
    const handlePointerOver = (event: PointerEvent) => showFromTarget(event.target, 360);
    const handlePointerOut = (event: PointerEvent) => {
      const button = findButton(event.target);
      if (!button || (event.relatedTarget instanceof Node && button.contains(event.relatedTarget))) return;
      hide();
    };
    const handleFocusIn = (event: FocusEvent) => showFromTarget(event.target, 120);
    const handleFocusOut = (event: FocusEvent) => {
      if (findButton(event.target)) hide();
    };
    document.addEventListener("pointerover", handlePointerOver);
    document.addEventListener("pointerout", handlePointerOut);
    document.addEventListener("focusin", handleFocusIn);
    document.addEventListener("focusout", handleFocusOut);
    document.addEventListener("pointerdown", hide, true);
    window.addEventListener("scroll", hide, true);
    window.addEventListener("resize", hide);
    return () => {
      clearTimer();
      document.removeEventListener("pointerover", handlePointerOver);
      document.removeEventListener("pointerout", handlePointerOut);
      document.removeEventListener("focusin", handleFocusIn);
      document.removeEventListener("focusout", handleFocusOut);
      document.removeEventListener("pointerdown", hide, true);
      window.removeEventListener("scroll", hide, true);
      window.removeEventListener("resize", hide);
    };
  }, []);

  useLayoutEffect(() => {
    const element = tooltipRef.current;
    if (!tooltip || !element) return;
    const margin = 8;
    const gap = 8;
    const width = element.offsetWidth;
    const height = element.offsetHeight;
    const left = Math.max(margin, Math.min(tooltip.bounds.left, window.innerWidth - width - margin));
    const below = tooltip.bounds.bottom + gap;
    const top = below + height <= window.innerHeight - margin
      ? below
      : Math.max(margin, tooltip.bounds.top - height - gap);
    setPosition({ left, top });
  }, [tooltip]);

  if (!tooltip) return null;
  return createPortal(
    <div className="global-button-tooltip" ref={tooltipRef} role="tooltip"
      style={{ left: position.left, top: position.top }}>
      {tooltip.text}
    </div>,
    document.body,
  );
}

function resolveToolbarTooltip(target: HTMLElement, text: string) {
  if (!target.closest(".toolbar-section-center, .toolbar-section-right") || !hasVisibleButtonText(target)) return text;
  const separator = text.indexOf("：");
  return separator >= 0 ? text.slice(separator + 1).trim() : text;
}

function hasVisibleButtonText(target: HTMLElement) {
  const walker = document.createTreeWalker(target, NodeFilter.SHOW_TEXT);
  let node = walker.nextNode();
  while (node) {
    const parent = node.parentElement;
    const text = node.textContent?.trim();
    if (text && parent && !parent.closest(".toolbar-count-badge, .document-search-count")
      && parent.getClientRects().length > 0 && getComputedStyle(parent).visibility !== "hidden") return true;
    node = walker.nextNode();
  }
  return false;
}
