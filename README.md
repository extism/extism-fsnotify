# extism-fsnotify

https://user-images.githubusercontent.com/7517515/188239556-029f5142-0cb8-4bfb-a5b2-e4bbe29e25c6.mp4

Use the filesystem as a computing environment. 

Make sure you have Extism installed: https://extism.org/docs/install

```sh
make build-plugins # requires Rust toolchain
make run # requires Go toolchain
```

### Demo: construct programmable directories on your filesystem.

- **`plugins/invert`**: invert the colors of a PNG file 
  - add a new PNG to the directory containing this .wasm plug-in
  - see new PNG replacing the original with inverted colors
  
- **`plugins/md2html`**: convert Markdown in to HTML files 
  - add a Markdown file to the directory containing this .wasm plug-in
  - see a new HTML file generated next to the Markdown file

The host program runs a recursive watcher, to build the initial structure, and all new directories 
with files + .wasm are watched and computed.

For each file create operation, the .wasm is loaded and passed the file that triggered the event. 

This program treats a directory, a written file, and a single .wasm module as the unit of computation. 

### Data types

1. Host -> Plug-in

```json
{
  "event_file_name": "path/to/file.md",
  "event_file_data": "<base64 encoded file>"
}
```

2. Plug-in -> Host

```json
{
  "op": "create",
  "output_file_name": "path/to/file.md",
  "output_file_data": "<base64 encoded file>"
}
```
