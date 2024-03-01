#!/usr/bin/env node --no-warnings
/**
 * This code snippet demonstrates how to load and run a Dilla design system with a WASM component using Node.js.
 * https://dilla.io
 */
'use strict'
import { dirname } from 'path'
import { env, argv } from 'node:process'
import { fileURLToPath } from 'url'
import { join } from 'node:path'
import { performance } from 'node:perf_hooks'
import { readFile } from 'node:fs/promises'
import { WASI } from 'wasi'

const __dirname = dirname(fileURLToPath(import.meta.url))
let __wasm_path = join(__dirname, '../#DS#.core.wasm')

const fname = 'render'
const req = '/local/payload/index.json'

const benchmarkRuns = argv[2] ?? 20
const warmupRuns = argv[3] ?? 2

// Warm-up phase
for (let i = 0; i < warmupRuns; i++) {
  await render()
}

// Reset performance data
performance.clearMarks()
performance.clearMeasures()

// Benchmark phase
let minDuration = Infinity
let maxDuration = -Infinity

for (let i = 0; i < benchmarkRuns; i++) {
  performance.mark('start')

  await render()

  performance.mark('end')
  performance.measure(`Run ${i + 1}`, 'start', 'end')

  // Update min and max durations
  const currentDuration = performance.getEntriesByType('measure')[i].duration
  minDuration = Math.min(minDuration, currentDuration)
  maxDuration = Math.max(maxDuration, currentDuration)
}

const measures = performance.getEntriesByType('measure')
const totalDuration = measures.reduce((sum, measure) => sum + measure.duration, 0)
const averageDuration = totalDuration / benchmarkRuns

console.log(`${averageDuration.toFixed(1)} ms ± ${(maxDuration - minDuration).toFixed(1)} ms (min: ${minDuration.toFixed(1)}, max: ${maxDuration.toFixed(1)})`)

async function render() {
  const wasi = new WASI({
    version: 'preview1',
    args: ['', fname, req, true],
    env,
    preopens: {
      '/local': join(__dirname, '../'),
    },
  })

  const wasm = await WebAssembly.compile(await readFile(__wasm_path))
  const instance = await WebAssembly.instantiate(wasm, wasi.getImportObject())

  const result = wasi.start(instance)
}