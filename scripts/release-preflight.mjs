import { readFileSync } from "node:fs";
import { execFileSync } from "node:child_process";
import { validateVersions, checkRemoteRelease } from "./release-policy.mjs";
const validationOnly = process.env.GSM_VALIDATION_ONLY === "true";
const tag = validateVersions({
  packageVersion: JSON.parse(readFileSync("package.json", "utf8")).version,
  tauriVersion: JSON.parse(readFileSync("src-tauri/tauri.conf.json", "utf8")).version,
  cargoVersion: readFileSync("src-tauri/Cargo.toml", "utf8").match(/^version\s*=\s*"([^"]+)"/m)?.[1],
  tagName: process.env.GITHUB_REF_TYPE === "tag" ? process.env.GITHUB_REF_NAME : undefined,
  validationOnly,
});
if (!validationOnly) {
  const exists = await checkRemoteRelease({repo:process.env.GITHUB_REPOSITORY,token:process.env.GH_TOKEN,tag,sha:process.env.GITHUB_SHA});
  if (!exists && process.argv.includes("--create-tag")) {
    execFileSync("git", ["tag", tag, process.env.GITHUB_SHA], { stdio: "inherit" });
    execFileSync("git", ["push", "origin", `refs/tags/${tag}`], { stdio: "inherit" });
  }
}
console.log(`Release preflight passed for ${tag}`);
