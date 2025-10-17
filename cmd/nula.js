// nula.js - Expanded Main binary in JavaScript (Node.js)
// Enhanced with better error handling, more commands, platform detection, and logging.
// Users call: node nula.js [command] [args]
// Invokes binaries from ~/.nula/bin/
// Supports init project, list packages, etc.

const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');
const os = require('os');
const process = require('process');

// Directories
const homeDir = os.homedir();
const nulaDir = path.join(homeDir, '.nula');
const binDir = path.join(nulaDir, 'bin');
const libDir = path.join(nulaDir, 'lib');
const logFile = path.join(nulaDir, 'nula.log');

// Ensure directories
[ nulaDir, binDir, libDir ].forEach(dir => {
  if (!fs.existsSync(dir)) {
    fs.mkdirSync(dir, { recursive: true });
  }
});

// Simple logging
function log(message) {
  fs.appendFileSync(logFile, `${new Date().toISOString()} - ${message}\n`);
}

// Invoke binary with better error handling
function invokeBinary(binaryName, args) {
  const binaryPath = path.join(binDir, binaryName + (os.platform() === 'win32' ? '.exe' : ''));
  if (!fs.existsSync(binaryPath)) {
    console.error(`Binary ${binaryName} not found in ${binDir}`);
    log(`Missing binary: ${binaryName}`);
    process.exit(1);
  }
  try {
    execSync(`${binaryPath} ${args.join(' ')}`, { stdio: 'inherit' });
  } catch (error) {
    console.error(`Error executing ${binaryName}: ${error.message}`);
    log(`Execution error: ${binaryName} - ${error.message}`);
    process.exit(1);
  }
}

// Parse args
const args = process.argv.slice(2);
if (args.length === 0) {
  console.log('Nula CLI - Future-proof programming language');
  console.log('Usage: nula [command] [options]');
  console.log('Commands:');
  console.log('  init                - Initialize a new Nula project');
  console.log('  run <file.nula>     - Run in dev mode (interpreter)');
  console.log('  build --platform <platform> <file.nula> - Build production binary');
  console.log('  install <package>   - Install a package');
  console.log('  list                - List installed packages');
  process.exit(0);
}

const command = args[0];
switch (command) {
  case 'init': {
    const projectDir = process.cwd();
    const nulaFolder = path.join(projectDir, 'nula');
    if (!fs.existsSync(nulaFolder)) {
      fs.mkdirSync(nulaFolder);
      fs.mkdirSync(path.join(nulaFolder, 'bin'));
      fs.writeFileSync(path.join(projectDir, 'main.nula'), 'write "Hello Nula"\n');
      console.log('Nula project initialized.');
      log('Project initialized');
    } else {
      console.error('Project already initialized.');
    }
    break;
  }
  case 'run': {
    if (args.length < 2) {
      console.error('Usage: nula run <file.nula>');
      process.exit(1);
    }
    invokeBinary('nula-backend', ['run', ...args.slice(1)]);
    break;
  }
  case 'build': {
    if (args.length < 4 || args[1] !== '--platform') {
      console.error('Usage: nula build --platform <platform> <file.nula>');
      console.log('Platforms: linux, windows, macos');
      process.exit(1);
    }
    invokeBinary('nula-compiler', [...args.slice(1)]);
    break;
  }
  case 'install': {
    if (args.length < 2) {
      console.error('Usage: nula install <package>');
      process.exit(1);
    }
    invokeBinary('nula-packages', ['install', ...args.slice(1)]);
    break;
  }
  case 'list': {
    invokeBinary('nula-packages', ['list']);
    break;
  }
  default: {
    console.error(`Unknown command: ${command}`);
    process.exit(1);
  }
}
