import { test, expect } from "@playwright/test";

test.describe("editor workflow", () => {
  test("scan → open game → apply basic → restore backup", async ({ page }) => {
    await page.goto("/e2e.html");

    await expect(page.getByRole("heading", { name: "Game library" })).toBeVisible();
    await expect(page.getByText("Test UE Game")).toBeVisible();

    await page.getByRole("button", { name: "Select" }).click();
    await expect(page.getByRole("heading", { name: "Basic settings" })).toBeVisible();

    const vsyncRow = page.getByTestId("parameter-row").filter({ hasText: "VSync" });
    await expect(vsyncRow).toBeVisible();
    const vsyncSwitch = vsyncRow.getByRole("switch");
    await vsyncSwitch.click();

    await page.getByRole("button", { name: "Apply to GameUserSettings" }).click();
    await expect(page.getByText(/Applied .* edits · backup/i)).toBeVisible();

    await page.getByRole("tab", { name: "Backups" }).click();
    await expect(page.getByRole("heading", { name: "Backup list" })).toBeVisible();
    await expect(page.getByRole("button", { name: "Restore" }).first()).toBeVisible();

    await page.getByRole("button", { name: "Restore" }).first().click();
    await expect(page.getByText(/Backup .* restored/i)).toBeVisible();
  });

  test("resolve sg/r conflict and apply from basic", async ({ page }) => {
    await page.goto("/e2e.html");
    await page.getByRole("button", { name: "Select" }).click();
    await expect(page.getByRole("heading", { name: "Basic settings" })).toBeVisible();

    await expect(page.getByText("sg.* and r.* overlap")).toBeVisible();
    await page.getByRole("button", { name: /Reset r\..*, keep sg\.ShadowQuality/i }).click();
    await expect(page.getByText(/Removed conflicting r\.\* overrides/i)).toBeVisible();

    await page.getByRole("button", { name: "Apply to GameUserSettings" }).click();
    await expect(page.getByText(/Applied .* edits · backup/i)).toBeVisible();
    await expect(page.getByText("sg.* and r.* overlap")).toHaveCount(0);
  });

  test("reset override ini from backups", async ({ page }) => {
    await page.goto("/e2e.html");
    await page.getByRole("button", { name: "Select" }).click();
    await page.getByRole("tab", { name: "Backups" }).click();

    await page.getByRole("button", { name: "Remove Engine / Scalability ini" }).click();
    await page.getByRole("button", { name: "Yes, remove override ini" }).click();
    await expect(page.getByText(/Override ini removed: Engine\.ini/i)).toBeVisible();
  });
});
