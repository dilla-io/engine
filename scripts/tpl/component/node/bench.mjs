#!/usr/bin/env node --no-warnings
/**
 * This code snippet demonstrates how to load and run a Dilla design system with a WASM component using Node.js.
 * https://dilla.io
 */
'use strict'
import { argv } from 'node:process'
import { dirname } from 'path'
import { fileURLToPath } from 'url'
import { join } from 'node:path'
import { render } from '../#DS#.mjs'
import { performance } from 'node:perf_hooks'

const __dirname = dirname(fileURLToPath(import.meta.url))
const payload = join(__dirname, '/../payload/index.json')

const benchmarkRuns = argv[2] ?? 20
const warmupRuns = argv[3] ?? 2

// Warm-up phase
for (let i = 0; i < warmupRuns; i++) {
  render(payload)
}

// Reset performance data
performance.clearMarks()
performance.clearMeasures()

// Benchmark phase
let minDuration = Infinity
let maxDuration = -Infinity

for (let i = 0; i < benchmarkRuns; i++) {
  performance.mark('start')

  render(payload)

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
