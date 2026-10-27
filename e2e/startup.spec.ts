import { test, expect } from "@playwright/test";

for (const lang of ["en", "ru"] as const)
  for (const theme of ["dark", "light"] as const) {
    test(`startup bootstrap survives a missing bundle in ${lang}/${theme}`, async ({
      page,
    }) => {
      await page.addInitScript(
        ({ lang, theme }) => {
          localStorage.setItem(
            "gsm-app-settings",
            JSON.stringify({ language: lang, theme }),
          );
        },
        { lang, theme },
      );
      await page.route("**/src/main.tsx", (route) =>
        route.fulfill({
          contentType: "application/javascript",
          body: "/* Paused application bundle. */",
        }),
      );
      await page.clock.install();
      await page.goto("/");
      await expect(page.locator("html")).toHaveAttribute("data-theme", theme);
      await expect(page.locator("html")).toHaveAttribute("lang", lang);
      await expect(page.locator("[data-boot-label]")).toHaveText(
        lang === "ru" ? "Подготовка приложения" : "Preparing the app",
      );
      await expect(page.locator(".startup-progress")).toBeVisible();
      await expect(page.locator("[data-boot-retry]")).toBeHidden();
      await expect(page.locator("html")).toHaveCSS(
        "background-color",
        theme === "light" ? "rgb(255, 249, 239)" : "rgb(20, 19, 16)",
      );
      await page.clock.runFor(8100);
      await expect(page.locator("[data-boot-retry]")).toBeVisible();
    });

    test(`update check hands off to an explicit library loading stage in ${lang}/${theme}`, async ({
      page,
    }) => {
      await page.addInitScript((lang) => {
        localStorage.setItem(
          "gsm-app-settings",
          JSON.stringify({ language: lang }),
        );
        localStorage.setItem("uesm:language", lang);
        sessionStorage.setItem("gsm-e2e-language", lang);
        sessionStorage.setItem("gsm-e2e-update-delay", "2000");
        sessionStorage.setItem("gsm-e2e-scan-delay", "4000");
      }, lang);
      await page.clock.install();
      await page.goto(`/e2e.html?theme=${theme}`);
      const startup = page.getByTestId("startup-screen");
      await expect(startup).toHaveAttribute("data-stage", "updates");
      await expect(page.getByRole("status")).toContainText(
        lang === "ru" ? "Проверка обновлений" : "Checking for updates",
      );
      await page.clock.runFor(2100);
      await expect(startup).toHaveAttribute("data-stage", "library");
      await expect(startup).toContainText(
        lang === "ru" ? "Поиск ваших игр" : "Finding your games",
      );
      await expect(startup.getByRole("progressbar")).toBeVisible();
      await page.emulateMedia({ reducedMotion: "reduce" });
      await expect(startup.locator(".startup-progress > span")).toHaveCSS(
        "animation-name",
        "none",
      );
      await page.clock.runFor(4200);
      await expect(startup).toHaveCount(0);
      await expect(
        page
          .getByRole("button", {
            name: lang === "ru" ? "Выбрать" : "Select",
            exact: true,
          })
          .first(),
      ).toBeVisible();
    });
  }

test("offline startup offers recovery and still opens the library", async ({
  page,
}) => {
  await page.addInitScript(() => {
    sessionStorage.setItem("gsm-e2e-language", "en");
    sessionStorage.setItem("gsm-e2e-update-error", "1");
  });
  await page.goto("/e2e.html");
  await expect(page.getByRole("alert")).toContainText("Network unavailable");
  await page
    .getByRole("button", {
      name: "Continue without updating (offline)",
      exact: true,
    })
    .click();
  await expect(
    page.getByRole("button", { name: "Select", exact: true }).first(),
  ).toBeVisible();
});
