import { test } from "node:test";
import assert from "node:assert/strict";
import { validateVersions, checkRemoteRelease } from "./release-policy.mjs";
const sha = "a".repeat(40);
const options = { repo:"owner/repo", token:"test", tag:"v1.4.0", sha };
const versions = { packageVersion:"1.4.0", tauriVersion:"1.4.0", cargoVersion:"1.4.0", tagName:"v1.4.0", validationOnly:false };
function responses(items) {
  const queue = [...items];
  return async () => {
    assert.ok(queue.length, "Unexpected API call");
    const value = queue.shift();
    return new Response(typeof value === "number" ? null : JSON.stringify(value), {status:typeof value === "number" ? value : 200});
  };
}
test("release version and tag must agree", () => {
  assert.equal(validateVersions(versions),"v1.4.0");
  assert.throws(()=>validateVersions({...versions,cargoVersion:"1.3.0"}));
  assert.throws(()=>validateVersions({...versions,tagName:"v1.3.0"}));
});
test("published releases and drafts cannot be replaced", async () => {
  for(const draft of [false,true]) await assert.rejects(checkRemoteRelease(options,responses([{}, {draft}])),/already exists/);
});
test("missing release with matching tag is allowed", async () => {
  assert.equal(await checkRemoteRelease(options,responses([{},404,{object:{type:"commit",sha}}])),true);
});
test("a missing tag can be created without force", async () => {
  assert.equal(await checkRemoteRelease(options,responses([{},404,404])),false);
});

test("preflight uses canonical GitHub repository, release and ref endpoints", async () => {
  const urls = [];
  const reply = responses([{},404,404]);
  assert.equal(await checkRemoteRelease(options, async (url) => {
    urls.push(url);
    return reply();
  }), false);
  assert.deepEqual(urls, [
    "https://api.github.com/repos/owner/repo",
    "https://api.github.com/repos/owner/repo/releases/tags/v1.4.0",
    "https://api.github.com/repos/owner/repo/git/ref/tags/v1.4.0",
  ]);
});
test("annotated tags must resolve to the same commit", async () => {
  assert.equal(await checkRemoteRelease(options,responses([{},404,{object:{type:"tag",sha:"tag"}},{object:{type:"commit",sha}}])),true);
  await assert.rejects(checkRemoteRelease(options,responses([{},404,{object:{type:"commit",sha:"b".repeat(40)}}])),/different commit/);
});
test("authentication and server failures never mean release absence", async () => {
  for(const status of [401,403,429,500]) await assert.rejects(checkRemoteRelease(options,responses([{},status])),/GitHub preflight failed/);
  await assert.rejects(checkRemoteRelease(options,responses([404])),/inaccessible/);
});
