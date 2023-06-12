APP_NAME := WgpuStart
WASM_BINDGEN = wasm-bindgen --target web --weak-refs --reference-types
WASM_TARGET_FEATURES := "+bulk-memory,+mutable-globals,+nontrapping-fptoint,+sign-ext,+simd128,+reference-types"
WASM_DIR = debug
WASM_OPT = wasm-opt --all-features --disable-gc
ifeq ($(WASM_RELEASE),1)
  WASM_BUILD_PROFILE := --release
  WASM_DIR=release
  WASM_OPT += -O3
else
  WASM_OPT += -O0
endif

macos-app:
	cargo install cargo-bundle
	cargo bundle --release
	cd target/release/bundle/osx && zip -r $(APP_NAME).zip $(APP_NAME).app &> /dev/null
	echo target/release/bundle/osx/$(APP_NAME).zip

run-app:
	@cargo bundle --release &> /dev/null
	@open target/release/bundle/osx/$(APP_NAME).app

wasm:
	# --cfg=web_sys_unstable_apis is necessary for webgpu:
	# https://rustwasm.github.io/wasm-bindgen/api/web_sys/enum.GpuTextureFormat.html
	RUSTFLAGS="--cfg=web_sys_unstable_apis -C target-feature=$(WASM_TARGET_FEATURES)" \
		cargo build $(WASM_BUILD_PROFILE) --target wasm32-unknown-unknown
	rm -Rf site/generated
	$(WASM_BINDGEN) --out-dir site/generated target/wasm32-unknown-unknown/$(WASM_DIR)/wgpu_game_of_life.wasm
	$(WASM_OPT) -o site/generated/wgpu_game_of_life_bg.wasm site/generated/wgpu_game_of_life_bg.wasm
	(sleep 1 && open http://localhost:8888) & cd site && python3 -m http.server 8888
