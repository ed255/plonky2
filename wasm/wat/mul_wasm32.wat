(module
  (memory 1)
  (func $main (result i64)
    i64.const 9223372034707292160
    i64.const 42
    call $mul
  )
  (func $mul (param $a i64) (param $b i64) (result i64)
    (local i64 i64 i64)
    local.get 1
    i64.const 4294967295
    i64.and
    local.tee 2
    local.get 0
    i64.const 32
    i64.shr_u
    local.tee 3
    i64.mul
    local.get 1
    i64.const 32
    i64.shr_u
    local.tee 1
    local.get 0
    i64.const 4294967295
    i64.and
    local.tee 0
    i64.mul
    i64.add
    local.get 1
    local.get 3
    i64.mul
    local.tee 1
    i64.add
    local.tee 3
    i64.const 32
    i64.shr_u
    local.tee 4
    local.get 3
    i64.add
    i64.const 32
    i64.shl
    local.get 2
    local.get 0
    i64.mul
    local.get 1
    local.get 4
    i64.add
    i64.sub
    i64.or)
  (export "main" (func $main))
)
