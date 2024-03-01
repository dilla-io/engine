// Specific call WASM code for wasm-bindgen generated WASM and wrapper.

import createPlugin from './node_modules/@extism/extism/dist/browser/mod.js'

const wasm_url = window.location.href.replace('/browser/test.html', '/#DS#_dev.wasm')

async function callExtismPlugin(fname, input) {
  const time_start = Date.now()

  const plugin = await createPlugin(wasm_url, {
    useWasi: true,
  })

  const req = await plugin.call(fname, new TextEncoder().encode(input))
  let result = new TextDecoder().decode(req.buffer)

  const duration = Date.now() - time_start
  if (fname == "render") {
    console.debug(`[Dilla #TYPE#] generated in ${duration}ms`)
  }

  await plugin.close()

  return JSON.parse(result)
}

async function callDillaDescribe(request) {
  let result = await callExtismPlugin('describe', request)

  if (typeof result === 'string') {
    result = JSON.parse(result)
  }

  return result
}

async function callDillaRender(payload) {
  if (typeof payload === 'object') {
    payload = JSON.stringify(payload)
  }

  let result = await callExtismPlugin('render', payload)

  if (typeof result === 'string') {
    result = JSON.parse(result)
  }

  return result
}

export { callDillaDescribe, callDillaRender }
