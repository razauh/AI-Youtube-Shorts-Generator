use std::path::Path;

#[test]
fn golden_parity_harness_files_exist() {
    let required_paths = [
        "../../tests/characterization/run_python_baseline.py",
        "../../tests/parity/compare_outputs.py",
        "../../tests/fixtures/golden/v1/api_success.json",
        "../../tests/fixtures/golden/v1/no_segments.json",
        "../../tests/fixtures/golden/v1/clip_failure.json",
        "../../tests/fixtures/golden/v1/manifest.json",
        "../../tests/fixtures/golden/v1/REFRESH_POLICY.md",
    ];

    for path in required_paths {
        assert!(
            Path::new(path).exists(),
            "missing parity harness file: {path}"
        );
    }
}
