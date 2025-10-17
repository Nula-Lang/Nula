// src/commands/install.js
const { invokeBinary } = require('../utils/invoke');

module.exports = function installCommand(pkg) {
  invokeBinary('nula-packages', ['install', pkg]);
};
