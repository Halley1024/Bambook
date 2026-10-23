import { ContextColorPicker } from "./ContextColorPicker";

export function BookmarkColorPicker({ value, onChange }: {
  value: string;
  onChange: (color: string) => void;
}) {
  return <ContextColorPicker value={value} onChange={onChange} scopeLabel="书签" />;
}
