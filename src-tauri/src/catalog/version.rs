use super::types::ReferenceEntry;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct UeSemver {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

pub fn parse_ue_semver(raw: &str) -> Option<UeSemver> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    let mut parts = trimmed.split('.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next().unwrap_or("0").parse().ok()?;
    let patch = parts.next().unwrap_or("0").parse().ok()?;
    Some(UeSemver {
        major,
        minor,
        patch,
    })
}

fn parse_version_label(label: &str) -> Option<UeSemver> {
    parse_ue_semver(label)
}

pub(crate) fn reference_applies_to_version(
    entry: &ReferenceEntry,
    game_version: Option<UeSemver>,
    is_ue4: bool,
) -> bool {
    if let Some(gv) = game_version {
        if let Some(intro) = entry.introduced_in.as_deref() {
            if let Some(intro_v) = parse_version_label(intro) {
                if gv < intro_v {
                    return false;
                }
            }
        } else if !entry.versions_present.is_empty() {
            let applicable = entry.versions_present.iter().any(|label| {
                parse_version_label(label)
                    .is_some_and(|snap| gv.major == snap.major && gv.minor == snap.minor)
            });
            if !applicable {
                return false;
            }
        }
        if let Some(removed) = entry.removed_in.as_deref() {
            if let Some(removed_v) = parse_version_label(removed) {
                if gv >= removed_v {
                    return false;
                }
            }
        }
        return true;
    }
    if is_ue4 {
        entry.ue4
    } else {
        entry.ue5
    }
}

pub(crate) fn pick_reference_default(
    reference: &ReferenceEntry,
    game_version: Option<UeSemver>,
) -> String {
    if let Some(gv) = game_version {
        for label in [
            format!("{}.{}.{}", gv.major, gv.minor, gv.patch),
            format!("{}.{}", gv.major, gv.minor),
        ] {
            if let Some(value) = reference.defaults_by_version.get(&label) {
                return value.clone();
            }
        }
        if let Some((_, value)) = reference
            .defaults_by_version
            .iter()
            .filter(|(label, _)| {
                parse_version_label(label).is_some_and(|snap| snap <= gv)
            })
            .max_by(|(a, _), (b, _)| {
                parse_version_label(a)
                    .unwrap_or(UeSemver {
                        major: 0,
                        minor: 0,
                        patch: 0,
                    })
                    .cmp(&parse_version_label(b).unwrap_or(UeSemver {
                        major: 0,
                        minor: 0,
                        patch: 0,
                    }))
            })
        {
            return value.clone();
        }
    }
    reference
        .defaults_by_version
        .get("5.4")
        .or_else(|| reference.defaults_by_version.get("4.27"))
        .or_else(|| reference.defaults_by_version.values().next())
        .cloned()
        .unwrap_or_else(|| "1".to_string())
}
