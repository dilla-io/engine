#!/usr/bin/env node --no-warnings
/**
 * This code snippet demonstrates how to load and run a Dilla design system describe API with a WASM extism using Node.js.
 * https://dilla.io
 */
'use strict'
import createPlugin from '@extism/extism'
import { fileURLToPath } from 'url'
import { dirname } from 'path'

const __dirname = dirname(fileURLToPath(import.meta.url))
const wasm_url = __dirname + '/../#DS#_dev.wasm'

const plugin = await createPlugin(wasm_url, {
  useWasi: true,
})

const describe = await plugin.call('describe', new TextEncoder().encode('components::_list'))
const result = new TextDecoder().decode(describe.buffer)
console.log(result)

await plugin.close()
