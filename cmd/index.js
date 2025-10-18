const { program } = require('commander');
const chalk = require('chalk');
const ora = require('ora'); // For spinners
const figlet = require('figlet'); // For ASCII art
const path = require('path');
const os = require('os');
const fs = require('fs');

const initCommand = require('./commands/init');
const runCommand = require('./commands/run');
const buildCommand = require('./commands/build');
const installCommand = require('./commands/install');
const listCommand = require('./commands/list');
const newCommand = require('./commands/new');
const cleanCommand = require('./commands/clean');
const versionCommand = require('./commands/version');
const testCommand = require('./commands/test');

const { log, error, success, info, warn } = require('./utils/logger');
const { getNulaDir, ensureDirs } = require('./utils/dirUtils');

// Display banner
console.log(chalk.magenta(figlet.textSync('Nula', { horizontalLayout: 'full' })));
console.log(chalk.cyan('Future-proof, high-performance programming language CLI\n'));

// Setup directories
const homeDir = os.homedir();
const nulaDir = getNulaDir(homeDir);
ensureDirs(nulaDir);

// Setup CLI
program
  .name('nula')
  .description(chalk.cyan('Nula CLI v1.0.0'))
  .version('1.0.0', '-v, --version', 'Output the current version');

program
  .command('init')
  .description(chalk.yellow('Initialize a new Nula project'))
  .action(() => {
    const spinner = ora('Initializing project...').start();
    try {
      initCommand();
      spinner.succeed('Project initialized!');
    } catch (err) {
      spinner.fail('Initialization failed');
      error(err.message);
    }
  });

program
  .command('new')
  .description(chalk.yellow('Create a new .nula file'))
  .argument('<filename>', 'Name of the file to create')
  .action((filename) => {
    const spinner = ora(`Creating ${filename}...`).start();
    try {
      newCommand(filename);
      spinner.succeed(`${filename} created!`);
    } catch (err) {
      spinner.fail('Creation failed');
      error(err.message);
    }
  });

program
  .command('run')
  .description(chalk.yellow('Run a .nula file in dev mode'))
  .argument('<file>', 'Path to .nula file')
  .option('--debug', 'Enable debug mode')
  .action((file, options) => {
    info(`Running ${file}...`);
    const spinner = ora('Executing...').start();
    try {
      runCommand(file, options.debug);
      spinner.succeed('Execution complete!');
    } catch (err) {
      spinner.fail('Execution failed');
      error(err.message);
    }
  });

program
  .command('build')
  .description(chalk.yellow('Build production binary'))
  .requiredOption('--platform <platform>', 'Target platform (linux, windows, macos)')
  .argument('<file>', 'Path to .nula file')
  .option('--optimize', 'Enable optimizations')
  .action((file, options) => {
    info(`Building for ${options.platform}...`);
    const spinner = ora('Building...').start();
    try {
      buildCommand(options.platform, file, options.optimize);
      spinner.succeed('Build complete!');
    } catch (err) {
      spinner.fail('Build failed');
      error(err.message);
    }
  });

program
  .command('install')
  .description(chalk.yellow('Install a package'))
  .argument('<package>', 'Package name')
  .action((pkg) => {
    info(`Installing ${pkg}...`);
    const spinner = ora('Installing...').start();
    try {
      installCommand(pkg);
      spinner.succeed(`${pkg} installed!`);
    } catch (err) {
      spinner.fail('Install failed');
      error(err.message);
    }
  });

program
  .command('list')
  .description(chalk.yellow('List installed packages'))
  .action(() => {
    listCommand();
  });

program
  .command('clean')
  .description(chalk.yellow('Clean build artifacts'))
  .action(() => {
    const spinner = ora('Cleaning...').start();
    try {
      cleanCommand();
      spinner.succeed('Cleaned!');
    } catch (err) {
      spinner.fail('Clean failed');
      error(err.message);
    }
  });

program
  .command('test')
  .description(chalk.yellow('Run tests'))
  .action(() => {
    const spinner = ora('Running tests...').start();
    try {
      testCommand();
      spinner.succeed('Tests passed!');
    } catch (err) {
      spinner.fail('Tests failed');
      error(err.message);
    }
  });

program
  .command('version')
  .description(chalk.yellow('Show detailed version info'))
  .action(() => {
    versionCommand();
  });

program.parse();
