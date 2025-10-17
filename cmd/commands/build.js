// src/commands/build.js
const { invokeBinary } = require('../utils/invoke');
const { error } = require('../utils/logger');

module.exports = function buildCommand(platform, file) {
  const validPlatforms = ['linux', 'windows', 'macos'];
  if (!validPlatforms.includes(platform)) {
    error('Invalid platform. Supported: ' + validPlatforms.join(', '));
    process.exit(1);
  }
  if (!file.endsWith('.nula')) {
    error('File must be .nula');
    process.exit(1);
  }
  invokeBinary('nula-compiler', ['--platform', platform, file]);
};
