# extism-fsnotify

Use the filesystem as a computing environment. 

```sh
go build -o ext main.go

ls inverter | grep invert.wasm # ensure `invert.wasm` plugin is present
cp some.png invert

./ext run inverter
```

- open up a Finder.app or equivalent on your system 
- navigate to the `./inverter` directory 
- add a new PNG to the directory
- see new PNG replaced with an inverted on

Currently not run recursively, but would be cool to assume all new directories with files + .wasm are watched and computed.

This program treats a directory, it's root level files, and a single .wasm module as the unit of computation. 

For each WRITE operation, the .wasm is loaded and passed the directory's files. It returns a single value, which is then
used to overwrite the file which caused the WRITE event to trigger.