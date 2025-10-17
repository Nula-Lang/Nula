const chalk = require('chalk');
const fs = require('fs');
const path = require('path');
const { getNulaDir } = require('./dirUtils');

function getLogFile() {
  const homeDir = require('os').homedir();
  const nulaDir = getNulaDir(homeDir);
  return path.join(nulaDir, 'nula.log');
}

module.exports.log = function log(message) {
  const logFile = getLogFile();
  fs.appendFileSync(logFile, `${new Date().toISOString()} - ${message}\n`);
};

module.exports.error = function error(message) {
  console.error(chalk.red('Error: ' + message));
  log('ERROR: ' + message);
};

module.exports.success = function success(message) {
  console.log(chalk.green(message));
  log('SUCCESS: ' + message);
};

module.exports.info = function info(message) {
  console.log(chalk.blue(message));
  log('INFO: ' + message);
};
