#!/usr/bin/env node --no-warnings
/**
 * This code snippet demonstrates how to load and run a Dilla design system with a WASM #TYPE# using Node.js.
 * https://dilla.io
 */
'use strict'
import { dirname } from 'path'
import { fileURLToPath } from 'url'
import { render } from './#DS#.js'
import fs from 'fs'

const __dirname = dirname(fileURLToPath(import.meta.url))
let payload = __dirname + '/../payload/index.json'
payload = fs.readFileSync(payload, 'utf8')

const result = render(JSON.parse(payload), false)
console.dir(result, { depth: null, colors: true })
