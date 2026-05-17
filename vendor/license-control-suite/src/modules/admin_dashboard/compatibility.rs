use crate::modules::shared_contracts::versioning::CompatibilityInfo;

const SUPPORTED_MIN: &str = "1.0.0";
const SUPPORTED_MAX_EXCLUSIVE: &str = "2.0.0";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompatibilityRange {
    pub min: &'static str,
    pub max_exclusive: &'static str,
}

pub fn supported_shared_contracts_range() -> CompatibilityRange {
    CompatibilityRange {
        min: SUPPORTED_MIN,
        max_exclusive: SUPPORTED_MAX_EXCLUSIVE,
    }
}

pub fn assert_shared_contracts_compatibility() -> Result<(), String> {
    let actual = CompatibilityInfo::current();

    if !actual.contract_version.starts_with("1.") {
        return Err(format!(
            "unsupported shared-contracts version {} (supported: >= {}, < {})",
            actual.contract_version, SUPPORTED_MIN, SUPPORTED_MAX_EXCLUSIVE
        ));
    }

    Ok(())
}
