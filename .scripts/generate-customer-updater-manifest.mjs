#!/usr/bin/env node
import { copyFile, mkdir, readdir, readFile, stat, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const TARGETS = new Map([
  ['linux', 'linux'],
  ['windows', 'windows'],
  ['macos', 'darwin'],
  ['darwin', 'darwin'],
]);

const ARCHES = new Map([
  ['x64', 'x86_64'],
  ['x86_64', 'x86_64'],
  ['amd64', 'x86_64'],
  ['aarch64', 'aarch64'],
  ['arm64', 'aarch64'],
  ['i686', 'i686'],
  ['armv7', 'armv7'],
]);

const INSTALLER_PRIORITY = [
  '.exe',
  '.msi',
  '.appimage',
  '.dmg',
  '.app.tar.gz',
  '.deb',
  '.rpm',
  '.tar.gz',
];

export async function generateManifest(options) {
  const input = requiredPath(options.input, '--input');
  const assetsDir = requiredPath(options.assetsDir, '--assets-dir');
  const out = requiredPath(options.out, '--out');
  const repo = requiredValue(options.repo, '--repo');
  const tag = requiredValue(options.tag, '--tag');
  const version = normalizeVersion(requiredValue(options.version, '--version'));
  const notes = options.notes || `Customer app update ${version}`;
  const pubDate = options.pubDate || new Date().toISOString();

  const files = await walk(input);
  const signedInstallers = [];
  for (const sigPath of files.filter((file) => file.endsWith('.sig'))) {
    const installerPath = sigPath.slice(0, -4);
    if (!files.includes(installerPath) || isIgnoredInstaller(installerPath)) {
      continue;
    }
    const platformKey = platformKeyFromPath(input, installerPath);
    if (!platformKey) {
      continue;
    }
    signedInstallers.push({
      installerPath,
      sigPath,
      platformKey,
      score: installerScore(installerPath),
    });
  }

  const selected = new Map();
  for (const item of signedInstallers) {
    const existing = selected.get(item.platformKey);
    if (!existing || item.score > existing.score || (item.score === existing.score && item.installerPath < existing.installerPath)) {
      selected.set(item.platformKey, item);
    }
  }

  if (selected.size === 0) {
    throw new Error('No signed customer updater artifacts were found.');
  }

  await mkdir(assetsDir, { recursive: true });
  const platforms = {};
  for (const item of Array.from(selected.values()).sort((a, b) => a.platformKey.localeCompare(b.platformKey))) {
    const assetName = `${item.platformKey}-${sanitizeAssetName(path.basename(item.installerPath))}`;
    const sigAssetName = `${assetName}.sig`;
    await copyFile(item.installerPath, path.join(assetsDir, assetName));
    await copyFile(item.sigPath, path.join(assetsDir, sigAssetName));
    platforms[item.platformKey] = {
      url: `https://github.com/${repo}/releases/download/${tag}/${assetName}`,
      signature: (await readFile(item.sigPath, 'utf8')).trim(),
    };
  }

  const manifest = {
    version,
    notes,
    pub_date: pubDate,
    platforms,
  };

  await mkdir(path.dirname(out), { recursive: true });
  await writeFile(out, `${JSON.stringify(manifest, null, 2)}\n`);
  return manifest;
}

function parseArgs(argv) {
  const args = {};
  for (let i = 0; i < argv.length; i += 1) {
    const current = argv[i];
    if (!current.startsWith('--')) {
      throw new Error(`Unexpected argument: ${current}`);
    }
    const key = current.slice(2).replace(/-([a-z])/g, (_, char) => char.toUpperCase());
    const value = argv[i + 1];
    if (!value || value.startsWith('--')) {
      throw new Error(`Missing value for ${current}`);
    }
    args[key] = value;
    i += 1;
  }
  return args;
}

function requiredPath(value, name) {
  return path.resolve(requiredValue(value, name));
}

function requiredValue(value, name) {
  if (!value || !String(value).trim()) {
    throw new Error(`${name} is required.`);
  }
  return String(value).trim();
}

function normalizeVersion(value) {
  return String(value).trim().replace(/^v/i, '');
}

async function walk(root) {
  const entries = await readdir(root);
  const files = [];
  for (const entry of entries) {
    const fullPath = path.join(root, entry);
    const info = await stat(fullPath);
    if (info.isDirectory()) {
      files.push(...await walk(fullPath));
    } else if (info.isFile()) {
      files.push(fullPath);
    }
  }
  return files;
}

function platformKeyFromPath(input, installerPath) {
  const relative = path.relative(input, installerPath).toLowerCase().split(path.sep);
  const customerSegment = relative.find((part) => part.startsWith('customer-'));
  if (!customerSegment) {
    return null;
  }
  const parts = customerSegment.split('-').slice(1);
  const target = TARGETS.get(parts[0]);
  const arch = ARCHES.get(parts.slice(1).join('-')) || ARCHES.get(parts.at(-1));
  if (!target || !arch) {
    return null;
  }
  return `${target}-${arch}`;
}

function isIgnoredInstaller(filePath) {
  const lower = filePath.toLowerCase();
  return lower.endsWith('.sha256') || lower.endsWith('.sha256.txt') || lower.endsWith('.sig');
}

function installerScore(filePath) {
  const lower = filePath.toLowerCase();
  const index = INSTALLER_PRIORITY.findIndex((suffix) => lower.endsWith(suffix));
  return index === -1 ? 0 : INSTALLER_PRIORITY.length - index;
}

function sanitizeAssetName(name) {
  return name.replace(/[^A-Za-z0-9._-]/g, '-');
}

const isCli = process.argv[1] && fileURLToPath(import.meta.url) === path.resolve(process.argv[1]);
if (isCli) {
  generateManifest(parseArgs(process.argv.slice(2))).catch((error) => {
    console.error(error instanceof Error ? error.message : String(error));
    process.exit(1);
  });
}
