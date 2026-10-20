import { test, expect } from "@playwright/test";

for (const language of ["ru", "en"] as const) {
  test(`INI file drafts and preview in ${language}`, async ({ page }) => {
    await page.setViewportSize({ width: 1200, height: 600 });
    await page.addInitScript(
      (language) => sessionStorage.setItem("gsm-e2e-language", language),
      language,
    );
    await page.goto("/e2e.html");
    await page
      .getByRole("button", {
        name: language === "ru" ? "Выбрать" : "Select",
        exact: true,
      })
      .click();
    await page.evaluate(() =>
      document.documentElement.style.setProperty("--app-font-scale", "1.25"),
    );
    const files = page.getByRole("tab", {
      name: language === "ru" ? "Файлы" : "Files",
      exact: true,
    });
    await files.click();
    const editor = page.getByRole("textbox", {
      name: language === "ru" ? "Текст INI-файла" : "INI file text",
      exact: true,
    });
    const select = page.getByRole("combobox", {
      name: language === "ru" ? "Файл конфигурации" : "Configuration file",
    });
    const save = page.getByRole("button", {
      name:
        language === "ru" ? "Предпросмотр и сохранение" : "Preview and save",
    });
    await expect(editor).toHaveValue(/r.Test=1/);
    await expect(save).toBeDisabled();
    await editor.focus();
    await editor.press("Control+Home");
    await editor.press("ArrowRight");
    await editor.press("X");
    await editor.press("Y");
    await expect(editor).toHaveValue(/^;XY Example config/);
    await editor.fill("; engine draft\n[SystemSettings]\nr.Test=2\n");
    await select.selectOption("Input.ini");
    await expect(editor).toHaveValue(/r.Test=1/);
    await editor.fill(
      "; input draft\n[Input]\n+ActionMappings=One\n+ActionMappings=Two\n",
    );
    await page
      .getByRole("tab", {
        name: language === "ru" ? "Базовое" : "Basic",
        exact: true,
      })
      .click();
    await files.click();
    await expect(editor).toHaveValue(/input draft/);
    await page.screenshot({ path: `test-results/ini-editor-${language}.png` });
    await editor.press("Control+s");
    const dialog = page.getByRole("dialog", {
      name: language === "ru" ? "Подтверждение изменений" : "Confirm changes",
    });
    await expect(dialog).toBeVisible();
    await page.screenshot({ path: `test-results/ini-preview-${language}.png` });
    await dialog
      .getByRole("checkbox", { name: "Input.ini: *", exact: true })
      .uncheck();
    await dialog
      .getByRole("checkbox", { name: "Engine.ini: *", exact: true })
      .uncheck();
    const confirm = dialog.getByRole("button", {
      name: language === "ru" ? "Подтвердить и применить" : "Confirm and apply",
      exact: true,
    });
    await expect(confirm).toBeDisabled();
    await dialog
      .getByRole("checkbox", { name: "Engine.ini: *", exact: true })
      .check();
    await dialog
      .getByRole("checkbox", {
        name:
          language === "ru"
            ? "Я ознакомился с предупреждениями"
            : "I have reviewed the warnings",
      })
      .check();
    await confirm.click();
    await expect(dialog).not.toBeVisible();
    await expect(editor).toHaveValue(/input draft/);
    await expect(save).toBeEnabled();
    await select.selectOption("Engine.ini");
    await expect(editor).toHaveValue(/engine draft/);
    await select.selectOption("Game.ini");
    await expect(editor).toHaveValue("");
    await expect(
      page.getByText(
        language === "ru" ? /Файл отсутствует/ : /This file is missing/,
      ),
    ).toBeVisible();
    const areaBox = await editor.boundingBox();
    expect(areaBox!.height).toBeGreaterThan(100);
    const box = await save.boundingBox();
    expect(box!.y + box!.height).toBeLessThanOrEqual(600);
    expect(
      await page.evaluate(
        () => document.documentElement.scrollWidth <= window.innerWidth,
      ),
    ).toBe(true);
    await page.screenshot({ path: `test-results/ini-text-${language}.png` });
  });
}
