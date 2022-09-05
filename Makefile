clean:
	rm -rf apps

plugins: 
	cargo build --target wasm32-unknown-unknown --release --manifest-path plugins/invert/Cargo.toml
	cargo build --target wasm32-unknown-unknown --release --manifest-path plugins/md2html/Cargo.toml
	
	mkdir -p apps/{inverter,md2html}
	
	cp target/wasm32-unknown-unknown/release/invert.wasm apps/inverter
	cp target/wasm32-unknown-unknown/release/md2html.wasm apps/md2html

run:
	go build -o ext && open apps && ./ext run apps