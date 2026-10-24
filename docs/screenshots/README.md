# README screenshots

The English and Russian galleries are captured from the current application in
`e2e.html`, using isolated demonstration data. They do not access installed games
or modify game configuration files. The gallery assets live here independently
of the landing page screenshots.

Run `npm run screenshots:readme` from the repository root after installing the
project dependencies, Playwright Chromium, and Python with Pillow. The capture
script starts and stops its own local preview server on port 4178.

Each language has six lossless WebP images: the library, basic editor, advanced
editor, INI text editor, change preview, and backups. Captures use a 1920 by 1080
viewport at a device scale factor of 1.25, producing 2400 by 1350 images. The
clock is fixed, motion is reduced, fonts are loaded before capture, and rendering
errors cause the command to fail. Game names and configuration values are
examples, not verified compatibility profiles or tuning recommendations.

After regenerating, inspect both galleries and verify the image links in
`README.md` and `README.ru.md` before committing the assets.
