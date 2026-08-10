import { useMemo } from "react";
import type { AnnotationRect } from "../../annotations";
import type { PdfPageStructure, PdfTextSpan } from "../types/pdfStructure";
import { registerTextRunGeometry } from "../utils/selectionGeometry";

type TextRun = {
  text: string;
  bounds: PdfTextSpan["bounds"];
  fontSize: number;
  characters: { text: string; rect: AnnotationRect }[];
};

export function PdfTextLayer({ structure, displayScale }: { structure: PdfPageStructure; displayScale: number }) {
  const runs = useMemo(() => buildTextRuns(structure), [structure]);
  return <div className="textLayer mupdf-text-layer">
    {runs.map((run, index) => (
      <span key={index} className="mupdf-text-span mupdf-text-run"
        ref={(element) => { if (element) registerTextRunGeometry(element, run.characters); }}
        style={{
          left: run.bounds.left * displayScale,
          top: run.bounds.top * displayScale,
          width: (run.bounds.right - run.bounds.left) * displayScale,
          height: (run.bounds.bottom - run.bounds.top) * displayScale,
          fontSize: Math.max(1, run.fontSize * displayScale),
        }}>{run.text}</span>
    ))}
  </div>;
}

function buildTextRuns(structure: PdfPageStructure): TextRun[] {
  return structure.blocks.flatMap((block) => block.lines).flatMap((line) => {
    const runs: TextRun[] = [];
    let current: PdfTextSpan[] = [];
    const flush = () => {
      if (!current.length) return;
      const left = Math.min(...current.map((character) => character.bounds.left));
      const top = Math.min(...current.map((character) => character.bounds.top));
      const right = Math.max(...current.map((character) => character.bounds.right));
      const bottom = Math.max(...current.map((character) => character.bounds.bottom));
      runs.push({
        text: current.map((character) => character.text).join(""),
        bounds: { left, top, right, bottom },
        fontSize: Math.max(...current.map((character) => character.fontSize)),
        characters: current.map((character) => ({
          text: character.text,
          rect: {
            left: character.bounds.left / structure.width,
            top: character.bounds.top / structure.height,
            width: (character.bounds.right - character.bounds.left) / structure.width,
            height: (character.bounds.bottom - character.bounds.top) / structure.height,
          },
        })),
      });
      current = [];
    };

    for (const character of line.spans) {
      const previous = current[current.length - 1];
      const gap = previous ? character.bounds.left - previous.bounds.right : 0;
      const fontDifference = previous ? Math.abs(character.fontSize - previous.fontSize) : 0;
      const mustSplit = current.length >= 8
        || (previous && gap > Math.max(3, previous.fontSize * 0.75))
        || (previous && fontDifference > Math.max(1, previous.fontSize * 0.2));
      if (mustSplit) flush();
      current.push(character);
    }
    flush();
    return runs;
  });
}
