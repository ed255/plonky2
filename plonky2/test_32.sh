#!/bin/sh
set -ex
RUSTFLAGS=-Awarnings cargo test --release --target i686-unknown-linux-gnu batch_fri::oracle::test::batch_prove_openings -- --nocapture
