import { featureOperations, featureReport } from "./featureFixtures";
export function convertFileSrc(path: string) {
  return path;
}

export async function invoke<T>(command: string): Promise<T> {
  const responses: Record<string, unknown> = {
    validate_prepared_changes: [],
    discard_prepared_changes: null,
    compare_snapshots: featureOperations,
    get_diagnostic_report: featureReport,
  };
  if (!(command in responses))
    throw new Error(`Unsupported screenshot command: ${command}`);
  return responses[command] as T;
}
