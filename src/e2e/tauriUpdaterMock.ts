export async function check() {
  return null;
}

export type Update = {
  version: string;
  downloadAndInstall: () => Promise<void>;
};
