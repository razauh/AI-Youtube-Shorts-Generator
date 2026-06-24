use std::fs;
use std::path::Path;

#[test]
fn devolens_token_safety_validation_is_implemented_in_config() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    // Go up to the repository root and access the config file
    let config_path = root.join("../../app/src-tauri/src/core/config.rs");
    
    assert!(config_path.exists(), "Tauri app configuration file should exist at {:?}", config_path);
    
    let config_src = fs::read_to_string(config_path)
        .expect("Tauri app configuration source should be readable");

    // Assert that we have implemented validation logic for checking token permissions/scope
    assert!(
        config_src.contains("validate_devolens_token_permissions") || config_src.contains("check_devolens_token_safety"),
        "Tauri app configuration loader must validate Devolens token safety / scopes"
    );
}

#[test]
fn devolens_token_safety_adr_document_exists() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let adr_path = root.join("../../docs/adr/001-devolens-token-safety.md");
    
    assert!(
        adr_path.exists(),
        "Architecture Decision Record (ADR) for Devolens token safety must exist at {:?}",
        adr_path
    );
    
    let adr_content = fs::read_to_string(adr_path)
        .expect("ADR document should be readable");
        
    assert!(adr_content.contains("Devolens Token Safety"), "ADR should document Devolens Token Safety");
    assert!(adr_content.contains("Direct Tauri-to-Devolens"), "ADR should mention Direct Tauri-to-Devolens");
    assert!(adr_content.contains("Thin Backend Proxy"), "ADR should mention Thin Backend Proxy");
}
