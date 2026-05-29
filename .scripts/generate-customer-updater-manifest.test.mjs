import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdir, mkdtemp, readFile, rm, writeFile } from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import { generateManifest } from './generate-customer-updater-manifest.mjs';

test('generates customer updater manifest from signed release artifacts', async () => {
  const root = await mkdtemp(path.join(os.tmpdir(), 'customer-updater-manifest-'));
  try {
    const input = path.join(root, 'release-assets');
    const windows = path.join(input, 'customer-windows-x64');
    const linux = path.join(input, 'customer-linux-x64');
    const admin = path.join(input, 'admin-windows-x64');
    await mkdir(windows, { recursive: true });
    await mkdir(linux, { recursive: true });
    await mkdir(admin, { recursive: true });

    await writeFile(path.join(windows, 'setup.exe'), 'windows installer');
    await writeFile(path.join(windows, 'setup.exe.sig'), 'windows-signature\n');
    await writeFile(path.join(linux, 'app.AppImage'), 'linux installer');
    await writeFile(path.join(linux, 'app.AppImage.sig'), 'linux-signature\n');
    await writeFile(path.join(admin, 'admin.exe'), 'admin installer');
    await writeFile(path.join(admin, 'admin.exe.sig'), 'admin-signature\n');

    const assetsDir = path.join(root, 'release-upload');
    const out = path.join(assetsDir, 'customer-latest.json');
    const manifest = await generateManifest({
      input,
      assetsDir,
      out,
      repo: 'owner/repo',
      tag: 'v0.1.1',
      version: 'v0.1.1',
      notes: 'Fixture notes',
      pubDate: '2026-05-28T00:00:00Z',
    });

    assert.equal(manifest.version, '0.1.1');
    assert.equal(manifest.notes, 'Fixture notes');
    assert.equal(manifest.platforms['windows-x86_64'].signature, 'windows-signature');
    assert.equal(manifest.platforms['linux-x86_64'].signature, 'linux-signature');
    assert.equal(manifest.platforms['windows-x86_64'].url, 'https://github.com/owner/repo/releases/download/v0.1.1/windows-x86_64-setup.exe');
    assert.equal(Object.hasOwn(manifest.platforms, 'admin-windows-x86_64'), false);

    const written = JSON.parse(await readFile(out, 'utf8'));
    assert.equal(written.platforms['linux-x86_64'].url, 'https://github.com/owner/repo/releases/download/v0.1.1/linux-x86_64-app.AppImage');
  } finally {
    await rm(root, { recursive: true, force: true });
  }
});

test('fails when no signed customer artifacts exist', async () => {
  const root = await mkdtemp(path.join(os.tmpdir(), 'customer-updater-manifest-empty-'));
  try {
    const input = path.join(root, 'release-assets');
    await mkdir(path.join(input, 'customer-windows-x64'), { recursive: true });
    await writeFile(path.join(input, 'customer-windows-x64', 'setup.exe'), 'unsigned');

    await assert.rejects(
      () =>
        generateManifest({
          input,
          assetsDir: path.join(root, 'release-upload'),
          out: path.join(root, 'release-upload', 'customer-latest.json'),
          repo: 'owner/repo',
          tag: 'v0.1.1',
          version: '0.1.1',
        }),
      /No signed customer updater artifacts/,
    );
  } finally {
    await rm(root, { recursive: true, force: true });
  }
});
