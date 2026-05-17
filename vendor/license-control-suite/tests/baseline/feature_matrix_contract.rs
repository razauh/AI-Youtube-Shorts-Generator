use std::fs;
use std::path::Path;

#[test]
fn feature_manifest_declares_desktop_first_split() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let cargo_toml = fs::read_to_string(root.join("Cargo.toml")).expect("Cargo.toml should exist");

    assert!(cargo_toml.contains("[features]"));
    assert!(cargo_toml.contains("default = [\"core\", \"desktop-tauri\", \"desktop-persistence\", \"reference-worker\"]"));
    assert!(cargo_toml.contains("core = []"));
    assert!(cargo_toml.contains("desktop-tauri = ["));
    assert!(cargo_toml.contains("desktop-persistence = ["));
    assert!(cargo_toml.contains("reference-worker = ["));
    assert!(cargo_toml.contains("tauri = { version = \"2\", features = [], optional = true }"));
    assert!(cargo_toml.contains("keyring = { version = \"3\", optional = true }"));
    assert!(cargo_toml.contains("reqwest = { version = \"0.12\", default-features = false, features = [\"json\", \"rustls-tls\"], optional = true }"));
}

#[test]
fn feature_gating_is_encoded_in_curated_exports_and_modules() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let lib_rs = fs::read_to_string(root.join("src/lib.rs")).expect("src/lib.rs should exist");
    let desktop_mod =
        fs::read_to_string(root.join("src/desktop/mod.rs")).expect("src/desktop/mod.rs should exist");
    let user_reg_mod = fs::read_to_string(root.join("src/modules/user_reg/mod.rs"))
        .expect("src/modules/user_reg/mod.rs should exist");
    let tauri_mod = fs::read_to_string(root.join("src/modules/user_reg/auth_licensing_tauri/mod.rs"))
        .expect("auth_licensing_tauri mod should exist");

    assert!(lib_rs.contains("#[cfg(feature = \"core\")]"));
    assert!(lib_rs.contains("#[cfg(any(feature = \"desktop-tauri\", feature = \"desktop-persistence\"))]"));
    assert!(desktop_mod.contains("#[cfg(feature = \"desktop-persistence\")]"));
    assert!(desktop_mod.contains("#[cfg(feature = \"desktop-tauri\")]"));
    assert!(user_reg_mod.contains("#[cfg(feature = \"core\")]"));
    assert!(user_reg_mod.contains("#[cfg(any(feature = \"desktop-tauri\", feature = \"desktop-persistence\"))]"));
    assert!(user_reg_mod.contains("#[cfg(feature = \"reference-worker\")]"));
    assert!(tauri_mod.contains("#[cfg(feature = \"desktop-tauri\")]"));
    assert!(tauri_mod.contains("#[cfg(feature = \"desktop-persistence\")]"));
}

#[test]
fn binary_and_docs_describe_feature_gated_desktop_surfaces() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let cargo_toml = fs::read_to_string(root.join("Cargo.toml")).expect("Cargo.toml should exist");
    let main_rs = fs::read_to_string(root.join("src/main.rs")).expect("src/main.rs should exist");
    let readme = fs::read_to_string(root.join("README.md")).expect("README.md should exist");

    assert!(cargo_toml.contains("required-features = [\"desktop-tauri\"]"));
    assert!(main_rs.contains("#[cfg(feature = \"desktop-tauri\")]"));
    assert!(readme.contains("Feature Flags"));
    assert!(readme.contains("desktop-tauri"));
    assert!(readme.contains("desktop-persistence"));
    assert!(readme.contains("reference-worker"));
    assert!(readme.contains("desktop-only remains the supported runtime"));
}
