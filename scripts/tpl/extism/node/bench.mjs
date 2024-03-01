#!/usr/bin/env node --no-warnings
/**
 * This code snippet demonstrates how to load and run a Dilla design system with a WASM extism using Node.js.
 * https://dilla.io
 */
'use strict'
import { argv } from 'node:process'
import { dirname } from 'path'
import { fileURLToPath } from 'url'
import { performance } from 'node:perf_hooks'
import createPlugin from '@extism/extism'
import fs from 'fs'

const __dirname = dirname(fileURLToPath(import.meta.url))
const wasm_url = __dirname + '/../#DS#_dev.wasm'
let payload = __dirname + '/../payload/index.json'
payload = fs.readFileSync(payload, 'utf8')

const benchmarkRuns = argv[2] ?? 20
const warmupRuns = argv[3] ?? 2

// Warm-up phase
for (let i = 0; i < warmupRuns; i++) {
  const result = await callExtismRender(payload)
}

// Reset performance data
performance.clearMarks()
performance.clearMeasures()

// Benchmark phase
let minDuration = Infinity
let maxDuration = -Infinity

for (let i = 0; i < benchmarkRuns; i++) {
  performance.mark('start')

  const result = await callExtismRender(payload)

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

async function callExtismRender(payload) {
  const plugin = await createPlugin(wasm_url, {
    useWasi: true,
  })

  const req = await plugin.call('render', new TextEncoder().encode(payload))
  let result = new TextDecoder().decode(req.buffer)

  await plugin.close()

  if (typeof result === 'string') {
    result = JSON.parse(result)
  }

  return result
}
