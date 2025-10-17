// src/commands/run.js
const { invokeBinary } = require('../utils/invoke');
const { error } = require('../utils/logger');

module.exports = function runCommand(file) {
  if (!file.endsWith('.nula')) {
    error('File must be .nula');
    process.exit(1);
  }
  invokeBinary('nula-backend', ['run', file]);
};
