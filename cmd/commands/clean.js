const fs = require('fs');
const path = require('path');
const { success, error } = require('../utils/logger');

module.exports = function cleanCommand() {
  const projectDir = process.cwd();
  const buildDir = path.join(projectDir, 'nula', 'bin');
  if (fs.existsSync(buildDir)) {
    fs.rmSync(buildDir, { recursive: true, force: true });
    success('Build artifacts cleaned.');
  } else {
    warn('No build artifacts to clean.');
  }
};
