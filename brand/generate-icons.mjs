import fs from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";
import sharp from "sharp";
import pngToIco from "png-to-ico";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.join(__dirname, "..");
const LOGO_SVG = path.join(__dirname, "logo.svg");
const PNG_MASTER = path.join(ROOT, "public", "logo.png");

const TAURI_SIZES = [30, 32, 44, 71, 89, 107, 128, 142, 150, 284, 310, 512];
const TAURI_NAMES = {
  32: "32x32.png",
  128: "128x128.png",
  256: "128x128@2x.png",
  512: "icon.png",
};

/** High-res logo.png for UI, README, apple-touch-icon. */
const LOGO_PNG_SIZE = 512;

async function renderPngFromMaster(size, outPath) {
  await fs.mkdir(path.dirname(outPath), { recursive: true });
  const buf = await sharp(PNG_MASTER)
    .resize(size, size, { fit: "contain", background: { r: 0, g: 0, b: 0, alpha: 0 } })
    .png()
    .toBuffer();
  await fs.writeFile(outPath, buf);
}

async function main() {
  try {
    await fs.access(PNG_MASTER);
  } catch {
    console.error(`Missing PNG master: ${PNG_MASTER}`);
    process.exit(1);
  }

  const masterMeta = await sharp(PNG_MASTER).metadata();
  const masterSize = masterMeta.width ?? LOGO_PNG_SIZE;

  async function writeLogoPng(outPath) {
    if (masterSize >= LOGO_PNG_SIZE && path.resolve(outPath) === path.resolve(PNG_MASTER)) {
      return;
    }
    const size = Math.max(LOGO_PNG_SIZE, masterSize);
    if (size === masterSize && path.resolve(outPath) !== path.resolve(PNG_MASTER)) {
      await fs.copyFile(PNG_MASTER, outPath);
      return;
    }
    await renderPngFromMaster(size, outPath);
  }

  const tauriDir = path.join(ROOT, "src-tauri", "icons");
  const landingPublic = path.join(ROOT, "landing", "public");
  const appPublic = path.join(ROOT, "public");
  const logoSvg = await fs.readFile(LOGO_SVG, "utf8");

  for (const size of TAURI_SIZES) {
    const name = `Square${size}x${size}Logo.png`;
    await renderPngFromMaster(size, path.join(tauriDir, name));
  }
  await renderPngFromMaster(30, path.join(tauriDir, "StoreLogo.png"));

  for (const [size, name] of Object.entries(TAURI_NAMES)) {
    await renderPngFromMaster(Number(size), path.join(tauriDir, name));
  }

  const icoSizes = [16, 24, 32, 48, 64, 128, 256];
  const icoBuffers = await Promise.all(
    icoSizes.map((s) => sharp(PNG_MASTER).resize(s, s).png().toBuffer()),
  );
  const ico = await pngToIco(icoBuffers);
  await fs.writeFile(path.join(tauriDir, "icon.ico"), ico);

  await renderPngFromMaster(32, path.join(landingPublic, "favicon.png"));
  await writeLogoPng(path.join(landingPublic, "logo.png"));
  await fs.writeFile(path.join(landingPublic, "favicon.svg"), logoSvg);
  await fs.writeFile(path.join(landingPublic, "logo.svg"), logoSvg);

  await renderPngFromMaster(32, path.join(appPublic, "favicon.png"));
  await writeLogoPng(path.join(appPublic, "logo.png"));
  await fs.writeFile(path.join(appPublic, "favicon.svg"), logoSvg);
  await fs.writeFile(path.join(appPublic, "logo.svg"), logoSvg);

  const ogWidth = 1200;
  const ogHeight = 630;
  const logoSize = 280;
  const logoBuf = await sharp(PNG_MASTER).resize(logoSize, logoSize).png().toBuffer();
  await sharp({
    create: {
      width: ogWidth,
      height: ogHeight,
      channels: 4,
      background: { r: 15, g: 17, b: 21, alpha: 1 },
    },
  })
    .composite([
      { input: logoBuf, left: Math.round((ogWidth - logoSize) / 2), top: 120 },
    ])
    .png()
    .toFile(path.join(landingPublic, "og-image.png"));

  console.log("Icons generated from public/logo.png:");
  console.log("  src-tauri/icons/");
  console.log("  landing/public/favicon.png, favicon.svg, logo.png, logo.svg, og-image.png");
  console.log("  public/favicon.png, favicon.svg, logo.png, logo.svg");
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
