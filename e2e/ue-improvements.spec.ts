import { test, expect } from "@playwright/test";

test("Russian controls and preview are localized", async ({ page }) => {
  await page.addInitScript(() =>
    sessionStorage.setItem("gsm-e2e-language", "ru"),
  );
  await page.goto("/e2e.html");
  await page.getByRole("button", { name: "Выбрать", exact: true }).click();
  await page.getByRole("tab", { name: "Управление", exact: true }).click();
  await expect(
    page.getByRole("heading", { name: "Управление — Input.ini" }),
  ).toBeVisible();
  await page
    .getByRole("textbox", { name: "Jump: Клавиша", exact: true })
    .first()
    .fill("K");
  await page.getByRole("button", { name: "Применить", exact: true }).click();
  await expect(page.getByRole("dialog")).toContainText(
    "Подтверждение изменений",
  );
  await expect(page.getByRole("dialog")).toContainText("Было");
  await expect(page.getByRole("dialog")).toContainText("Станет");
  await page.screenshot({
    path: "test-results/ue-preview-ru.png",
    fullPage: true,
  });
  await page
    .getByRole("dialog")
    .getByRole("button", { name: "Отмена", exact: true })
    .click();
});

test("repeated input mappings keep excluded edits after confirmation", async ({
  page,
}) => {
  await page.goto("/e2e.html");
  await page.getByRole("button", { name: "Select", exact: true }).click();
  await page.getByRole("tab", { name: "Controls", exact: true }).click();
  const keys = page.getByRole("textbox", { name: "Jump: Key", exact: true });
  await expect(keys).toHaveCount(2);
  await keys.nth(0).fill("K");
  await keys.nth(1).fill("L");
  await page.getByRole("button", { name: "Apply", exact: true }).click();
  const dialog = page.getByRole("dialog");
  await expect(dialog.getByRole("checkbox")).toHaveCount(2);
  await dialog.getByRole("checkbox").nth(1).uncheck();
  await dialog.getByRole("button", { name: "Confirm and apply" }).click();
  await expect(keys.nth(0)).toHaveValue("K");
  await expect(keys.nth(1)).toHaveValue("L");
  await page.getByRole("button", { name: "Apply", exact: true }).click();
  await expect(dialog.getByRole("checkbox")).toHaveCount(1);
  await expect(dialog).toContainText("Key=J");
  await expect(dialog).toContainText("Key=L");
  await dialog.getByRole("button", { name: "Cancel", exact: true }).click();
});

test("GPU selection uses AMD capabilities independently of DLSS", async ({
  page,
}) => {
  await page.goto("/e2e.html");
  await page.getByRole("button", { name: "Select", exact: true }).click();
  await page.getByText("GPU used for settings", { exact: false }).click();
  await page
    .getByRole("combobox", { name: "Use", exact: true })
    .selectOption("test-amd");
  await expect(
    page.getByText("GPU used for settings · E2E AMD GPU"),
  ).toBeVisible();
  await expect(
    page.getByText("Hardware ray tracing: Supported", { exact: true }),
  ).toBeVisible();
  await expect(page.getByText(/Dedicated memory: 4096 MB/)).toBeVisible();
});

test("named snapshots can be created and renamed", async ({ page }) => {
  await page.goto("/e2e.html");
  await page.getByRole("button", { name: "Select", exact: true }).click();
  await page.getByRole("tab", { name: "Backups", exact: true }).click();
  await page
    .getByRole("textbox", { name: "Snapshot or preset name" })
    .fill("Before tuning");
  await page
    .getByRole("button", { name: "Create snapshot", exact: true })
    .click();
  const snapshot = page.getByRole("combobox", {
    name: "Snapshot",
    exact: true,
  });
  await expect(
    snapshot.getByRole("option", { name: "Before tuning" }),
  ).toHaveCount(1);
  await snapshot.selectOption({ label: "Before tuning" });
  await page
    .getByRole("textbox", { name: "Snapshot or preset name" })
    .fill("Known good");
  await page.getByRole("button", { name: "Rename", exact: true }).click();
  await expect(
    snapshot.getByRole("option", { name: "Known good" }),
  ).toHaveCount(1);
});

test("legacy preset import shows source mismatch and requires acknowledgement", async ({
  page,
}) => {
  await page.goto("/e2e.html");
  await page.getByRole("button", { name: "Select", exact: true }).click();
  await page.getByRole("tab", { name: "Presets", exact: true }).click();
  await page
    .locator('input[type="file"]')
    .setInputFiles({
      name: "legacy.json",
      mimeType: "application/json",
      buffer: Buffer.from(
        JSON.stringify({
          game_id: "another-game",
          name: "Old preset",
          files: {
            "GameUserSettings.ini": {
              ScalabilityGroups: { "sg.ShadowQuality": "2" },
            },
          },
        }),
      ),
    });
  await expect(
    page.getByText("Legacy preset has no compatibility information"),
  ).toBeVisible();
  await expect(
    page.getByText("Preset was created for another game: another-game"),
  ).toBeVisible();
  const save = page.getByRole("button", {
    name: "Save imported preset",
    exact: true,
  });
  await expect(save).toBeDisabled();
  await page
    .getByRole("checkbox", { name: "I have reviewed the warnings" })
    .check();
  await save.click();
  await expect(page.getByText("Review import of “Old preset”")).toHaveCount(0);
  await expect(page.getByText("Old preset", { exact: true })).toBeVisible();
});
