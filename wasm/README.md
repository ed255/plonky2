# plonky2/wasm

## Dev
Requires https://github.com/WebAssembly/wabt installed.

- compile wasm with the methods that appear in `src/lib`: `cargo build --target wasm32-unknown-unknown --release`
- convert the wasm to wat: `wasm2wat ../target/wasm32-unknown-unknown/release/wasm_test.wasm -o wasm_test.wat`
- manually isolate the desired methods in their respective wat files, place them in the `wat/` directory
- execute the isolated wat methods with the `run.sh` script, eg: `./run.sh add`
