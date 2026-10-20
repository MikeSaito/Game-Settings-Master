export function validateVersions({ packageVersion, tauriVersion, cargoVersion, tagName, validationOnly }) {
  if (packageVersion !== tauriVersion || packageVersion !== cargoVersion) throw new Error("Package, Cargo and Tauri versions differ");
  if (!validationOnly && !/^\d+\.\d+\.\d+$/.test(packageVersion)) throw new Error("Only stable semantic versions can be published");
  const tag = `v${packageVersion}`;
  if (tagName && tagName !== tag) throw new Error("Release tag and package version differ");
  return tag;
}
export async function checkRemoteRelease({ repo, token, tag, sha }, fetcher = fetch) {
  if (!repo || !token || !/^[a-f0-9]{40}$/.test(sha ?? "")) throw new Error("Repository, token and commit SHA are required");
  async function get(path) {
    const response = await fetcher(`https://api.github.com/repos/${repo}/${path}`, {
      headers: { Authorization: `Bearer ${token}`, Accept: "application/vnd.github+json" },
      signal: AbortSignal.timeout(30000),
    });
    if (response.status === 404) return null;
    if (!response.ok) throw new Error(`GitHub preflight failed: ${response.status}`);
    return response.json();
  }
  if (!await get("")) throw new Error("GitHub repository is inaccessible");
  if (await get(`releases/tags/${tag}`)) throw new Error(`Release ${tag} already exists; use a new version`);
  const ref = await get(`git/ref/tags/${tag}`);
  if (!ref) return false;
  let object = ref.object;
  let depth = 0;
  while (object?.type === "tag") {
    if (++depth > 16) throw new Error("Invalid annotated tag chain");
    object = (await get(`git/tags/${object.sha}`))?.object;
  }
  if (object?.type !== "commit" || object.sha !== sha) throw new Error("Existing tag points to a different commit");
  return true;
}
