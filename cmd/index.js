// src/index.js - Main entry point for Nula CLI
// Run with: node src/index.js [command] [args]
// Modularized structure

const { program } = require('commander');
const chalk = require('chalk');
const path = require('path');
const os = require('os');
const fs = require('fs');

const initCommand = require('./commands/init');
const runCommand = require('./commands/run');
const buildCommand = require('./commands/build');
const installCommand = require('./commands/install');
const listCommand = require('./commands/list');

const { log, error, success, info } = require('./utils/logger');
const { getNulaDir, ensureDirs } = require('./utils/dirUtils');

// Setup directories
const homeDir = os.homedir();
const nulaDir = getNulaDir(homeDir);
ensureDirs(nulaDir);

// Setup CLI with commander
program
  .name('nula')
  .description(chalk.cyan('Nula CLI - Future-proof programming language'))
  .version('1.0.0');

program
  .command('init')
  .description(chalk.yellow('Initialize a new Nula project'))
  .action(() => {
    initCommand();
    success('Project initialized successfully!');
  });

program
  .command('run')
  .description(chalk.yellow('Run a .nula file in dev mode'))
  .argument('<file>', 'Path to .nula file')
  .action((file) => {
    info(`Running ${file}...`);
    runCommand(file);
  });

program
  .command('build')
  .description(chalk.yellow('Build production binary'))
  .requiredOption('--platform <platform>', 'Target platform (linux, windows, macos)')
  .argument('<file>', 'Path to .nula file')
  .action((file, options) => {
    info(`Building for ${options.platform}...`);
    buildCommand(options.platform, file);
    success('Build completed!');
  });

program
  .command('install')
  .description(chalk.yellow('Install a package'))
  .argument('<package>', 'Package name')
  .action((pkg) => {
    info(`Installing ${pkg}...`);
    installCommand(pkg);
    success(`${pkg} installed!`);
  });

program
  .command('list')
  .description(chalk.yellow('List installed packages'))
  .action(() => {
    listCommand();
  });

program.parse();
