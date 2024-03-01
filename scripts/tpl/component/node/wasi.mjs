#!/usr/bin/env node --no-warnings
'use strict'

import { dirname } from 'path'
import { env, argv } from 'node:process'
import { fileURLToPath } from 'url'
import { join } from 'node:path'
import { readFile } from 'node:fs/promises'
import { WASI } from 'wasi'

const __dirname = dirname(fileURLToPath(import.meta.url))
let __wasm_path = join(__dirname, '../#DS#.core.wasm')

// First arg is node executable
argv.shift()

const fname = argv[1] ?? 'render'
let req = argv[2]

if (fname === 'describe') {
  __wasm_path = join(__dirname, '../#DS#_dev.core.wasm')
  req = argv[2] ?? 'components::_list'
} else {
  req = argv[2] ?? '/local/payload/index.json'
}

const wasi = new WASI({
  version: 'preview1',
  args: ['', fname, req],
  env,
  preopens: {
    '/local': join(__dirname, '../'),
  },
})

const wasm = await WebAssembly.compile(await readFile(__wasm_path))
const instance = await WebAssembly.instantiate(wasm, wasi.getImportObject())

wasi.start(instance)
