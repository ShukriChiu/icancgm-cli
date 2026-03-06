import { chmod, mkdir, writeFile } from "node:fs/promises";
import { existsSync } from "node:fs";
import { join } from "node:path";
import {
  __dirname,
  isLocalWorkspace,
  packageRoot,
  packagedBinaryPath,
  platformBinaryName,
} from "./shared/runtime.js";

type PackageJson = {
  name?: string;
  version: string;
  config?: {
    releaseRepo?: string;
  };
};

async function main(): Promise<void> {
  if (process.env.SINO_SKIP_DOWNLOAD === "1") {
    console.log("[sino] skipping binary download because SINO_SKIP_DOWNLOAD=1");
    return;
  }

  if (process.env.SINO_BINARY_PATH) {
    console.log("[sino] skipping binary download because SINO_BINARY_PATH is already set");
    return;
  }

  if (isLocalWorkspace(__dirname)) {
    console.log("[sino] local workspace detected, skipping release download");
    return;
  }

  const targetPath = packagedBinaryPath(__dirname);
  if (existsSync(targetPath)) {
    console.log(`[sino] binary already present at ${targetPath}`);
    return;
  }

  const pkg = await loadPackageJson();
  const repo = process.env.SINO_RELEASE_REPO || pkg.config?.releaseRepo;
  if (!repo) {
    throw new Error("missing release repository. Set SINO_RELEASE_REPO or package.json config.releaseRepo");
  }

  const version = process.env.SINO_RELEASE_VERSION || `v${pkg.version}`;
  const assetName = platformBinaryName();
  const downloadUrl = process.env.SINO_RELEASE_BASE_URL
    ? `${trimTrailingSlash(process.env.SINO_RELEASE_BASE_URL)}/${version}/${assetName}`
    : `https://github.com/${repo}/releases/download/${version}/${assetName}`;

  console.log(`[sino] downloading ${assetName} from ${downloadUrl}`);
  const response = await fetch(downloadUrl, {
    headers: {
      "User-Agent": `${pkg.name ?? "sino-cli-installer"}/${pkg.version}`,
    },
  });

  if (!response.ok) {
    throw new Error(`download failed with status ${response.status} from ${downloadUrl}`);
  }

  const buffer = Buffer.from(await response.arrayBuffer());
  const targetDir = join(packageRoot(__dirname), "bin");
  await mkdir(targetDir, { recursive: true });
  await writeFile(targetPath, buffer);

  if (process.platform !== "win32") {
    await chmod(targetPath, 0o755);
  }

  console.log(`[sino] installed binary to ${targetPath}`);
}

async function loadPackageJson(): Promise<PackageJson> {
  const path = join(packageRoot(__dirname), "package.json");
  const content = await BunCompatibleFs.readText(path);
  return JSON.parse(content) as PackageJson;
}

function trimTrailingSlash(value: string): string {
  return value.endsWith("/") ? value.slice(0, -1) : value;
}

const BunCompatibleFs = {
  async readText(path: string): Promise<string> {
    const fs = await import("node:fs/promises");
    return fs.readFile(path, "utf8");
  },
};

main().catch((error) => {
  console.error(`[sino] install failed: ${(error as Error).message}`);
  process.exit(1);
});
