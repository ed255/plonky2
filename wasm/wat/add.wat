(module
  (type (;0;) (func (param i64 i64) (result i64)))
  (func $main (result i64)
    ;; values that will wrap Goldilocks
    i64.const 18446744069414584312
    i64.const 51
    call $test_goldilocks_add
  )
  (func $test_goldilocks_add (type 0) (param i64 i64) (result i64)
    local.get 0
    local.get 1
    i64.add
    local.tee 1
    i64.const 4294967295
    i64.const 0
    local.get 1
    local.get 0
    i64.lt_u
    select
    i64.add
    local.tee 0
    i64.const 4294967295
    i64.add
    local.get 0
    local.get 0
    local.get 1
    i64.lt_u
    select
    local.tee 0
    i64.const 4294967295
    i64.add
    local.get 0
    local.get 0
    i64.const -4294967296
    i64.gt_u
    select)
  (export "main" (func $main))
)
