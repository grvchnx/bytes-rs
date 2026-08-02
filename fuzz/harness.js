'use strict';

const fs = require('fs');
const path = require('path');
const native = require('../index.node');

// Reference JavaScript implementation for differential fuzzing
function jsFormat(value, options) {
  if (!Number.isFinite(value)) return null;
  var map = { b: 1, kb: 1024, mb: 1048576, gb: 1073741824, tb: 1099511627776, pb: 1125899906842624 };
  var mag = Math.abs(value);
  var thousandsSeparator = (options && options.thousandsSeparator) || '';
  var unitSeparator = (options && options.unitSeparator) || '';
  var decimalPlaces = (options && options.decimalPlaces !== undefined && options.decimalPlaces !== null) ? options.decimalPlaces : 2;
  var fixedDecimals = Boolean(options && options.fixedDecimals);
  var unit = (options && options.unit) || '';

  if (!unit || !map[unit.toLowerCase()]) {
    if (mag >= map.pb) unit = 'PB';
    else if (mag >= map.tb) unit = 'TB';
    else if (mag >= map.gb) unit = 'GB';
    else if (mag >= map.mb) unit = 'MB';
    else if (mag >= map.kb) unit = 'KB';
    else unit = 'B';
  }

  var val = value / map[unit.toLowerCase()];
  var str = val.toFixed(decimalPlaces);

  if (!fixedDecimals) {
    str = str.replace(/(?:\.0*|(\.[^0]+)0+)$/, '$1');
  }

  if (thousandsSeparator) {
    str = str.split('.').map(function (s, i) {
      return i === 0 ? s.replace(/\B(?=(\d{3})+(?!\d))/g, thousandsSeparator) : s;
    }).join('.');
  }

  return str + unitSeparator + unit;
}

function jsParse(val) {
  if (typeof val === 'number' && !isNaN(val)) return val;
  if (typeof val !== 'string') return null;
  var map = { b: 1, kb: 1024, mb: 1048576, gb: 1073741824, tb: 1099511627776, pb: 1125899906842624 };
  var results = /^((-|\+)?(\d+(?:\.\d+)?)) *(kb|mb|gb|tb|pb)$/i.exec(val);
  var floatValue;
  var unit = 'b';

  if (!results) {
    floatValue = parseInt(val, 10);
    unit = 'b';
  } else {
    floatValue = parseFloat(results[1]);
    unit = results[4].toLowerCase();
  }

  if (isNaN(floatValue)) return null;
  return Math.floor(map[unit] * floatValue);
}

function jsBytes(val, options) {
  if (typeof val === 'string') return jsParse(val);
  if (typeof val === 'number') return jsFormat(val, options);
  return null;
}

// Random generators
function randomChoice(arr) {
  return arr[Math.floor(Math.random() * arr.length)];
}

function randomString() {
  const units = ['b', 'kb', 'mb', 'gb', 'tb', 'pb', 'B', 'KB', 'MB', 'GB', 'TB', 'PB', ''];
  const nums = ['1', '0.5', '1.5', '-10', '+100', '1024', '999999', '0', '0.0001', 'xyz', '0x11'];
  const spaces = ['', ' ', '   ', '\t'];
  return randomChoice(spaces) + randomChoice(nums) + randomChoice(spaces) + randomChoice(units);
}

function randomOptions() {
  if (Math.random() < 0.2) return undefined;
  return {
    decimalPlaces: randomChoice([undefined, null, 0, 1, 2, 3, 4, 10]),
    fixedDecimals: randomChoice([undefined, null, true, false]),
    thousandsSeparator: randomChoice([undefined, null, '', ' ', ',', '.', '_']),
    unit: randomChoice([undefined, null, '', 'B', 'KB', 'MB', 'GB', 'TB', 'PB', 'invalid']),
    unitSeparator: randomChoice([undefined, null, '', ' ', '\t', '_'])
  };
}

function sanitizeForNative(opts) {
  if (!opts || typeof opts !== 'object') return opts;
  var cleaned = {};
  if (opts.decimalPlaces !== undefined && opts.decimalPlaces !== null) cleaned.decimalPlaces = opts.decimalPlaces;
  if (opts.fixedDecimals !== undefined && opts.fixedDecimals !== null) cleaned.fixedDecimals = Boolean(opts.fixedDecimals);
  if (opts.thousandsSeparator !== undefined && opts.thousandsSeparator !== null) cleaned.thousandsSeparator = String(opts.thousandsSeparator);
  if (opts.unit !== undefined && opts.unit !== null) cleaned.unit = String(opts.unit);
  if (opts.unitSeparator !== undefined && opts.unitSeparator !== null) cleaned.unitSeparator = String(opts.unitSeparator);
  return cleaned;
}

console.log('=== Starting Differential Fuzzing Session (Target: 60s+) ===');
const startTime = Date.now();
const DURATION_MS = 62000; // 62 seconds
let iterations = 0;
let divergences = 0;

const logLines = [];
logLines.push(`[FUZZ] Session started at ${new Date().toISOString()}`);
logLines.push(`[FUZZ] Target duration: 60 seconds`);
logLines.push(`[FUZZ] Engines under test: JS Reference vs Rust N-API (bytes-rs)`);

while (Date.now() - startTime < DURATION_MS) {
  iterations++;
  
  // Test Parse
  const inputStr = randomString();
  const jsParsed = jsParse(inputStr);
  const rustParsed = native.parse(inputStr);
  if (jsParsed !== rustParsed && !(Number.isNaN(jsParsed) && Number.isNaN(rustParsed))) {
    divergences++;
    logLines.push(`[DIVERGENCE] Parse input: "${inputStr}" | JS: ${jsParsed} | Rust: ${rustParsed}`);
  }

  // Test Format
  const numVal = (Math.random() - 0.5) * 1e12;
  const opts = randomOptions();
  const jsFormatted = jsFormat(numVal, opts);
  const rustFormatted = native.format(numVal, sanitizeForNative(opts));
  if (jsFormatted !== rustFormatted) {
    divergences++;
    logLines.push(`[DIVERGENCE] Format input: ${numVal}, opts: ${JSON.stringify(opts)} | JS: "${jsFormatted}" | Rust: "${rustFormatted}"`);
  }

  // Test Main Entrypoint
  const jsMain = jsBytes(inputStr, opts);
  const rustMain = native.bytes(inputStr, sanitizeForNative(opts));
  if (jsMain !== rustMain && !(Number.isNaN(jsMain) && Number.isNaN(rustMain))) {
    divergences++;
    logLines.push(`[DIVERGENCE] Main input: "${inputStr}" | JS: ${jsMain} | Rust: ${rustMain}`);
  }
}

const elapsedSec = ((Date.now() - startTime) / 1000).toFixed(2);
logLines.push(`[FUZZ] Fuzzing complete.`);
logLines.push(`[FUZZ] Total Iterations: ${iterations.toLocaleString()}`);
logLines.push(`[FUZZ] Elapsed Time: ${elapsedSec}s`);
logLines.push(`[FUZZ] Divergences Found: ${divergences}`);
logLines.push(`[FUZZ] Status: ${divergences === 0 ? 'PASSED (ZERO DIVERGENCES)' : 'FAILED'}`);

const logText = logLines.join('\n') + '\n';
fs.writeFileSync(path.join(__dirname, 'log.txt'), logText);

console.log(logText);

if (divergences > 0) {
  process.exit(1);
} else {
  process.exit(0);
}
