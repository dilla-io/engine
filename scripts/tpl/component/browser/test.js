// Local JavaScript helper for local testing of Dilla Design System

import { callDillaDescribe, callDillaRender } from './test_call.js'

const _DILLA_HELPER_URL = 'https://data.dilla.io/helpers/index.js'

const _TEST_EDITOR = init_editor()
const _TEST_CLOSE = document.getElementById('test_close')
const _SELECT_EXAMPLES = document.getElementById('test_examples')

init()
await init_select()

/**
 * Initializes the functionality of the test page.
 * Sets up event listeners for various elements, loads saved payload and artefact selection from local storage,
 * and calls other functions to perform necessary actions.
 */
function init() {
  _TEST_CLOSE.addEventListener('click', test_toggle_nav);

  _SELECT_EXAMPLES.addEventListener('change', function (e) {
    test_load_examples(this.value);
  });

  document.getElementById('test_reset').addEventListener('click', () => {
    localStorage.removeItem('artefact_id');
    localStorage.removeItem('payload');
  });

  document.getElementById('test_save').addEventListener('click', () => {
    const payload = _TEST_EDITOR.get();
    localStorage.setItem('payload', JSON.stringify(payload, true, 2));
  });

  // If a saved payload in local storage, apply to editor and load.
  let saved_payload = localStorage.getItem('payload');
  if (saved_payload !== null) {
    saved_payload = JSON.parse(saved_payload);
    _TEST_EDITOR.set(saved_payload);
    test_call_wasm(saved_payload);
  } else {
    // If a saved selection, apply to editor and load.
    const existing_selection = localStorage.getItem('artefact_id');
    if (existing_selection !== null) {
      _SELECT_EXAMPLES.value = existing_selection;
      test_load_examples(existing_selection);
    }
  }
}

/**
 * Initializes the select element with options for components and examples.
 * It retrieves the list of components and examples with the Dilla Descripbe API.
 */
async function init_select() {
  const optComponents = await callDillaDescribe('components::_list');
  const optExamples = await callDillaDescribe('examples::_list');

  const optionList = _SELECT_EXAMPLES.options;

  const optGrpCo = document.createElement('optgroup');
  optGrpCo.label = 'Components';

  optComponents.forEach((option) => {
    const value = `components:${option}`;
    optGrpCo.appendChild(new Option(option, value));
  });

  optionList.add(optGrpCo);

  const optGrpEx = document.createElement('optgroup');
  optGrpEx.label = 'Examples';

  optExamples.forEach((option) => {
    const value = `examples:${option}`;
    optGrpEx.appendChild(new Option(option, value));
  });

  optionList.add(optGrpEx);
}

async function test_load_examples(val) {
  localStorage.setItem('artefact_id', val)

  const values = val.split(':')
  const artefact = values[0]
  const id = values[1]

  const result = await callDillaDescribe(artefact + '::' + id)

  let payload = null
  if (result.renderable !== undefined) {
    payload = result.renderable
  } else if (result.examples !== undefined) {
    // @todo merge multiple renderable
    // const renderables = [];
    // for (const preview in result.examples) {
    //   if (result.examples[preview].renderable !== undefined) {
    //     renderables.push(result.examples[preview].renderable);
    //   }
    // }
    // if (renderables.length > 0) {
    //   payload = renderables;
    // }
    for (const preview in result.examples) {
      if (result.examples[preview].renderable !== undefined) {
        payload = result.examples[preview].renderable
        break
      }
    }
  } else {
    document.getElementById('dilla').innerHTML = `No examples definition found for <b>${id}</b>!`
    return
  }

  if (payload) {
    await test_call_wasm(payload)
    _TEST_EDITOR.set(payload)
  }
}

async function test_call_wasm(payload = null) {
  const styles = document.createElement('head')
  document.querySelectorAll('head style').forEach((style) => {
    styles.appendChild(style)
  })

  document.head.innerHTML = ''
  document.head.appendChild(styles)
  document.getElementById('dilla').innerHTML = ''

  if (payload === null) {
    let payload = document.getElementById('test_payload').value
    if (payload == '') {
      document.getElementById('dilla').innerHTML = `Empty payload`
    }
  }

  let result = await callDillaRender(payload)

  const dilla_helper = await import(_DILLA_HELPER_URL)
  dilla_helper.insertResultFull(result)
}

function init_editor() {
  const json_container = document.getElementById('test_jsoneditor')
  const options = {
    mainMenuBar: false,
    statusBar: false,
    navigationBar: false,
    history: false,
    enableSort: false,
    enableTransform: false,
    search: false,
    modes: ['code'],
    onChangeText: function (payload) {
      test_call_wasm(JSON.parse(payload))
    },
  }
  const editor = new JSONEditor(json_container, options)
  if (editor.aceEditor !== undefined) {
    editor.aceEditor.setOptions({
      showLineNumbers: false,
      showGutter: false,
      maxLines: 1000,
    })
    editor.aceEditor.container.style.background = 'transparent'
  }

  return editor
}

function test_toggle_nav() {
  const element = document.getElementById('test_mySidebar')
  const container = document.getElementById('test_container')
  if (element.classList.contains('test_closed')) {
    element.style.width = '400px'
    container.style.marginLeft = '400px'
    _TEST_CLOSE.innerHTML = '<'
  } else {
    element.style.width = '21px'
    container.style.marginLeft = '21px'
    _TEST_CLOSE.innerHTML = '>'
  }
  element.classList.toggle('test_closed')
}
