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
without JavaScript, direct downloads and four viewport sizes at 100% and 125% in both
palettes. Additional tests cover system theme changes, stored preference, language
navigation, unchanged demo drafts during theme switches and unavailable storage.
Analytics requests are blocked in tests; local assets and runtime errors are checked.

## Interface illustrations

From the repository root, install the app dependencies and run:

```sh
npm run landing:capture
```

This requires Chromium from Playwright and Python with Pillow. The capture script
starts the isolated screenshot entry point and renders the application's actual
preview, snapshot and diagnostic components with anonymized RU/EN fixtures.
It exports lossless WebP fragments for desktop, narrow cards and mobile in both
palettes under `screenshots/<language>/<theme>/`, and regenerates
`src/site/screenshots.ts` with their intrinsic dimensions.

The screenshot entry point uses Tauri stubs only when `GSM_SCREENSHOT=1`.
It does not alter application commands or production behavior.

IBM Plex fonts are served locally. Their source and SIL Open Font License are
included in `public/fonts/`. Social sharing images in `public/` show the landing
at 1200×630; replace them when the first screen changes.

## Material design

The page follows the same warm neutral and amber color roles as the application.
Tonal surfaces, pill actions, compact tabs and rounded panels replace the former
scroll presentation. The first screen contains an independent interactive example;
subsequent sections explain preview, restore, diagnostics, INI text, presets,
hardware selection and classic input support.

The default palette follows the operating system. The header toggle stores an
explicit preference in `gsm-site-theme`. A tiny first-paint script applies it before
the page renders; the main module updates screenshot sources and theme metadata.
Use `?theme=dark` or `?theme=light` for a fixed preview. Changing a palette preserves
the example's draft and selected preview operations. Without JavaScript, the text,
direct downloads, compatibility details and FAQ remain available in the dark palette.

Both localized pages retain versioned Windows installer URLs from the root package.
No application version, release, backend API, runtime framework or hosting provider
is changed by this redesign.
