// src/commands/init.js
const fs = require('fs');
const path = require('path');
const { success, error } = require('../utils/logger');

module.exports = function initCommand() {
  const projectDir = process.cwd();
  const nulaFolder = path.join(projectDir, 'nula');
  const binFolder = path.join(nulaFolder, 'bin');
  if (!fs.existsSync(nulaFolder)) {
    fs.mkdirSync(nulaFolder, { recursive: true });
    fs.mkdirSync(binFolder);
    fs.writeFileSync(path.join(projectDir, 'main.nula'), 'write "Hello Nula"\n');
    success('Nula project initialized in ' + projectDir);
  } else {
    error('Project already initialized.');
    process.exit(1);
  }
};
