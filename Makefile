clean:
	rm -rf apps

build-plugins: 
	cargo build --target wasm32-unknown-unknown --release --manifest-path plugins/invert/Cargo.toml
	cargo build --target wasm32-unknown-unknown --release --manifest-path plugins/md2html/Cargo.toml
	
	mkdir -p apps/inverter
	mkdir -p apps/md2html
	
	cp target/wasm32-unknown-unknown/release/invert.wasm apps/inverter
	cp target/wasm32-unknown-unknown/release/md2html.wasm apps/md2html

debug-build-plugins: 
	cargo build --target wasm32-unknown-unknown --manifest-path plugins/invert/Cargo.toml
	cargo build --target wasm32-unknown-unknown --manifest-path plugins/md2html/Cargo.toml
	
	mkdir -p apps/inverter
	mkdir -p apps/md2html
	
	cp target/wasm32-unknown-unknown/debug/invert.wasm apps/inverter
	cp target/wasm32-unknown-unknown/debug/md2html.wasm apps/md2html
	
run:
	go build -o ext && open apps && ./ext run apps
