export async function check() {
  const delay = Number(sessionStorage.getItem("gsm-e2e-update-delay")) || 0;
  if (delay > 0) await new Promise(resolve => setTimeout(resolve, delay));
  if (sessionStorage.getItem("gsm-e2e-update-error")) throw new Error("Network unavailable");
  return null;
}

export type Update = {
  version: string;
  downloadAndInstall: () => Promise<void>;
};
