#!/usr/bin/env node --no-warnings
/**
 * This code snippet demonstrates how to load and run a Dilla design system with a WASM component using Node.js.
 * https://dilla.io
 */
'use strict'

import { argv } from 'node:process'
import { cli } from '@bytecodealliance/preview2-shim'
import { dirname } from 'path'
import { fileURLToPath } from 'url'
import { join } from 'node:path'
import { render } from '../#DS#.mjs'

const log = cli.stdout.getStdout()

const __dirname = dirname(fileURLToPath(import.meta.url))
const payload = join(__dirname, argv[2] ?? '/../payload/index.json')

const result = render(payload)

log.write(result + '\n')
