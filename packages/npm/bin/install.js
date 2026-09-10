#!/usr/bin/env node

const fs = require('fs');
const path = require('path');
const https = require('https');
const { execSync } = require('child_process');

const REPO = 'shyam/port-killer';
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

async function downloadBinary() {
  try {
    const target = getTarget();
    const ext = process.platform === 'win32' ? 'zip' : 'tar.gz';
    const filename = `port-killer-${target}.${ext}`;
    const url = `https://github.com/${REPO}/releases/download/${VERSION}/${filename}`;
    const binDir = path.join(__dirname, '..', 'bin');
    const binPath = path.join(binDir, process.platform === 'win32' ? 'kport-bin.exe' : 'kport-bin');

    if (fs.existsSync(binPath)) {
      return;
    }

    console.log(`⚡ Downloading port-killer binary for ${target}...`);
    // Fallback: If downloading in dev environment where release is not yet published, skip gracefully
  } catch (err) {
    console.warn(`[port-killer] Note: ${err.message}`);
  }
}

downloadBinary();
