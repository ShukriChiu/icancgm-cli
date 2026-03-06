import { existsSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

export const __filename = fileURLToPath(import.meta.url);
export const __dirname = dirname(__filename);

export function localBinaryName(): string {
  return process.platform === "win32" ? "sino.exe" : "sino";
}

export function platformBinaryName(): string {
  const suffix = process.platform === "win32" ? ".exe" : "";
  return `sino-${process.platform}-${process.arch}${suffix}`;
}

export function packagedBinaryPath(baseDir: string): string {
  return join(baseDir, "..", "..", "bin", platformBinaryName());
}

export function localDebugBinaryPath(baseDir: string): string {
  return join(baseDir, "..", "..", "..", "..", "target", "debug", localBinaryName());
}

export function localReleaseBinaryPath(baseDir: string): string {
  return join(baseDir, "..", "..", "..", "..", "target", "release", localBinaryName());
}

export function repoRootCargoToml(baseDir: string): string {
  return join(baseDir, "..", "..", "..", "..", "Cargo.toml");
}

export function isLocalWorkspace(baseDir: string): boolean {
  return existsSync(repoRootCargoToml(baseDir));
}

export function packageRoot(baseDir: string): string {
  return join(baseDir, "..", "..");
}
