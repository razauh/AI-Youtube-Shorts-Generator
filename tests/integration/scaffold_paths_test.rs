use std::path::Path;

#[test]
fn required_scaffold_paths_exist() {
    let required_paths = [
        "app/package.json",
        "app/src-tauri/Cargo.toml",
        "app/src-tauri/src/main.rs",
        "app/src-tauri/tauri.conf.json",
        "app/src",
        "python_legacy",
        "tests/integration",
    ];

    for path in required_paths {
        assert!(
            Path::new(path).exists(),
            "required scaffold path missing: {}",
            path
        );
    }
}
