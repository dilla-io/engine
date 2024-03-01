// Specific call WASM code for wasm-bindgen generated WASM and wrapper.

const js_bg_dev = window.location.href.replace('/test.html', '/#DS#_dev.js')

async function callDilla() {
  const dilla = await import(js_bg_dev)
  await dilla.default()
  return dilla
}

async function callDillaDescribe(request) {
  const values = request.split('::')
  const artefact = values[0]
  const id = values[1]

  const dilla_bg = await callDilla()
  let result = dilla_bg.describe(artefact, id)

  if (typeof result === 'string') {
    result = JSON.parse(result)
  }

  return result
}

async function callDillaRender(payload) {
  if (typeof payload === 'string') {
    payload = JSON.parse(payload)
  }

  const time_start = Date.now()

  const dilla_bg = await callDilla()
  let result = dilla_bg.render(payload, false)

  const duration = Date.now() - time_start
  console.debug(`[Dilla #TYPE#] generated in ${duration}ms`)

  if (typeof result === 'string') {
    result = JSON.parse(result)
  }

  return result
}

export { callDillaDescribe, callDillaRender }
