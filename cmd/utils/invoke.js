const { execSync } = require('child_process');
const path = require('path');
const os = require('os');
const { getNulaDir } = require('./dirUtils');
const { error, log } = require('./logger');

module.exports.invokeBinary = function invokeBinary(binaryName, args) {
  const homeDir = os.homedir();
  const nulaDir = getNulaDir(homeDir);
  const binDir = path.join(nulaDir, 'bin');
  const ext = os.platform() === 'win32' ? '.exe' : '';
  const binaryPath = path.join(binDir, binaryName + ext);
  
  if (!fs.existsSync(binaryPath)) {
    error(`Binary ${binaryName} not found in ${binDir}. Install or build it.`);
    throw new Error('Missing binary');
  }
  
  try {
    execSync(`${binaryPath} ${args.join(' ')}`, { stdio: 'inherit' });
    log(`Executed ${binaryName} with args: ${args.join(' ')}`);
  } catch (err) {
    error(`Error executing ${binaryName}: ${err.message}`);
    log(`Error: ${err.message}`);
    throw err;
  }
};
