const path = require('path');
const fs = require('fs');
const { error } = require('./logger');

module.exports.getNulaDir = (homeDir) => path.join(homeDir, '.nula');

module.exports.ensureDirs = (nulaDir) => {
  const binDir = path.join(nulaDir, 'bin');
  const libDir = path.join(nulaDir, 'lib');
  const logFile = path.join(nulaDir, 'nula.log');

  [nulaDir, binDir, libDir].forEach(dir => {
    if (!fs.existsSync(dir)) {
      fs.mkdirSync(dir, { recursive: true });
    }
  });

  if (!fs.existsSync(logFile)) {
    fs.writeFileSync(logFile, '');
  }
};
