/*!
 * bytes
 * Copyright(c) 2012-2014 TJ Holowaychuk
 * Copyright(c) 2015 Jed Watson
 * MIT Licensed
 */

'use strict';

var native;
try {
  native = require('./index.node');
} catch (e) {
  throw new Error('Native binding index.node not found. Run `npm run build` or `cargo build --release`.');
}

function sanitizeOptions(options) {
  if (!options || typeof options !== 'object') return options;
  var opts = {};
  if (options.decimalPlaces !== undefined && options.decimalPlaces !== null) opts.decimalPlaces = options.decimalPlaces;
  if (options.fixedDecimals !== undefined && options.fixedDecimals !== null) opts.fixedDecimals = Boolean(options.fixedDecimals);
  if (options.thousandsSeparator !== undefined && options.thousandsSeparator !== null) opts.thousandsSeparator = String(options.thousandsSeparator);
  if (options.unit !== undefined && options.unit !== null) opts.unit = String(options.unit);
  if (options.unitSeparator !== undefined && options.unitSeparator !== null) opts.unitSeparator = String(options.unitSeparator);
  return opts;
}

function bytes(value, options) {
  return native.bytes(value, sanitizeOptions(options));
}

function format(value, options) {
  return native.format(value, sanitizeOptions(options));
}

function parse(value) {
  return native.parse(value);
}

bytes.format = format;
bytes.parse = parse;

module.exports = bytes;
module.exports.format = format;
module.exports.parse = parse;


