const { invokeBinary } = require('../utils/invoke');
const { error } = require('../utils/logger');

module.exports = function runCommand(file, debug) {
  if (!file.endsWith('.nula')) {
    error('File must end with .nula');
    throw new Error('Invalid file');
  }
  const args = ['run', file];
  if (debug) {
    args.push('--debug');
  }
  invokeBinary('nula-backend', args);
};
