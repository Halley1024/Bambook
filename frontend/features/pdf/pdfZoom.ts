export const PDF_MIN_SCALE = 0.1;
export const PDF_MAX_SCALE = 10;
export const PDF_ZOOM_STEP = 0.1;
export const PDF_ZOOM_LEVELS = [10, 25, 50, 75, 100, 125, 150, 200, 250, 300, 400, 500, 600, 800, 1000];

export function clampPdfScale(scale: number) {
  if (!Number.isFinite(scale)) return 1;
  return Math.min(PDF_MAX_SCALE, Math.max(PDF_MIN_SCALE, scale));
}
