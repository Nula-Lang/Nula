const { invokeBinary } = require('../utils/invoke');
const { error } = require('../utils/logger');

module.exports = function buildCommand(platform, file, optimize) {
  const validPlatforms = ['linux', 'windows', 'macos'];
  if (!validPlatforms.includes(platform)) {
    error(`Invalid platform. Supported: ${validPlatforms.join(', ')}`);
    throw new Error('Invalid platform');
  }
  if (!file.endsWith('.nula')) {
    error('File must end with .nula');
    throw new Error('Invalid file');
  }
  const args = ['--platform', platform, file];
  if (optimize) {
    args.push('--optimize');
  }
  invokeBinary('nula-compiler', args);
};
