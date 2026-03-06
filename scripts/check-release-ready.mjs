#!/usr/bin/env node

import { execFileSync } from "node:child_process";
import { readFile } from "node:fs/promises";
import { resolve } from "node:path";

const rootDir = resolve(import.meta.dirname, "..");
const cargoTomlPath = resolve(rootDir, "Cargo.toml");
const packageJsonPath = resolve(rootDir, "packages", "sino-cli", "package.json");

const cargoToml = await readFile(cargoTomlPath, "utf8");
const packageJson = JSON.parse(await readFile(packageJsonPath, "utf8"));

const cargoVersionMatch = cargoToml.match(/\[workspace\.package\][\s\S]*?version\s*=\s*"([^"]+)"/);
if (!cargoVersionMatch) {
  fail("Could not find workspace.package.version in Cargo.toml");
}

const cargoVersion = cargoVersionMatch[1];
const packageVersion = packageJson.version;

if (cargoVersion !== packageVersion) {
  fail(`Version mismatch: Cargo.toml=${cargoVersion}, package.json=${packageVersion}`);
}

const remoteUrl = execGit(["remote", "get-url", "origin"]);
const remoteRepo = normalizeRepoSlug(remoteUrl);
const configuredRepo = packageJson.config?.releaseRepo;

if (!configuredRepo) {
  fail("packages/sino-cli/package.json is missing config.releaseRepo");
}

if (remoteRepo !== configuredRepo) {
  fail(`Release repo mismatch: git remote=${remoteRepo}, package config.releaseRepo=${configuredRepo}`);
}

const homepage = packageJson.homepage ?? "";
const bugsUrl = packageJson.bugs?.url ?? "";
const repositoryUrl = packageJson.repository?.url ?? "";
const expectedBaseUrl = `https://github.com/${configuredRepo}`;

for (const [field, value] of [
  ["homepage", homepage],
  ["bugs.url", bugsUrl],
  ["repository.url", repositoryUrl],
]) {
  if (!String(value).toLowerCase().includes(configuredRepo.toLowerCase())) {
    fail(`Package field ${field} does not point to ${configuredRepo}`);
  }
}

const expectedTag = `v${packageVersion}`;
const providedTag = process.argv[2] || process.env.GITHUB_REF_NAME || "";
if (providedTag && providedTag !== expectedTag) {
  fail(`Tag mismatch: expected ${expectedTag}, got ${providedTag}`);
}

console.log("Release readiness check passed");
console.log(`- version: ${packageVersion}`);
console.log(`- repo: ${configuredRepo}`);
console.log(`- tag: ${expectedTag}`);
console.log(`- homepage: ${expectedBaseUrl}`);

function execGit(args) {
  return execFileSync("git", args, { cwd: rootDir, encoding: "utf8" }).trim();
}

function normalizeRepoSlug(remoteUrl) {
  const cleaned = remoteUrl.replace(/^git\+/, "").replace(/\.git$/, "");
  const sshMatch = cleaned.match(/github\.com[:/]([^/]+\/[^/]+)$/i);
  if (sshMatch) {
    return sshMatch[1];
  }
  fail(`Unsupported origin URL format: ${remoteUrl}`);
}

function fail(message) {
  console.error(`Release readiness check failed: ${message}`);
  process.exit(1);
}
