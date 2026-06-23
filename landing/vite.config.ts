import { copyFileSync, existsSync, mkdirSync, writeFileSync } from "node:fs";
import { resolve } from "node:path";
import { defineConfig, type Plugin } from "vite";

const base = process.env.VITE_BASE_PATH ?? "/";
const siteUrl = (process.env.VITE_SITE_URL ?? "http://localhost").replace(/\/$/, "");

function rewriteEnRoute(url: string): string | null {
  const parsed = new URL(url, "http://localhost");
  if (parsed.pathname === "/en" || parsed.pathname === "/en/") {
    parsed.pathname = "/en.html";
    return `${parsed.pathname}${parsed.search}`;
  }
  const baseTrim = base.replace(/\/$/, "");
  if (baseTrim && (parsed.pathname === `${baseTrim}/en` || parsed.pathname === `${baseTrim}/en/`)) {
    parsed.pathname = `${baseTrim}/en.html`;
    return `${parsed.pathname}${parsed.search}`;
  }
  return null;
}

function enRoutePlugin(): Plugin {
  return {
    name: "en-route",
    configureServer(server) {
      server.middlewares.use((req, _res, next) => {
        if (!req.url) {
          next();
          return;
        }
        const rewritten = rewriteEnRoute(req.url);
        if (rewritten) {
          req.url = rewritten;
        }
        next();
      });
    },
    closeBundle() {
      const outDir = resolve(__dirname, "dist");
      const enHtml = resolve(outDir, "en.html");
      if (!existsSync(enHtml)) {
        return;
      }
      const enDir = resolve(outDir, "en");
      mkdirSync(enDir, { recursive: true });
      copyFileSync(enHtml, resolve(enDir, "index.html"));
    },
  };
}

function injectYandexMetrika(): Plugin {
  const id = process.env.VITE_YANDEX_METRIKA_ID?.trim();
  if (!id || !/^\d+$/.test(id)) {
    return { name: "inject-yandex-metrika" };
  }

  const snippet = `<!-- Yandex.Metrika counter -->
    <script type="text/javascript">
      (function (m, e, t, r, i, k, a) {
        m[i] =
          m[i] ||
          function () {
            (m[i].a = m[i].a || []).push(arguments);
          };
        m[i].l = 1 * new Date();
        for (var j = 0; j < document.scripts.length; j++) {
          if (document.scripts[j].src === r) {
            return;
          }
        }
        (k = e.createElement(t)),
          (a = e.getElementsByTagName(t)[0]),
          (k.async = 1),
          (k.src = r),
          a.parentNode.insertBefore(k, a);
      })(window, document, "script", "https://mc.yandex.ru/metrika/tag.js?id=${id}", "ym");

      ym(${id}, "init", {
        ssr: true,
        webvisor: true,
        clickmap: true,
        referrer: document.referrer,
        url: location.href,
        accurateTrackBounce: true,
        trackLinks: true,
      });
    </script>
    <noscript
      ><div>
        <img
          src="https://mc.yandex.ru/watch/${id}"
          style="position: absolute; left: -9999px"
          alt=""
        /></div
    ></noscript>
    <!-- /Yandex.Metrika counter -->`;

  return {
    name: "inject-yandex-metrika",
    transformIndexHtml(html) {
      return html.replace("<head>", `<head>\n${snippet}`);
    },
  };
}

function injectSiteMeta(): Plugin {
  return {
    name: "inject-site-meta",
    transformIndexHtml(html, ctx) {
      const isEn = ctx.filename?.includes("en.html");
      const pathSuffix = isEn ? "/en/" : "/";
      const canonical = `${siteUrl}${pathSuffix === "/" ? "" : pathSuffix}`.replace(
        /([^:]\/)\/+/g,
        "$1",
      );
      const pageUrl = canonical.endsWith("/") ? canonical : `${canonical}/`;

      return html
        .replace(/http:\/\/localhost\/?/g, `${siteUrl}/`)
        .replace(/http:\/\/localhost\/en\//g, `${siteUrl}/en/`)
        .replace(
          /<link rel="canonical" href="[^"]*"/,
          `<link rel="canonical" href="${pageUrl}"`,
        )
        .replace(
          /<meta property="og:url" content="[^"]*"/,
          `<meta property="og:url" content="${pageUrl}"`,
        );
    },
  };
}

function emitSeoFiles(): Plugin {
  return {
    name: "emit-seo-files",
    closeBundle() {
      const outDir = resolve(__dirname, "dist");
      const root = siteUrl.replace(/\/$/, "");
      writeFileSync(
        resolve(outDir, "sitemap.xml"),
        `<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
  <url><loc>${root}/</loc><changefreq>weekly</changefreq><priority>1.0</priority></url>
  <url><loc>${root}/en/</loc><changefreq>weekly</changefreq><priority>0.9</priority></url>
</urlset>
`,
      );
      writeFileSync(
        resolve(outDir, "robots.txt"),
        `User-agent: *\nAllow: /\n\nSitemap: ${root}/sitemap.xml\n`,
      );
    },
  };
}

export default defineConfig({
  base,
  appType: "mpa",
  plugins: [enRoutePlugin(), injectYandexMetrika(), injectSiteMeta(), emitSeoFiles()],
  build: {
    outDir: "dist",
    emptyOutDir: true,
    rollupOptions: {
      input: {
        main: resolve(__dirname, "index.html"),
        en: resolve(__dirname, "en.html"),
      },
    },
  },
});
