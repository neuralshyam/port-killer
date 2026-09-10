#!/usr/bin/env node

const fs = require('fs');
const path = require('path');
const https = require('https');
const { execSync } = require('child_process');

const REPO = 'neuralshyam/kport';
const VERSION = 'v0.2.0';

function getTarget() {
  const os = process.platform;
  const arch = process.arch;

  let platformName = '';
  if (os === 'linux') {
    platformName = 'unknown-linux-gnu';
  } else if (os === 'darwin') {
    platformName = 'apple-darwin';
  } else if (os === 'win32') {
    platformName = 'pc-windows-msvc';
  } else {
    throw new Error(`Unsupported OS platform: ${os}`);
  }

  let archName = '';
  if (arch === 'x64') {
    archName = 'x86_64';
  } else if (arch === 'arm64') {
    archName = 'aarch64';
  } else {
    throw new Error(`Unsupported architecture: ${arch}`);
  }

  return `${archName}-${platformName}`;
}

function download(url, dest) {
  return new Promise((resolve, reject) => {
    const file = fs.createWriteStream(dest);
    const get = (targetUrl) => {
      https.get(targetUrl, (res) => {
        if (res.statusCode >= 300 && res.statusCode < 400 && res.headers.location) {
          get(res.headers.location);
          return;
        }
        if (res.statusCode !== 200) {
          file.close();
          fs.unlink(dest, () => {});
          return reject(new Error(`HTTP ${res.statusCode}: ${res.statusMessage}`));
        }
        res.pipe(file);
        file.on('finish', () => {
          file.close(resolve);
        });
      }).on('error', (err) => {
        file.close();
        fs.unlink(dest, () => {});
        reject(err);
      });
    };
    get(url);
  });
}

async function downloadBinary() {
  try {
    const target = getTarget();
    const isWin = process.platform === 'win32';
    const ext = isWin ? 'zip' : 'tar.gz';
    const filename = `port-killer-${target}.${ext}`;
    const url = `https://github.com/${REPO}/releases/download/${VERSION}/${filename}`;
    const binDir = path.join(__dirname);
    const binPath = path.join(binDir, isWin ? 'kport-bin.exe' : 'kport-bin');
    const archivePath = path.join(binDir, filename);

    if (fs.existsSync(binPath)) {
      return;
    }

    console.log(`⚡ Downloading kport binary for ${target}...`);
    await download(url, archivePath);

    if (isWin) {
      execSync(`powershell -command "Expand-Archive -Path '${archivePath}' -DestinationPath '${binDir}' -Force"`);
      const extractedBin = path.join(binDir, 'port-killer.exe');
      if (fs.existsSync(extractedBin)) {
        fs.renameSync(extractedBin, binPath);
      }
    } else {
      execSync(`tar -xzf "${archivePath}" -C "${binDir}"`);
      const extractedBin = path.join(binDir, 'port-killer');
      if (fs.existsSync(extractedBin)) {
        fs.renameSync(extractedBin, binPath);
      }
    }

    if (fs.existsSync(archivePath)) {
      fs.unlinkSync(archivePath);
    }

    if (fs.existsSync(binPath)) {
      fs.chmodSync(binPath, 0o755);
      console.log('✓ kport binary ready!');
    }
  } catch (err) {
    console.warn(`[kport] Note: ${err.message}. Will try local build or PATH fallback.`);
  }
}

module.exports = { downloadBinary };

if (require.main === module) {
  downloadBinary();
}

