#!/bin/sh

set -e

FILENAME=$1

if [ -z $FILENAME ] ; then
	echo "missing 1st argument (FILENAME, from the wat/ directory)"
	echo "examples of usage:"
	echo "  ./run.sh add"
	echo "  ./run.sh mul"
	exit 1
fi


mkdir -p wasm

wat2wasm wat/$FILENAME.wat -o wasm/$FILENAME.wasm

wasm-interp wasm/$FILENAME.wasm --run-all-exports --trace
