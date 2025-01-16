(module
  (type (;0;) (func (param i64 i64) (result i64)))
  (type (;1;) (func (param i64 i64) (result i64)))
  (type (;2;) (func (param i32 i64 i64 i64 i64)))
  (func $main (result i64)
    ;; values that will wrap Goldilocks
    i64.const 9223372034707292160
    i64.const 42
    call $test_goldilocks_mul
  )
  (func $test_goldilocks_mul (type 0) (param i64 i64) (result i64)
    local.get 0
    local.get 1
    call $_ZN90_$LT$plonky2_field..goldilocks_field..GoldilocksField$u20$as$u20$core..ops..arith..Mul$GT$3mul17h7e29d6cd27545fc1E
    local.tee 1
    i64.const 4294967295
    i64.add
    local.get 1
    local.get 1
    i64.const -4294967296
    i64.gt_u
    select)
  (func $_ZN90_$LT$plonky2_field..goldilocks_field..GoldilocksField$u20$as$u20$core..ops..arith..Mul$GT$3mul17h7e29d6cd27545fc1E (type 1) (param i64 i64) (result i64)
    (local i32 i64 i64)
    global.get $__stack_pointer
    i32.const 16
    i32.sub
    local.tee 2
    global.set $__stack_pointer
    local.get 2
    local.get 1
    i64.const 0
    local.get 0
    i64.const 0
    call $__multi3
    local.get 2
    i32.const 8
    i32.add
    i64.load
    local.set 0
    local.get 2
    i64.load
    local.set 1
    local.get 2
    i32.const 16
    i32.add
    global.set $__stack_pointer
    i64.const 4294967295
    i64.const 0
    local.get 1
    local.get 0
    i64.const 32
    i64.shr_u
    local.tee 3
    i64.sub
    local.tee 4
    i64.const -4294967295
    i64.add
    local.get 4
    local.get 1
    local.get 3
    i64.lt_u
    select
    local.tee 1
    local.get 0
    i64.const 4294967295
    i64.and
    i64.const 4294967295
    i64.mul
    i64.add
    local.tee 0
    local.get 1
    i64.lt_u
    select
    local.get 0
    i64.add)
  (func $__multi3 (type 2) (param i32 i64 i64 i64 i64)
    (local i64 i64 i64 i64 i64 i64)
    local.get 0
    local.get 3
    i64.const 4294967295
    i64.and
    local.tee 5
    local.get 1
    i64.const 4294967295
    i64.and
    local.tee 6
    i64.mul
    local.tee 7
    local.get 3
    i64.const 32
    i64.shr_u
    local.tee 8
    local.get 6
    i64.mul
    local.tee 6
    local.get 5
    local.get 1
    i64.const 32
    i64.shr_u
    local.tee 9
    i64.mul
    i64.add
    local.tee 5
    i64.const 32
    i64.shl
    i64.add
    local.tee 10
    i64.store
    local.get 0
    local.get 8
    local.get 9
    i64.mul
    local.get 5
    local.get 6
    i64.lt_u
    i64.extend_i32_u
    i64.const 32
    i64.shl
    local.get 5
    i64.const 32
    i64.shr_u
    i64.or
    i64.add
    local.get 10
    local.get 7
    i64.lt_u
    i64.extend_i32_u
    i64.add
    local.get 4
    local.get 1
    i64.mul
    local.get 3
    local.get 2
    i64.mul
    i64.add
    i64.add
    i64.store offset=8)
  (global $__stack_pointer (mut i32) (i32.const 1048576))
  (memory (;0;) 17)
  (export "main" (func $main))
)
