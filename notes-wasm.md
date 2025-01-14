# Test native

```
cargo test --release --test poseidon test_poseidon_n -- --nocapture
```

# Test wasm

```
wasm-pack test --release --firefox --headless --test poseidon_wasm -- test_poseidon_n --nocapture
```

# Build test wasm

```
cargo test --release --test poseidon_wasm --no-run --target wasm32-unknown-unknown
```

# Build lib wasm

```
cargo build --release --target wasm32-unknown-unknown
```

# Build and review the wat

From the folder `wasm`:
```
cargo build --release --target wasm32-unknown-unknown
wasm2wat ../target/wasm32-unknown-unknown/release/wasm_test.wasm -o wasm_test.wat
``
