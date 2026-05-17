#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SemverChange {
    Patch,
    Minor,
    Major,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompatibilityInfo {
    pub contract_version: &'static str,
    pub supported_min: &'static str,
    pub supported_max_exclusive: &'static str,
}

impl CompatibilityInfo {
    pub fn current() -> Self {
        Self {
            contract_version: "1.0.0",
            supported_min: "1.0.0",
            supported_max_exclusive: "2.0.0",
        }
    }

    pub fn is_change_allowed(change: SemverChange, is_breaking: bool) -> Result<(), &'static str> {
        match (change, is_breaking) {
            (SemverChange::Patch, true) => {
                Err("breaking changes are not allowed in patch releases")
            }
            (SemverChange::Minor, true) => {
                Err("breaking changes are not allowed in minor releases")
            }
            _ => Ok(()),
        }
    }
}
