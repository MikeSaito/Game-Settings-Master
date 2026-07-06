export type ValidationSeverity = "error" | "warning";

export type ValidationIssueCode =
  | "version_mismatch"
  | "family_mismatch"
  | "value_out_of_range"
  | "sg_exceeds_limit"
  | "sg_limits_pending"
  | "sg_r_conflict"
  | "shipped_key_removal"
  | "combo_rt_shadows"
  | "combo_engine_scalability_dup"
  | "combo_rt_no_hw"
  | "combo_streaming_texture"
  | "combo_gpu_pending"
  | "combo_rt_gpu_unknown";

export interface ValidationIssue {
  code: ValidationIssueCode;
  severity: ValidationSeverity;
  key?: string;
  i18nKey: string;
  i18nParams?: Record<string, string | number>;
}
