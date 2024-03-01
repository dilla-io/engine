// Specific call WASM code for wasm-bindgen generated WASM and wrapper.

const js_co_dev = window.location.href.replace('/browser/test.html', '/#DS#_dev.mjs')

async function callDilla() {
  const dilla = await import(js_co_dev)
  return dilla
}

async function callDillaDescribe(request) {
  const dilla_co = await callDilla()
  let result = dilla_co.describe(request)

  if (typeof result === 'string') {
    result = JSON.parse(result)
  }

  return result
}

async function callDillaRender(payload) {
  if (typeof payload === 'object') {
    payload = JSON.stringify(payload)
  }

  const time_start = Date.now()

  const dilla_co = await callDilla()
  let result = dilla_co.render(payload)

  const duration = Date.now() - time_start
  console.debug(`[Dilla #TYPE#] generated in ${duration}ms`)

  if (typeof result === 'string') {
    result = JSON.parse(result)
  }

  return result
}

export { callDillaDescribe, callDillaRender }
