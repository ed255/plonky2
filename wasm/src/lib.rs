use plonky2::field::goldilocks_field::GoldilocksField as F;
use plonky2::field::types::{Field, PrimeField64};
// use plonky2::hash::poseidon::{Poseidon, SPONGE_WIDTH};
// extern crate wasm_bindgen_test;
// use wasm_bindgen_test::*;
// use web_sys::{console, window, Performance};

extern "C" {
    // We use these functions to make sure the input is not a constant and that the output is used
    // so that the compiler doesn't optimize the function away.  It also makes it easier to follow
    // the wat
    fn get_input() -> u64;
    fn set_output(v: u64);
}

// wasm_bindgen_test_configure!(run_in_browser);

// fn get_wasm_time() -> u64 {
//     let window = window().expect("should have a window in this context");
//     let performance = window
//         .performance()
//         .expect("performance should be available");
//     performance.now() as u64
// }
//
// #[wasm_bindgen_test]
// fn test_poseidon_n() {
//     const N: usize = 512 * 1024;
//     let mut input = [F::ZERO; SPONGE_WIDTH];
//     let mut output = input;
//     let start = get_wasm_time() as u64;
//     for _ in 0..N {
//         output = F::poseidon(input);
//         input = output;
//     }
//     let end = get_wasm_time() as u64;
//     console::log_1(&format!("{} ms", end - start).into());
// }
//
// #[wasm_bindgen_test]
// fn test_poseidon_1() {
//     let input = [F::ZERO; SPONGE_WIDTH];
//     let start = get_wasm_time() as u64;
//     let _output = F::poseidon(input);
//     let end = get_wasm_time() as u64;
//     console::log_1(&format!("{} ms", end - start).into());
// }

#[no_mangle]
pub extern "C" fn test_goldilocks_add_external_inputs() {
    let a = F::from_canonical_u64(unsafe { get_input() });
    let b = F::from_canonical_u64(unsafe { get_input() });
    let c = a + b;
    unsafe {
        set_output(c.0);
    }
}

#[no_mangle]
pub extern "C" fn test_goldilocks_mul_external_inputs() {
    let a = F::from_canonical_u64(unsafe { get_input() });
    let b = F::from_canonical_u64(unsafe { get_input() });
    let c = a * b;
    unsafe {
        set_output(c.0);
    }
}

#[no_mangle]
pub extern "C" fn test_goldilocks_add(a: u64, b: u64) -> u64 {
    let a = F::from_canonical_u64(a);
    let b = F::from_canonical_u64(b);
    let c = a + b;
    c.0
}
#[no_mangle]
pub extern "C" fn test_goldilocks_mul(a: u64, b: u64) -> u64 {
    let a = F::from_canonical_u64(a);
    let b = F::from_canonical_u64(b);
    let c = a * b;
    c.0
}

#[no_mangle]
pub extern "C" fn test_goldilocks_many_add(a: u64, b: u64) -> u64 {
    let mut a = F::from_canonical_u64(a);
    let b = F::from_canonical_u64(b);
    const N: usize = 0x8000000;
    for _ in 0..N {
        a = a + b;
    }
    a.0
}
