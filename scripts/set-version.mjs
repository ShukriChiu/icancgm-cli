#!/usr/bin/env node

import { readFile, writeFile } from "node:fs/promises";
import { resolve } from "node:path";

const version = process.argv[2];

if (!version || !/^\d+\.\d+\.\d+([-.][0-9A-Za-z.-]+)?$/.test(version)) {
  console.error("Usage: node scripts/set-version.mjs <semver>");
  process.exit(1);
}

const rootDir = resolve(import.meta.dirname, "..");
const cargoTomlPath = resolve(rootDir, "Cargo.toml");
const packageJsonPath = resolve(rootDir, "packages", "sino-cli", "package.json");

const cargoToml = await readFile(cargoTomlPath, "utf8");
const currentCargoVersionMatch = cargoToml.match(
  /\[workspace\.package\][\s\S]*?version\s*=\s*"([^"]+)"/,
);
if (!currentCargoVersionMatch) {
  console.error("Failed to read Cargo.toml workspace version");
  process.exit(1);
}

const currentCargoVersion = currentCargoVersionMatch[1];
const nextCargoToml = cargoToml.replace(
  /(\[workspace\.package\][\s\S]*?version\s*=\s*")([^"]+)(")/,
  `$1${version}$3`,
);

if (currentCargoVersion === version) {
  console.log(`Version is already ${version}, nothing to update`);
  process.exit(0);
}

const packageJson = JSON.parse(await readFile(packageJsonPath, "utf8"));
packageJson.version = version;

await writeFile(cargoTomlPath, nextCargoToml);
await writeFile(packageJsonPath, `${JSON.stringify(packageJson, null, 2)}\n`);

console.log(`Updated workspace and npm package version to ${version}`);
