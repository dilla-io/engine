# Dilla WASM #TYPE# (WASI) with #DS# - #VERSION#

[Dilla](https://dilla.io) WASM #TYPE# for __#DS#__, version __#VERSION#__.

## Test

### Wasmtime CLI

```bash
wasmtime_render.sh
wasmtime_describe.sh
```

### NodeJS

```bash
node --no-warnings node/describe.mjs components::_list
# Path is relative to the script folder.
node --no-warnings node/render.mjs ../payload/index.json
```

```bash
node --no-warnings node/wasi.mjs describe components::_list
# `/local` is a path shared in WASM relative to the script folder.
node --no-warnings node/wasi.mjs render /local/payload/index.json
```

### Browser

Install jco preview_shim:

```shell
cd browser && npm install
```

Run a local webserver from the root dir, ie: this file folder!

```shell
npm install http-server -g
http-server
```

Visit:

* [http://127.0.0.1:8080/browser](http://127.0.0.1:8080/browser)
* [http://127.0.0.1:8080/browser/describe.html](http://127.0.0.1:8080/browser/describe.html)
* [http://127.0.0.1:8080/browser/test.html](http://127.0.0.1:8080/browser/test.html)
