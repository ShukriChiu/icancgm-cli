#!/usr/bin/env node

import { spawnSync } from "node:child_process";
import { existsSync } from "node:fs";
import {
  __dirname,
  localDebugBinaryPath,
  localReleaseBinaryPath,
  packagedBinaryPath,
} from "../shared/runtime.js";

function main(): void {
  const binaryPath = resolveBinaryPath();
  const result = spawnSync(binaryPath, process.argv.slice(2), {
    stdio: "inherit",
  });

  if (result.error) {
    console.error(`[sino] failed to start binary: ${result.error.message}`);
    process.exit(1);
  }

  process.exit(result.status ?? 0);
}

function resolveBinaryPath(): string {
  if (process.env.SINO_BINARY_PATH && existsSync(process.env.SINO_BINARY_PATH)) {
    return process.env.SINO_BINARY_PATH;
  }

  const packagedBinary = packagedBinaryPath(__dirname);
  if (existsSync(packagedBinary)) {
    return packagedBinary;
  }

  const localDebugBinary = localDebugBinaryPath(__dirname);
  if (existsSync(localDebugBinary)) {
    return localDebugBinary;
  }

  const localReleaseBinary = localReleaseBinaryPath(__dirname);
  if (existsSync(localReleaseBinary)) {
    return localReleaseBinary;
  }

  console.error("[sino] binary not found.");
  console.error("[sino] set SINO_BINARY_PATH, reinstall the package, or build the Rust CLI first.");
  process.exit(1);
}

main();
