import { useEffect, useState } from "react";

const key = "bambook.document-favorites.v1";
const changed = "bambook:document-favorites-changed";
const normalize = (path: string) => path.replace(/\\/g, "/");

function readFavorites(): string[] {
  try {
    const value: unknown = JSON.parse(localStorage.getItem(key) ?? "[]");
    return Array.isArray(value) ? value.filter((item): item is string => typeof item === "string") : [];
  } catch { return []; }
}

export function useDocumentFavorites() {
  const [paths, setPaths] = useState(readFavorites);
  useEffect(() => {
    const refresh = () => setPaths(readFavorites());
    window.addEventListener(changed, refresh);
    window.addEventListener("storage", refresh);
    return () => { window.removeEventListener(changed, refresh); window.removeEventListener("storage", refresh); };
  }, []);
  return {
    paths,
    isFavorite: (path: string) => paths.includes(normalize(path)),
    toggle: (path: string) => {
      if (!path) return;
      const current = readFavorites();
      const normalized = normalize(path);
      const next = current.includes(normalized) ? current.filter((item) => item !== normalized) : [...current, normalized];
      localStorage.setItem(key, JSON.stringify(next));
      setPaths(next);
      window.dispatchEvent(new Event(changed));
    },
  };
}
