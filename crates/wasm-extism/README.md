
// DS=swing_1 cargo build --target wasm32-wasi && extism call target/wasm32-wasi/debug/wasm_extism.wasm render --input "{\"@element\":\"p\",\"@content\":\"yo\"}" --wasi
// DS=swing_1 cargo build --target wasm32-wasi --release && extism call target/wasm32-wasi/release/wasm_extism.wasm render --input "{\"@element\":\"p\",\"@content\":\"yo\"}" --wasi
// DS=swing_1 cargo build --target wasm32-wasi
// extism call target/wasm32-wasi/release/wasm_extism.wasm render --input "{\"@element\":\"p\",\"@content\":\"yo\"}" --wasi
// extism call target/wasm32-wasi/release/wasm_extism.wasm describe --input "component::alert" --wasi
