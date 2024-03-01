#!/usr/bin/env node --no-warnings
/**
 * This code snippet demonstrates how to load and run a Dilla design system describe API with a WASM component using Node.js.
 * https://dilla.io
 */
'use strict'

import { cli } from '@bytecodealliance/preview2-shim'
import { describe } from '../#DS#_dev.mjs'
import { argv } from 'node:process'

const _return = cli.stdout.getStdout()

const result = describe(argv[2] ?? 'component::_list')

_return.write(result + '\n')
