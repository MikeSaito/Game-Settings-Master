//! Shared, data-defined semantic rules; filesystem and UI state stay outside.
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Policy {
    version: u32,
    decimal_pattern: String,
    truthy: Vec<String>,
    rules: Vec<Rule>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Rule {
    code: String,
    severity: String,
    key: Option<String>,
    changed_any: Vec<String>,
    all: Vec<Clause>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Clause {
    any_truthy: Option<Vec<String>>,
    rt_status: Option<String>,
    key: Option<String>,
    lte: Option<f64>,
    gt: Option<f64>,
}
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct SharedIssue {
    pub code: String,
    pub severity: String,
    pub key: Option<String>,
}
fn policy() -> &'static Policy {
    static POLICY: OnceLock<Policy> = OnceLock::new();
    POLICY.get_or_init(|| {
        let policy: Policy =
            serde_json::from_str(include_str!("../../shared/validation-rules.json"))
                .expect("shared validation policy");
        assert_eq!(policy.version, 1);
        policy
    })
}
pub(crate) fn finite_decimal(raw: &str) -> Option<f64> {
    static DECIMAL: OnceLock<regex::Regex> = OnceLock::new();
    let raw = raw.trim();
    if !DECIMAL
        .get_or_init(|| regex::Regex::new(&policy().decimal_pattern).unwrap())
        .is_match(raw)
    {
        return None;
    }
    raw.parse::<f64>().ok().filter(|n| n.is_finite())
}
pub(crate) fn evaluate(
    values: &HashMap<String, String>,
    changed: &HashSet<String>,
    rt_status: &str,
) -> Vec<SharedIssue> {
    policy()
        .rules
        .iter()
        .filter(|rule| {
            rule.changed_any.iter().any(|key| changed.contains(key))
                && rule.all.iter().all(|clause| {
                    if let Some(keys) = &clause.any_truthy {
                        return keys.iter().any(|key| {
                            values.get(key).is_some_and(|value| {
                                policy().truthy.contains(&value.trim().to_ascii_lowercase())
                            })
                        });
                    }
                    if let Some(status) = &clause.rt_status {
                        return status == rt_status;
                    }
                    let Some(value) = clause
                        .key
                        .as_ref()
                        .and_then(|k| values.get(k))
                        .and_then(|v| finite_decimal(v))
                    else {
                        return false;
                    };
                    clause.lte.is_none_or(|max| value <= max)
                        && clause.gt.is_none_or(|min| value > min)
                })
        })
        .map(|r| SharedIssue {
            code: r.code.clone(),
            severity: r.severity.clone(),
            key: r.key.clone(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Cases {
        combo_cases: Vec<Case>,
        number_cases: Vec<NumberCase>,
    }
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Case {
        name: String,
        values: HashMap<String, String>,
        changed_keys: HashSet<String>,
        rt_status: String,
        expected: Vec<SharedIssue>,
    }
    #[derive(Deserialize)]
    struct NumberCase {
        raw: String,
        expected: Option<f64>,
    }
    #[test]
    fn shared_scenarios() {
        let cases: Cases =
            serde_json::from_str(include_str!("../../shared/validation-cases.json")).unwrap();
        for case in cases.combo_cases {
            assert_eq!(
                evaluate(&case.values, &case.changed_keys, &case.rt_status),
                case.expected,
                "{}",
                case.name
            );
        }
        for case in cases.number_cases {
            assert_eq!(finite_decimal(&case.raw), case.expected, "{}", case.raw);
        }
    }
}
