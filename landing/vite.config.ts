import { copyFileSync, existsSync, mkdirSync, writeFileSync } from "node:fs";
import { resolve } from "node:path";
import { defineConfig, type Plugin } from "vite";

const base = process.env.VITE_BASE_PATH ?? "/";
const siteUrl = (process.env.VITE_SITE_URL ?? "https://gsm-tool.com").replace(/\/$/, "");
const lastmod = new Date().toISOString().slice(0, 10);

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
        webvisor: false,
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

function injectContentSecurityPolicy(): Plugin {
  const csp = [
    "default-src 'self'",
    "script-src 'self' 'unsafe-inline' https://mc.yandex.ru",
    "style-src 'self' 'unsafe-inline' https://fonts.googleapis.com",
    "font-src 'self' https://fonts.gstatic.com",
    `img-src 'self' data: https://mc.yandex.ru ${siteUrl}`,
    "connect-src 'self' https://mc.yandex.ru",
    "frame-src https://mc.yandex.ru",
    "base-uri 'self'",
    "form-action 'self'",
  ].join("; ");

  return {
    name: "inject-csp",
    apply: "build",
    transformIndexHtml(html) {
      return html.replace(
        "<head>",
        `<head>\n    <meta http-equiv="Content-Security-Policy" content="${csp}" />`,
      );
    },
  };
}

function injectSiteMeta(): Plugin {
  const ruUrl = `${siteUrl}/`;
  const enUrl = `${siteUrl}/en/`;

  return {
    name: "inject-site-meta",
    transformIndexHtml(html, ctx) {
      const isEn = ctx.filename?.includes("en.html");
      const pageUrl = isEn ? enUrl : ruUrl;
      const ogLocale = isEn ? "en_US" : "ru_RU";

      return html
        .replaceAll("https://gsm-tool.com/", `${siteUrl}/`)
        .replaceAll("https://gsm-tool.com/en/", enUrl)
        .replace(
          /<link rel="canonical" href="[^"]*"/,
          `<link rel="canonical" href="${pageUrl}"`,
        )
        .replace(
          /<link rel="alternate" hreflang="ru" href="[^"]*"/,
          `<link rel="alternate" hreflang="ru" href="${ruUrl}"`,
        )
        .replace(
          /<link rel="alternate" hreflang="en" href="[^"]*"/,
          `<link rel="alternate" hreflang="en" href="${enUrl}"`,
        )
        .replace(
          /<link rel="alternate" hreflang="x-default" href="[^"]*"/,
          `<link rel="alternate" hreflang="x-default" href="${ruUrl}"`,
        )
        .replace(
          /<meta property="og:locale" content="[^"]*"/,
          `<meta property="og:locale" content="${ogLocale}"`,
        )
        .replace(
          /<meta property="og:url" content="[^"]*"/,
          `<meta property="og:url" content="${pageUrl}"`,
        )
        .replace(
          /<meta property="og:image" content="[^"]*"/,
          `<meta property="og:image" content="${siteUrl}/og-image.png"`,
        )
        .replace(
          /<meta name="twitter:image" content="[^"]*"/,
          `<meta name="twitter:image" content="${siteUrl}/og-image.png"`,
        )
        .replace(
          /"url": "https:\/\/gsm-tool.com\/"/,
          `"url": "${pageUrl}"`,
        );
    },
  };
}

function sitemapEntry(loc: string, priority: string): string {
  const ruUrl = `${siteUrl}/`;
  const enUrl = `${siteUrl}/en/`;
  return `  <url>
    <loc>${loc}</loc>
    <xhtml:link rel="alternate" hreflang="ru" href="${ruUrl}" />
    <xhtml:link rel="alternate" hreflang="en" href="${enUrl}" />
    <xhtml:link rel="alternate" hreflang="x-default" href="${ruUrl}" />
    <lastmod>${lastmod}</lastmod>
    <changefreq>weekly</changefreq>
    <priority>${priority}</priority>
  </url>`;
}

function emitSeoFiles(): Plugin {
  return {
    name: "emit-seo-files",
    closeBundle() {
      const outDir = resolve(__dirname, "dist");

      writeFileSync(
        resolve(outDir, "sitemap.xml"),
        `<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9"
        xmlns:xhtml="http://www.w3.org/1999/xhtml">
${sitemapEntry(`${siteUrl}/`, "1.0")}
${sitemapEntry(`${siteUrl}/en/`, "0.9")}
</urlset>
`,
      );

      writeFileSync(
        resolve(outDir, "robots.txt"),
        `User-agent: *
Allow: /

Sitemap: ${siteUrl}/sitemap.xml
`,
      );
    },
  };
}

export default defineConfig({
  base,
  appType: "mpa",
  plugins: [enRoutePlugin(), injectYandexMetrika(), injectContentSecurityPolicy(), injectSiteMeta(), emitSeoFiles()],
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
