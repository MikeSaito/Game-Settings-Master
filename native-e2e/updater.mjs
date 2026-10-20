import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { readFile, mkdtemp, mkdir, writeFile } from "node:fs/promises";
import { createServer } from "node:http";
import { join, resolve } from "node:path";
import { launch, waitFor } from "./driver.mjs";

if (!process.env.CI) throw new Error("Installer update tests must run on an isolated CI runner");
const fixtures = resolve("test-results/updater-fixtures");
await mkdir(resolve(".verify"), { recursive: true });
const root = await mkdtemp(resolve(".verify/gsm-updater-"));
const destination = join(root,"installed");
const appRoot = join(root,"data");
const binary = join(destination,"ue-settings-master.exe");
const payload = await readFile(join(fixtures,"1.4.1.exe"));
const signature = (await readFile(join(fixtures,"1.4.1.exe.sig"),"utf8")).trim();
let mode = "corrupt";
const server = createServer((request,response) => {
  if (request.url === "/latest.json") {
    response.setHeader("Content-Type","application/json");
    response.end(JSON.stringify({version:"1.4.1",notes:"Native update fixture",pub_date:"2026-10-20T00:00:00Z",platforms:{"windows-x86_64":{url:"http://127.0.0.1:17891/update.exe",signature:mode==="bad-signature" ? Buffer.from("invalid minisign signature").toString("base64") : signature}}}));
  } else {
    const bytes = mode === "corrupt" ? Buffer.concat([payload,Buffer.from("tampered")]) : payload;
    response.setHeader("Content-Length",bytes.length); response.end(bytes);
  }
});
await new Promise((resolve) => server.listen(17891,"127.0.0.1",resolve));
async function installBaseline() {
  await mkdir(destination,{recursive:true});
  const child=spawn(join(fixtures,"1.4.0.exe"),["/S",`/D=${destination}`],{windowsHide:true});
  const code=await new Promise((resolve,reject)=>{child.on("error",reject);child.on("exit",resolve);});
  assert.equal(code,0,"Baseline installer must succeed");
}
let app;
try {
  await installBaseline();
  const baseline = await readFile(binary);
  for (mode of ["corrupt","bad-signature"]) {
    app=await launch(binary,{GSM_NATIVE_TEST_ROOT:appRoot},resolve(`test-results/native-updater/${mode}`));
    await app.execute("localStorage.setItem('gsm-app-settings',JSON.stringify({language:'en'}));location.reload();return true");
    await app.button("Update to 1.4.1");
    await waitFor(()=>app.execute("return document.body.innerText.includes('Update failed') || document.body.innerText.includes('install') && !!document.querySelector('button') && document.body.innerText.includes('Retry')"));
    const installed=await readFile(binary);
    assert.ok(baseline.equals(installed), "Rejected update must leave the baseline executable unchanged");
    await app.close(); app=null;
    // Rejection must leave the installed baseline executable unchanged.
    const hash=await import("node:crypto").then(({createHash})=>createHash("sha256").update(installed).digest("hex"));
    await writeFile(join(fixtures,`${mode}.sha256`),hash);
  }
  assert.equal(await readFile(join(fixtures,"corrupt.sha256"),"utf8"),await readFile(join(fixtures,"bad-signature.sha256"),"utf8"));
  mode="valid";
  app=await launch(binary,{GSM_NATIVE_TEST_ROOT:appRoot},resolve("test-results/native-updater/valid"));
  const before=await readFile(binary);
  await app.button("Update to 1.4.1").catch(()=>{});
  await waitFor(async()=>!before.equals(await readFile(binary)),60000);
  await waitFor(async()=>(await readFile(join(appRoot,"launches.txt"),"utf8")).split(/\r?\n/).includes("1.4.1"),60000);
  // Tauri's updater restarts the application; verify the replacement version from its PE metadata.
  const ps=spawn("powershell.exe",["-NoProfile","-Command","(Get-Item -LiteralPath $env:GSM_UPDATED_BINARY).VersionInfo.ProductVersion"],{env:{...process.env,GSM_UPDATED_BINARY:binary},windowsHide:true});
  const output=[];ps.stdout.on("data",b=>output.push(b));
  assert.equal(await new Promise(resolve=>ps.on("exit",resolve)),0);
  assert.match(Buffer.concat(output).toString(),/^1\.4\.1/m);
  console.log("Native updater rejected tampered packages and installed version 1.4.1");
} finally {
  if(app) await app.close();
  server.close();
  // Stop only the fixture at its exact installation path, including the updater's restarted child.
  const cleanup = spawn("powershell.exe", ["-NoProfile","-Command", "Get-Process ue-settings-master -ErrorAction SilentlyContinue | Where-Object { $_.Path -eq $env:GSM_UPDATED_BINARY } | Stop-Process -Force"], {env:{...process.env,GSM_UPDATED_BINARY:binary},windowsHide:true});
  await new Promise(resolve=>cleanup.on("exit",resolve));
}
