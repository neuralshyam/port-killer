#!/usr/bin/env node

const { spawn } = require('child_process');
const path = require('path');
const fs = require('fs');

const isWin = process.platform === 'win32';
const binName = isWin ? 'kport-bin.exe' : 'kport-bin';
const binPath = path.join(__dirname, binName);

// Fallback search locations
const localDevBinary = path.join(__dirname, '..', '..', '..', 'target', 'release', isWin ? 'port-killer.exe' : 'port-killer');
const debugDevBinary = path.join(__dirname, '..', '..', '..', 'target', 'debug', isWin ? 'port-killer.exe' : 'port-killer');

let executable = binPath;
if (!fs.existsSync(executable)) {
  if (fs.existsSync(localDevBinary)) {
    executable = localDevBinary;
  } else if (fs.existsSync(debugDevBinary)) {
    executable = debugDevBinary;
  }
}

const child = spawn(executable, process.argv.slice(2), {
  stdio: 'inherit',
});

child.on('error', (err) => {
  if (err.code === 'ENOENT') {
    console.error(`\nError: port-killer native binary not found at ${executable}`);
    console.error('Please run "cargo install port-killer" or reinstall the npm package.\n');
  } else {
    console.error('Error executing port-killer:', err);
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
