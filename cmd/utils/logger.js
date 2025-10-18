const chalk = require('chalk');
const fs = require('fs');
const path = require('path');
const { getNulaDir } = require('./dirUtils');

function getLogFile() {
  const homeDir = require('os').homedir();
  const nulaDir = getNulaDir(homeDir);
  return path.join(nulaDir, 'nula.log');
}

module.exports.log = (message) => {
  const logFile = getLogFile();
  fs.appendFileSync(logFile, `${new Date().toISOString()} - ${message}\n`);
};

module.exports.error = (message) => {
  console.error(chalk.red.bold('ERROR: ') + chalk.red(message));
  this.log('ERROR: ' + message);
};

module.exports.success = (message) => {
  console.log(chalk.green.bold('SUCCESS: ') + chalk.green(message));
  this.log('SUCCESS: ' + message);
};

module.exports.info = (message) => {
  console.log(chalk.blue.bold('INFO: ') + chalk.blue(message));
  this.log('INFO: ' + message);
};

module.exports.warn = (message) => {
  console.log(chalk.yellow.bold('WARN: ') + chalk.yellow(message));
  this.log('WARN: ' + message);
};
