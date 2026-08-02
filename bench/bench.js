'use strict';

const fs = require('fs');
const path = require('path');
const { performance } = require('perf_hooks');

// Benchmark JS Reference vs Rust N-API (bytes-rs)

// JS Pure implementation
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
  if (!fixedDecimals) str = str.replace(/(?:\.0*|(\.[^0]+)0+)$/, '$1');
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
  var floatValue, unit = 'b';
  if (!results) {
    floatValue = parseInt(val, 10);
  } else {
    floatValue = parseFloat(results[1]);
    unit = results[4].toLowerCase();
  }
  if (isNaN(floatValue)) return null;
  return Math.floor(map[unit] * floatValue);
}

// Measure module load time
const t0 = performance.now();
const native = require('../index.node');
const t1 = performance.now();
const startupTimeMs = parseFloat((t1 - t0).toFixed(3));

function measureLatencyAndThroughput(fn, testData, iterations = 100000) {
  // Warmup
  for (let i = 0; i < 5000; i++) {
    fn(testData[i % testData.length]);
  }

  const latenciesNs = new BigInt64Array(iterations);
  const startTotal = performance.now();

  for (let i = 0; i < iterations; i++) {
    const item = testData[i % testData.length];
    const s = process.hrtime.bigint();
    fn(item);
    const e = process.hrtime.bigint();
    latenciesNs[i] = e - s;
  }

  const totalTimeMs = performance.now() - startTotal;
  const opsPerSec = Math.round((iterations / totalTimeMs) * 1000);

  // Sort for percentiles
  const sorted = Array.from(latenciesNs).map(n => Number(n) / 1000).sort((a, b) => a - b);
  const p50 = parseFloat(sorted[Math.floor(iterations * 0.50)].toFixed(3));
  const p95 = parseFloat(sorted[Math.floor(iterations * 0.95)].toFixed(3));
  const p99 = parseFloat(sorted[Math.floor(iterations * 0.99)].toFixed(3));

  return { opsPerSec, p50_us: p50, p95_us: p95, p99_us: p99 };
}

// Generate benchmark dataset
const parseInputs = ['1024', '1.5MB', '2.5GB', '100TB', '500B', '0.5KB', '-1024MB', '10PB'];
const parseDataset = [];
for (let i = 0; i < 10000; i++) {
  parseDataset.push(parseInputs[i % parseInputs.length]);
}

const formatInputs = [1024, 1572864, 2684354560, 1099511627776, 500, 512, -1073741824];
const formatDataset = [];
for (let i = 0; i < 10000; i++) {
  formatDataset.push(formatInputs[i % formatInputs.length]);
}

// Clean options for Rust
function sanitizeOpts(opts) {
  if (!opts) return opts;
  return {
    decimalPlaces: opts.decimalPlaces,
    fixedDecimals: opts.fixedDecimals,
    thousandsSeparator: opts.thousandsSeparator,
    unit: opts.unit,
    unitSeparator: opts.unitSeparator
  };
}

console.log('Running Benchmarks...');

// JS Benchmarks
const jsParseBench = measureLatencyAndThroughput(jsParse, parseDataset);
const jsFormatBench = measureLatencyAndThroughput(val => jsFormat(val, null), formatDataset);

// Rust N-API Benchmarks
const rustParseBench = measureLatencyAndThroughput(val => native.parse(val), parseDataset);
const rustFormatBench = measureLatencyAndThroughput(val => native.format(val, null), formatDataset);

const memUsage = process.memoryUsage();
const rssMb = parseFloat((memUsage.rss / 1024 / 1024).toFixed(2));
const heapUsedMb = parseFloat((memUsage.heapUsed / 1024 / 1024).toFixed(2));

const results = {
  timestamp: new Date().toISOString(),
  environment: {
    nodeVersion: process.version,
    platform: process.platform,
    arch: process.arch
  },
  startupTimeMs: startupTimeMs,
  memory: {
    rssMb: rssMb,
    heapUsedMb: heapUsedMb
  },
  benchmarks: {
    parse: {
      javascript: jsParseBench,
      rust_napi: rustParseBench,
      speedup_factor: parseFloat((rustParseBench.opsPerSec / jsParseBench.opsPerSec).toFixed(2))
    },
    format: {
      javascript: jsFormatBench,
      rust_napi: rustFormatBench,
      speedup_factor: parseFloat((rustFormatBench.opsPerSec / jsFormatBench.opsPerSec).toFixed(2))
    }
  }
};

fs.writeFileSync(path.join(__dirname, 'results.json'), JSON.stringify(results, null, 2));

console.log('Benchmark Results Written to bench/results.json:');
console.log(JSON.stringify(results, null, 2));
