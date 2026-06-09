import express from "express";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const PUBLIC = path.join(__dirname, "public");
const PORT = Number(process.env.PORT || 8787);

const app = express();

app.use((req, res, next) => {
  res.setHeader("Access-Control-Allow-Origin", "*");
  res.setHeader("Access-Control-Allow-Methods", "GET, OPTIONS");
  res.setHeader("Access-Control-Allow-Headers", "Content-Type, If-None-Match");
  if (req.method === "OPTIONS") {
    res.sendStatus(204);
    return;
  }
  next();
});

app.use(
  express.static(PUBLIC, {
    etag: true,
    lastModified: true,
    maxAge: process.env.NODE_ENV === "production" ? "1h" : 0,
  }),
);

app.get("/health", (_req, res) => {
  res.json({ ok: true, service: "gsm-preset-server" });
});

app.listen(PORT, () => {
  console.log(`GSM preset server: http://localhost:${PORT}`);
  console.log(`Catalog: http://localhost:${PORT}/catalog.json`);
});
