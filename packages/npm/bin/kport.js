#!/usr/bin/env node

const { spawn } = require('child_process');
const path = require('path');
const fs = require('fs');

const isWin = process.platform === 'win32';
const binName = isWin ? 'kport-bin.exe' : 'kport-bin';
const binPath = path.join(__dirname, binName);

// Fallback search locations
const localDevBinary = path.join(__dirname, '..', '..', '..', 'target', 'release', isWin ? 'kport.exe' : 'kport');
const debugDevBinary = path.join(__dirname, '..', '..', '..', 'target', 'debug', isWin ? 'kport.exe' : 'kport');
const legacyDevBinary = path.join(__dirname, '..', '..', '..', 'target', 'release', isWin ? 'port-killer.exe' : 'port-killer');

async function ensureBinary() {
  if (fs.existsSync(binPath)) {
    return binPath;
  }
  if (fs.existsSync(localDevBinary)) {
    return localDevBinary;
  }
  if (fs.existsSync(debugDevBinary)) {
    return debugDevBinary;
  }
  if (fs.existsSync(legacyDevBinary)) {
    return legacyDevBinary;
  }

  // If binary wasn't fetched during postinstall (e.g. bunx/npx runtime or untrusted sandbox), fetch now
  try {
    const installScript = path.join(__dirname, 'install.js');
    if (fs.existsSync(installScript)) {
      const { downloadBinary } = require('./install.js');
      if (typeof downloadBinary === 'function') {
        await downloadBinary();
      }
    }
  } catch (_) {}

  return binPath;
}

async function main() {
  const executable = await ensureBinary();

  const child = spawn(executable, process.argv.slice(2), {
    stdio: 'inherit',
  });

  child.on('error', (err) => {
    if (err.code === 'ENOENT') {
      console.error(`\nError: kport native binary not found at ${executable}`);
      console.error('Please run "cargo install kport" or reinstall the npm package.\n');
    } else {
      console.error('Error executing kport:', err);
    }
    process.exit(1);
  });

  child.on('exit', (code, signal) => {
    if (signal) {
      process.kill(process.pid, signal);
    } else {
      process.exit(code || 0);
    }
  });
}

main();
