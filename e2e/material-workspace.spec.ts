import { test, expect } from "@playwright/test";

for (const language of ["en", "ru"] as const) {
  for (const theme of ["dark", "light"] as const) {
    test(`Material workspace, keyboard and theme in ${language}/${theme}`, async ({
      page,
    }) => {
      const errors: string[] = [];
      page.on("pageerror", (error) => errors.push(error.message));
      const label = (en: string, ru: string) => (language === "ru" ? ru : en);
      await page.addInitScript(
        (language) => sessionStorage.setItem("gsm-e2e-language", language),
        language,
      );
      await page.setViewportSize({ width: 1280, height: 720 });
      await page.emulateMedia({ reducedMotion: "reduce" });
      await page.goto("/e2e.html");
      await expect(
        page
          .getByRole("link", { name: "Game Settings Master", exact: true })
          .locator("img"),
      ).toBeVisible();
      if (theme === "light")
        await page
          .getByRole("button", { name: label("Light", "Светлая"), exact: true })
          .click();
      await expect(page.locator("html")).toHaveAttribute("data-theme", theme);
      const settings = JSON.parse(
        (await page.evaluate(() => localStorage.getItem("gsm-app-settings"))) ??
          "{}",
      );
      expect(settings.theme).toBe(theme);
      await page
        .getByRole("button", { name: label("Select", "Выбрать"), exact: true })
        .click();
      const firstRow = page.getByTestId("parameter-row").first();
      await expect(firstRow).toBeVisible();
      await firstRow.hover();
      await expect(page.getByRole("dialog")).toHaveCount(0);
      const info = firstRow.getByRole("button", {
        name: /Details for|Сведения о параметре/,
      });
      await info.focus();
      await page.keyboard.press("Enter");
      const detail = page.getByRole("dialog");
      await expect(detail).toBeVisible();
      await expect(page.getByTestId("parameter-list-details")).toContainText(
        "GameUserSettings.ini",
      );
      // Native dialogs prevent focus from reaching background controls.
      // Chromium may expose browser chrome focus as document.body between cycles.
      for (let index = 0; index < 5; index++) {
        await page.keyboard.press("Tab");
        expect(
          await page.evaluate(
            () =>
              document.activeElement === document.body ||
              !!document.activeElement?.closest("dialog[open]"),
          ),
        ).toBe(true);
      }
      await page.keyboard.press("Escape");
      await expect(detail).not.toBeVisible();
      await expect(info).toBeFocused();
      const gameInfo = page.getByRole("button", {
        name: label("Game details", "Сведения об игре"),
        exact: true,
      });
      await gameInfo.click();
      await expect(page.getByRole("dialog")).toContainText("Saved\\Config");
      await page.keyboard.press("Escape");
      await expect(gameInfo).toBeFocused();
      const basic = page.getByRole("tab", {
        name: label("Basic", "Базовое"),
        exact: true,
      });
      await basic.focus();
      await page.keyboard.press("ArrowRight");
      await expect(
        page.getByRole("tab", {
          name: label("Advanced", "Расширенное"),
          exact: true,
        }),
      ).toBeFocused();
      await page.keyboard.press("End");
      await expect(
        page.getByRole("tab", {
          name: label("Backups", "Бэкапы"),
          exact: true,
        }),
      ).toBeFocused();
      await page.keyboard.press("Home");
      await expect(basic).toBeFocused();
      for (const [width, height, scale] of [
        [1440, 900, 1],
        [1200, 750, 1.25],
        [1200, 600, 1.25],
      ] as const) {
        await page.setViewportSize({ width, height });
        await page.evaluate(
          (scale) =>
            document.documentElement.style.setProperty(
              "--app-font-scale",
              String(scale),
            ),
          scale,
        );
        for (const name of [
          label("Basic", "Базовое"),
          label("Controls", "Управление"),
          label("Files", "Файлы"),
          label("Presets", "Пресеты"),
          label("Backups", "Бэкапы"),
        ]) {
          await page.getByRole("tab", { name, exact: true }).click();
          expect(
            await page.evaluate(
              () => document.documentElement.scrollWidth <= innerWidth,
            ),
          ).toBe(true);
        }
        await basic.click();
        await expect(
          page.getByRole("button", {
            name: label(
              "Apply to GameUserSettings",
              "Применить в пользовательские настройки",
            ),
            exact: true,
          }),
        ).toBeInViewport({ ratio: 1 });
      }
      await page.screenshot({
        path: `test-results/material-${language}-${theme}.png`,
      });
      expect(errors).toEqual([]);
    });
  }
}

test("diagnostic reapply opens above game details and restores modal focus", async ({
  page,
}) => {
  await page.addInitScript(() =>
    sessionStorage.setItem("gsm-e2e-diagnostic", "changed"),
  );
  await page.goto("/e2e.html");
  await page.getByRole("button", { name: "Select", exact: true }).click();
  // A changed diagnostic remains visible without opening game details.
  await expect(page.getByText("bUseVSync: True → False")).toBeVisible();
  await page.getByRole("button", { name: "Game details", exact: true }).click();
  const gameInfo = page.getByRole("dialog", {
    name: "Game details",
    exact: true,
  });
  const reapply = gameInfo.getByRole("button", {
    name: "Reapply settings",
    exact: true,
  });
  await reapply.click();
  const preview = page.getByRole("dialog", {
    name: "Confirm changes",
    exact: true,
  });
  await expect(preview).toBeVisible();
  await preview.getByRole("checkbox").uncheck();
  await expect(
    preview.getByRole("button", { name: "Confirm and apply" }),
  ).toBeDisabled();
  await page.keyboard.press("Escape");
  await expect(preview).not.toBeVisible();
  await expect(gameInfo).toBeVisible();
  await expect(reapply).toBeFocused();
  await reapply.click();
  await expect(preview).toBeVisible();
  await preview
    .getByRole("button", { name: "Confirm and apply", exact: true })
    .click();
  await expect(preview).not.toBeVisible();
  await expect(gameInfo).toBeVisible();
  await expect(reapply).toBeFocused();
  await page.keyboard.press("Escape");
  await expect(gameInfo).not.toBeVisible();
});
