const chalk = require('chalk');
const os = require('os');
const { info } = require('../utils/logger');

module.exports = function versionCommand() {
  info(chalk.green('Nula CLI Version: 0.0.1'));
  info(chalk.green(`Platform: ${os.platform()}`));
  info(chalk.green(`Node Version: ${process.version}`));
};
