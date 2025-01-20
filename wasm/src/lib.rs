use plonky2::field::goldilocks_field::{mul_wasm32, GoldilocksField as F};
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

#[no_mangle]
pub extern "C" fn test_goldilocks_many_mul(a: u64, b: u64) -> u64 {
    let mut a = F::from_canonical_u64(a);
    let b = F::from_canonical_u64(b);
    const N: usize = 0x8000000;
    for _ in 0..N {
        a = a * b;
    }
    a.0
}

const EPSILON: u64 = (1 << 32) - 1;

/// mul_wasm32 implements the trick explained by Jordi Baylina
pub fn mul_wasm32_dbg(a: F, b: F) -> F {
    fn split_u64(v: u64) -> (u64, u64) {
        (v & 0xffffffff, v >> 32)
    }

    let (a0, a1) = split_u64(a.0);
    let (b0, b1) = split_u64(b.0);

    let a0_b0 = a0 * b0;
    let a0_b1 = a0 * b1;
    let a1_b0 = a1 * b0;
    let a1_b1 = a1 * b1;

    // let m = a0*b1 + a1*b0 + a1*b1;
    let (m, over) = a0_b1.overflowing_add(a1_b0);
    dbg!(over);
    let (mut m, over) = m.overflowing_add((over as u64) * EPSILON);
    dbg!(over);
    if over {
        m += EPSILON;
    }
    let (m, over) = m.overflowing_add(a1_b1);
    dbg!(over);
    let (mut m, over) = m.overflowing_add((over as u64) * EPSILON);
    dbg!(over);
    if over {
        m += EPSILON;
    }
    let (m0, m1) = split_u64(m);

    // let c0 = a0*b0 - a1*b1 - m1;
    let (c0, borrow) = a0_b0.overflowing_sub(a1_b1);
    dbg!(borrow);
    let (mut c0, borrow) = c0.overflowing_sub((borrow as u64) * EPSILON);
    dbg!(borrow);
    if borrow {
        c0 -= EPSILON;
    }
    let (c0, borrow) = c0.overflowing_sub(m1);
    dbg!(borrow);
    let (mut c0, borrow) = c0.overflowing_sub((borrow as u64) * EPSILON);
    dbg!(borrow);
    if borrow {
        c0 -= EPSILON;
    }

    let c1 = m0 + m1;

    // let c: u64 = (c1 << 32) | c0;
    // let (c, over) = c1.overflowing_shl(32);
    let c = c1 << 32;
    let over = c1 > EPSILON;
    dbg!(over);
    let (mut c, over) = c.overflowing_add((over as u64) * EPSILON);
    dbg!(over);
    if over {
        c += EPSILON;
    }
    let (c, over) = c.overflowing_add(c0);
    dbg!(over);
    let (mut c, over) = c.overflowing_add((over as u64) * EPSILON);
    dbg!(over);
    if over {
        c += EPSILON;
    }
    F(c)
}

// to run these tests:
// cargo test --release -- --nocapture
#[cfg(test)]
mod tests {
    use plonky2::field::types::Sample;
    use rand::rngs::OsRng;
    use rand::{Rng, RngCore};

    use super::*;

    fn print_u64(tag: &str, v: u64) {
        println!("{}={:08x}_{:08x}", tag, v >> 32, v & EPSILON);
    }

    #[test]
    fn test_mul_wasm32_0() {
        let a: u64 = 9223372034707292160;
        let b: u64 = 42;
        let a = F::from_canonical_u64(a);
        let b = F::from_canonical_u64(b);
        let c = mul_wasm32(a, b);
        assert_eq!(c.to_canonical_u64(), (a * b).to_canonical_u64()); // compare to the non-wasm32 mult
        assert_eq!(c.to_canonical_u64(), 18446744069414584300 as u64);

        println!("");

        let a: u64 = 9223372034707292160;
        let b: u64 = 9223372034707292161;
        let a = F::from_canonical_u64(a);
        let b = F::from_canonical_u64(b);
        let c = mul_wasm32(a, b);
        assert_eq!(c.to_canonical_u64(), (a * b).to_canonical_u64()); // compare to the non-wasm32 mult
        assert_eq!(c.to_canonical_u64(), 4611686017353646080 as u64);

        let a: u64 = 10455546295413958833;
        let b: u64 = 4511168707820812235;
        let a = F::from_canonical_u64(a);
        let b = F::from_canonical_u64(b);
        let c = mul_wasm32(a, b);
        print_u64("l", c.to_canonical_u64());
        print_u64("r", (a * b).to_canonical_u64());
        assert_eq!(c.to_canonical_u64(), (a * b).to_canonical_u64()); // compare to the non-wasm32 mult
        assert_eq!(c.to_canonical_u64(), 8353554915440457809 as u64);
    }

    // uncomment once the previous test passes:
    #[test]
    fn test_mul_wasm32_loop() {
        for _ in 0..10_000 {
            let a = F::rand();
            let b = F::rand();
            let c = mul_wasm32(a, b);
            assert_eq!(
                c.to_canonical_u64(),
                (a * b).to_canonical_u64(),
                "a={}, b={}",
                a,
                b
            ); // compare to the non-wasm32 mult
        }
    }
}
