clean:
	rm -f inverter/*

plugin: 
	cargo build --target wasm32-unknown-unknown --release --manifest-path plugins/invert/Cargo.toml
	mkdir -p inverter
	cp target/wasm32-unknown-unknown/release/invert.wasm inverter

run:
	go build -o ext && open inverter && ./ext run inverter