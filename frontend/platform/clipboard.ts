export function copyText(value: string) {
  return navigator.clipboard.writeText(value);
}
