import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import cli from "@tauri-apps/cli";

const projectRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");

// Tauri's frontend directory contains package.json and the Vite configuration.
process.env.TAURI_APP_PATH = resolve(projectRoot, "backend");
process.env.TAURI_FRONTEND_PATH = projectRoot;
process.chdir(projectRoot);

try {
  await cli.run(
    process.argv.slice(2),
    `npm run ${process.env.npm_lifecycle_event ?? "tauri"}`,
  );
} catch (error) {
  cli.logError(error instanceof Error ? error.message : String(error));
  process.exitCode = 1;
}
