import { useContext } from "react";
import { SettingsContext } from "../SettingsProvider";

export function useSettings() {
  const context = useContext(SettingsContext);
  if (!context) throw new Error("useSettings 必须在 SettingsProvider 中使用");
  return context;
}
