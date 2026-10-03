# Landing development

The RU (`/`) and EN (`/en/`) pages share typed copy and a static HTML renderer.
Vite renders the product text, metadata and versioned Windows download links
at build time. The example editor enhances that HTML in the browser and never
calls the application's backend or writes configuration files.

The release version comes from the root `package.json`. Existing build overrides
(`VITE_APP_VERSION`, `VITE_BASE_PATH`, `VITE_SITE_URL`, `VITE_GITHUB_REPO` and
`VITE_YANDEX_METRIKA_ID`) remain supported.

From `landing/`:

```sh
npm ci
npm run dev -- --host 127.0.0.1 --port 4180
npm run build
npx playwright install chromium
npm run test:e2e
```

Playwright starts the production preview on port 4181. Tests cover both languages,
the complete example workflow, keyboard navigation, reduced motion, static content
without JavaScript, direct downloads and four viewport sizes at 100% and 125%.
Analytics requests are blocked in tests; local assets and runtime errors are checked.

## Interface illustrations

From the repository root, install the app dependencies and run:

```sh
npm run landing:capture
```

This requires Chromium from Playwright and Python with Pillow. The capture script
starts the isolated screenshot entry point and renders the application's actual
preview, snapshot and diagnostic components with anonymized RU/EN fixtures.
It exports lossless WebP fragments for desktop and mobile and regenerates
`src/site/screenshots.ts` with their intrinsic dimensions.

The screenshot entry point uses Tauri stubs only when `GSM_SCREENSHOT=1`.
It does not alter application commands or production behavior.

IBM Plex fonts are served locally. Their source and SIL Open Font License are
included in `public/fonts/`. Social sharing images in `public/` show the landing
at 1200×630; replace them when the first screen changes.
