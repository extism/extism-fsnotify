# extism-fsnotify

Use the filesystem as a computing environment. 

```sh
make plugin # requires Rust toolchain
make run # requires Go toolchain
```

- add a new PNG to the opened directory (or navigate to `./inverter` at repo root)
- see new PNG replacing the original with inverted colors

Currently not run recursively, but would be cool to assume all new directories with files + .wasm are watched and computed.

This program treats a directory, a written file, and a single .wasm module as the unit of computation. 

For each WRITE operation, the .wasm is loaded and passed the file that triggered the WRITE event. It returns a single value, which is then used to overwrite the file which caused the WRITE event to trigger.