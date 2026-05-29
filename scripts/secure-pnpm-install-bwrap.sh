#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MODE="${1:-online}"
STORE_DIR="${ROOT_DIR}/.pnpm-store"

if ! command -v bwrap >/dev/null 2>&1; then
  echo "[secure-install-bwrap] bubblewrap is not installed. Install the OS package named bubblewrap or bwrap." >&2
  exit 127
fi

if ! command -v pnpm >/dev/null 2>&1; then
  echo "[secure-install-bwrap] pnpm is not available. Enable Corepack first: corepack enable" >&2
  exit 127
fi

if ! command -v node >/dev/null 2>&1; then
  echo "[secure-install-bwrap] node is not available." >&2
  exit 127
fi

NODE_BIN="$(command -v node)"
NODE_PREFIX="$(cd "$(dirname "${NODE_BIN}")/.." && pwd)"
PNPM_ENTRY="${NODE_PREFIX}/lib/node_modules/corepack/dist/pnpm.js"
HOST_COREPACK_HOME="${COREPACK_HOME:-${HOME}/.cache/node/corepack}"

if [[ ! -f "${PNPM_ENTRY}" ]]; then
  echo "[secure-install-bwrap] Corepack pnpm entry was not found at ${PNPM_ENTRY}." >&2
  echo "[secure-install-bwrap] Run: corepack enable && corepack prepare pnpm@11.3.0 --activate" >&2
  exit 127
fi

case "${MODE}" in
  online)
    NETWORK_ARGS=(--share-net)
    PNPM_ARGS=(install --frozen-lockfile --store-dir "${STORE_DIR}")
    ;;
  offline)
    NETWORK_ARGS=()
    PNPM_ARGS=(install --offline --frozen-lockfile --store-dir "${STORE_DIR}")
    ;;
  *)
    echo "Usage: $0 [online|offline]" >&2
    exit 2
    ;;
esac

mkdir -p "${STORE_DIR}"

echo "[secure-install-bwrap] Running pnpm ${MODE} install inside bubblewrap"
echo "[secure-install-bwrap] Repo is writable; home, cache, and temp are isolated."
echo "[secure-install-bwrap] pnpm store is persisted at ${STORE_DIR}."
echo "[secure-install-bwrap] Node/Corepack runtime is mounted read-only from ${NODE_PREFIX}."

exec bwrap \
  --die-with-parent \
  --unshare-all \
  "${NETWORK_ARGS[@]}" \
  --proc /proc \
  --dev /dev \
  --tmpfs /tmp \
  --tmpfs /home \
  --tmpfs /run \
  --dir /opt \
  --ro-bind /usr /usr \
  --ro-bind /bin /bin \
  --ro-bind /lib /lib \
  --ro-bind-try /lib64 /lib64 \
  --ro-bind "${NODE_PREFIX}" /opt/node-toolchain \
  --ro-bind-try "${HOST_COREPACK_HOME}" /opt/corepack-cache \
  --ro-bind /etc/resolv.conf /etc/resolv.conf \
  --ro-bind /etc/hosts /etc/hosts \
  --bind "${ROOT_DIR}" "${ROOT_DIR}" \
  --bind "${STORE_DIR}" "${STORE_DIR}" \
  --chdir "${ROOT_DIR}" \
  --setenv HOME /home/sandbox \
  --setenv XDG_CACHE_HOME /tmp/xdg-cache \
  --setenv XDG_CONFIG_HOME /tmp/xdg-config \
  --setenv XDG_DATA_HOME /tmp/xdg-data \
  --setenv COREPACK_HOME /opt/corepack-cache \
  --setenv PNPM_HOME /tmp/pnpm-home \
  --setenv PATH /opt/node-toolchain/bin:/usr/bin:/bin \
  --setenv npm_config_userconfig /tmp/empty-npmrc \
  /opt/node-toolchain/bin/node /opt/node-toolchain/lib/node_modules/corepack/dist/pnpm.js "${PNPM_ARGS[@]}"
