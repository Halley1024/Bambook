import { open, save } from "@tauri-apps/plugin-dialog";

export type FileDialogFilter = {
  name: string;
  extensions: string[];
};

export async function selectFilePath(filters: FileDialogFilter[]) {
  const selected = await open({
    multiple: false,
    filters,
  });
  return typeof selected === "string" ? selected : null;
}

export async function selectDirectoryPath() {
  const selected = await open({ directory: true, multiple: false });
  return typeof selected === "string" ? selected : null;
}

export async function selectSavePath(options: {
  defaultPath: string;
  filters: FileDialogFilter[];
  title?: string;
}) {
  const selected = await save(options);
  return typeof selected === "string" ? selected : null;
}
