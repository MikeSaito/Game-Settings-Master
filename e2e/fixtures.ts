import type { Page } from "@playwright/test";
import type { E2eFixtureMode } from "../src/e2e/parameters";

export async function useE2eFixtureMode(page: Page, mode: E2eFixtureMode): Promise<void> {
  await page.addInitScript((fixtureMode) => {
    sessionStorage.setItem("gsm-e2e-fixtures", fixtureMode);
  }, mode);
}
