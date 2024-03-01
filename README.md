# Dilla Engine

<div align="center">
  <img src="https://data.dilla.io/dilla.png" alt="" width=320>
  <p>Share your design systems in a tiny universal package. [https://dilla.io](https://dilla.io)</p>

[![License](https://img.shields.io/badge/License%20-%20GPLv3%20-%20orange
)](https://gitlab.com/dilla-io/engine/-/tree/master/LICENSE.md)
![rustc 1.76.0](https://img.shields.io/badge/rust-1.76%2B-orange.svg)
[![Documentation](https://img.shields.io/badge/Documentation%20-%20%233fb5e0
)](https://doc.dilla.io/renderer/index.html)

Dilla is a fast but minimal WASM builder based on the syntax and behavior
of the [Jinja2](https://jinja.palletsprojects.com/) implemented on top of
[Minijinja](https://docs.rs/minijinja/latest/minijinja). The goal is to
be able to pack your design system into a <strong>universal</strong>
package, executable through a simple <strong>declarative API</strong>, for
both server side and headless rendering.
</div>

To know more about Dilla visit our website [dilla.io](https://dilla.io).

---

- [Local WASM build](#local-wasm-build)
  - [Requirements](#requirements)
  - [Build WASM from a repository](#build-wasm-from-a-repository)
  - [Build WASM from local](#build-wasm-from-local)
    - [Validate the templates](#validate-the-templates)
  - [View and test created WASM](#view-and-test-created-wasm)
- [Rust build](#rust-build)
  - [Requirements for dev](#requirements-for-dev)
  - [Installation](#installation)
  - [Build and run locally](#build-and-run-locally)

## Local WASM build

No need to install Rust or any toolchain!

### Requirements

- Bash, Git, Docker

Clone this project.

Init the `.env` and set a `DS` if working with only one Design System.

```bash
cp .env.dist .env
```

### Build WASM from a repository

Next command will populate `./var/run_ds_src/_DS_NAME_` content with repository cloned, and run prebuild in `./var/run/_DS_NAME_`, then create WASM files in `./dist/bindgen/_DS_NAME_`:

```bash
make init DS=_DS_NAME_ REPO=_REPO_GIT_URL_
# Example:
make init DS=bootstrap_5 REPO=git@gitlab.com:dilla-io/ds/bootstrap_5.git
```

### Build WASM from local

Assuming there is a Design System code available in `./var/run_ds_src/_DS_NAME_`.

To Build the wasm from a modified source in `./var/run_ds_src/_DS_NAME_`:

```bash
make build DS=_DS_NAME_
# Example:
make build DS=bootstrap_5
# Build component and extism versions:
make build-component
make build-extism
```

#### Validate the templates

```bash
make check DS=_DS_NAME_
```

### View and test created WASM

You can run a local server from the `./dist/_DS_NAME_` folder created, then visit `bindgen/browser` or `component/browser`.

Example of local server:

```bash
# Node
npm install -g http-server
# Cargo
cargo install http-server
```

```bash
# from dist/_DS_NAME_/
http-server --help
```

Simply visit the test page:

- http://127.0.0.1:8080/bindgen/browser/test.html

## Rust build

### Requirements for dev

- [Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html)
- [Just](https://github.com/casey/just?tab=readme-ov-file#installation)

See all commands:

```bash
just
```

### Installation

```bash
just install
```

### Build and run locally

Payload is loaded from a file `./payload.json` by default for Just commands.

Design system used is by default _test_, as it's the only internal design system,
other design systems must be set in `./var/run` folder and can be used setting a variable in the shell, ie:

```bash
just build bootstrap_5
just run bootstrap_5
```
