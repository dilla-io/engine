# Dilla WASM #TYPE# (WASI) with #DS# - #VERSION#

[Dilla](https://dilla.io) WASM #TYPE# for __#DS#__, version __#VERSION#__.

## Test

### #TYPE# CLI

```bash
extism_cli.sh
```

### NodeJS

```bash
node node/describe.mjs
node node/render.mjs
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
