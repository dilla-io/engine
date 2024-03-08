#!/usr/bin/env node --no-warnings
/**
 * This code snippet demonstrates how to load and run a Dilla design system with a WASM component using Node.js.
 * https://dilla.io
 */
'use strict'

import { cli } from '@bytecodealliance/preview2-shim'
import { dirname } from 'path'
import { fileURLToPath } from 'url'
import fs from 'fs'
import { render } from '../#DS#.mjs'

const log = cli.stdout.getStdout()

const __dirname = dirname(fileURLToPath(import.meta.url))
let payload = __dirname + '/../payload/index.json'
payload = fs.readFileSync(payload, 'utf8')

const result = render(payload)

log.write(result + '\n')