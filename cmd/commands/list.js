// src/commands/list.js
const { invokeBinary } = require('../utils/invoke');

module.exports = function listCommand() {
  invokeBinary('nula-packages', ['list']);
};
