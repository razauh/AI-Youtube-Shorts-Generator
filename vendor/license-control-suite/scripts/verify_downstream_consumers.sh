#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
core_consumer="${repo_root}/fixtures/downstream_consumers/core_only_consumer"
tauri_consumer="${repo_root}/fixtures/downstream_consumers/tauri_host_consumer"
git_consumer_template="${repo_root}/fixtures/downstream_consumers/git_dependency_consumer"
tmp_dir="$(mktemp -d)"
trap 'rm -rf "${tmp_dir}"' EXIT

echo "[downstream] core-only consumer"
cargo run --manifest-path "${core_consumer}/Cargo.toml"

echo "[downstream] tauri host consumer"
cargo check --manifest-path "${tauri_consumer}/Cargo.toml"

echo "[downstream] git-dependency consumer"
repo_rev="$(git -C "${repo_root}" rev-parse HEAD)"
git clone --bare "${repo_root}" "${tmp_dir}/license-control-suite.git" >/dev/null 2>&1
repo_url="file://${tmp_dir}/license-control-suite.git"

mkdir -p "${tmp_dir}/git-consumer/src"
sed \
  -e "s|__REPO_URL__|${repo_url}|g" \
  -e "s|__REPO_REV__|${repo_rev}|g" \
  "${git_consumer_template}/Cargo.toml.template" > "${tmp_dir}/git-consumer/Cargo.toml"
cp "${git_consumer_template}/src/main.rs" "${tmp_dir}/git-consumer/src/main.rs"

cargo check --manifest-path "${tmp_dir}/git-consumer/Cargo.toml"

echo "[downstream] packaged consumer"
(
  cd "${repo_root}"
  cargo package --allow-dirty
)

crate_archive="$(find "${repo_root}/target/package" -maxdepth 1 -name 'license-control-suite-*.crate' | sort | tail -n 1)"
if [[ -z "${crate_archive}" ]]; then
  echo "[downstream] failed to locate packaged crate artifact" >&2
  exit 1
fi

tar -xzf "${crate_archive}" -C "${tmp_dir}"
packaged_root="$(find "${tmp_dir}" -maxdepth 1 -mindepth 1 -type d -name 'license-control-suite-*' | sort | tail -n 1)"

mkdir -p "${tmp_dir}/packaged-consumer/src"
cat > "${tmp_dir}/packaged-consumer/Cargo.toml" <<EOF
[package]
name = "license-control-suite-packaged-consumer"
version = "0.1.0"
edition = "2021"
publish = false

[dependencies]
license-control-suite = { path = "${packaged_root}", default-features = false, features = ["core"] }
EOF

cat > "${tmp_dir}/packaged-consumer/src/main.rs" <<'EOF'
fn main() {
    let _ = core::mem::size_of::<license_control_suite::core::AuthService>();
}
EOF

cargo check --manifest-path "${tmp_dir}/packaged-consumer/Cargo.toml"

echo "[downstream] all consumer checks passed"
