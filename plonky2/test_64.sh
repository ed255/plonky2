#!/bin/sh
set -ex
RUSTFLAGS=-Awarnings cargo test --release batch_fri::oracle::test::batch_prove_openings -- --nocapture
