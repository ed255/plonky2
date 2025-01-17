// new WebAssembly.Module(os.file.readFile("add.wasm","binary"));
m = new WebAssembly.Module(os.file.readFile("mul.wasm","binary"));
c = wasmExtractCode(m);
os.file.writeTypedArrayToFile('out.bin', c.code);

