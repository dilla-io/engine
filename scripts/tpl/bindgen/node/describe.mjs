#!/usr/bin/env node --no-warnings
/**
 * This code snippet demonstrates how to load and run a Dilla design system describe API with a WASM #TYPE# using Node.js.
 * https://dilla.io
 */
'use strict'
import { describe } from './#DS#_dev.js'

const result = describe('component', '_list')
console.log(result)
