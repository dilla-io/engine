#!/usr/bin/env node --no-warnings
/**
 * This code snippet demonstrates how to load and run a Dilla design system with a WASM extism using Node.js.
 * https://dilla.io
 */
'use strict'
import createPlugin from '@extism/extism'
import { fileURLToPath } from 'url'
import { dirname } from 'path'
import fs from 'fs'

const __dirname = dirname(fileURLToPath(import.meta.url))
const wasm_url = __dirname + '/../#DS#.wasm'
let payload = __dirname + '/../payload/index.json'

payload = fs.readFileSync(payload, 'utf8')

const result = await callExtismRender(payload)
console.dir(result, { depth: null, colors: true })

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