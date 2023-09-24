APP_NAME := Game of Life
WASM_BINDGEN = wasm-bindgen --target web --weak-refs --reference-types
WASM_TARGET_FEATURES := "+bulk-memory,+mutable-globals,+nontrapping-fptoint,+sign-ext,+simd128,+reference-types"
WASM_DIR = debug
WASM_OPT = wasm-opt --all-features --disable-gc
ifeq ($(RELEASE),1)
  CARGO_BUILD_PROFILE := --release
  GRADLE_BUILD_TASK := assembleRelease
  GRADLE_RUN_TASK := installRelease
  WASM_DIR=release
  WASM_OPT += -O3
else
  GRADLE_BUILD_TASK := assembleDebug
  GRADLE_RUN_TASK := installDebug
  WASM_OPT += -O0
endif

CLIPPY_TARGETS = --all-targets \
	--target wasm32-unknown-unknown \
	--target x86_64-linux-android \
	--target x86_64-pc-windows-msvc \
	--target x86_64-unknown-linux-gnu
CLIPPY_PARAMS = -- \
	-W clippy::cargo \
	-W clippy::cast_lossless \
	-W clippy::dbg_macro \
	-W clippy::expect_used \
	-W clippy::manual_filter_map \
	-W clippy::if_not_else \
	-W clippy::items_after_statements \
	-W clippy::large_stack_arrays \
	-W clippy::linkedlist \
	-W clippy::match_same_arms \
	-W clippy::option_if_let_else \
	-W clippy::redundant_closure_for_method_calls \
	-W clippy::needless_continue \
	-W clippy::needless_pass_by_value \
	-W clippy::semicolon_if_nothing_returned \
	-W clippy::similar_names \
	-W clippy::single_match_else \
	-W clippy::trivially_copy_pass_by_ref \
	-W clippy::unreadable-literal \
	-W clippy::unseparated-literal-suffix \
	-W clippy::unnested_or_patterns \
	-A clippy::wildcard_dependencies \
	-D warnings

ifneq ($(OS),Windows_NT)
    UNAME_S := $(shell uname -s)
    ifeq ($(UNAME_S),Darwin)
        CLIPPY_TARGETS += --target aarch64-apple-darwin
    endif
endif

CARGO_COMMAND = cargo

check:
	$(CARGO_COMMAND) fmt --all --check
	RUSTFLAGS="--cfg=web_sys_unstable_apis" $(CARGO_COMMAND) clippy $(CLIPPY_TARGETS) $(CLIPPY_PARAMS)

check-js:
	cd site && npm install && npm run check

format-js:
	cd site && npm install && npm run format

macos-app:
	rustup target add aarch64-apple-darwin x86_64-apple-darwin
	cargo install cargo-bundle
	cargo bundle --release --target x86_64-apple-darwin
	cargo bundle --release --target aarch64-apple-darwin
	cd target/aarch64-apple-darwin/release/bundle/osx && tar cf "Game of Life.app.tar" "Game of Life.app"
	cd target/x86_64-apple-darwin/release/bundle/osx && tar cf "Game of Life.app.tar" "Game of Life.app"

build-android:
	./gradlew $(GRADLE_BUILD_TASK)

run-android:
	./gradlew $(GRADLE_RUN_TASK)
	adb shell am start -n net.fornwall.wgpugameoflife/android.app.NativeActivity

uninstall-android:
	adb uninstall net.fornwall.wgpugameoflife

run-app:
	@cargo bundle --release &> /dev/null
	@open target/release/bundle/osx/"$(APP_NAME)".app

build-ios-simulator-app:
	cargo install cargo-bundle
	rustup target add aarch64-apple-ios-sim
	cargo bundle --target aarch64-apple-ios-sim

run-ios-simulator: build-ios-simulator-app
	xcrun simctl boot "iPhone 14" || echo "Perhaps already running"
	open /Applications/Xcode.app/Contents/Developer/Applications/Simulator.app
	xcrun simctl install booted "target/aarch64-apple-ios-sim/debug/bundle/ios/Game of Life.app"
	xcrun simctl launch --console booted "net.fornwall.wgpugameoflife"

generate-wasm:
	# --cfg=web_sys_unstable_apis is necessary for webgpu:
	# https://rustwasm.github.io/wasm-bindgen/api/web_sys/enum.GpuTextureFormat.html
	RUSTFLAGS="--cfg=web_sys_unstable_apis -C target-feature=$(WASM_TARGET_FEATURES)" \
		cargo build $(CARGO_BUILD_PROFILE) --target wasm32-unknown-unknown
	rm -Rf site/generated
	$(WASM_BINDGEN) --out-dir site/generated target/wasm32-unknown-unknown/$(WASM_DIR)/wgpu_game_of_life.wasm
	$(WASM_OPT) -o site/generated/wgpu_game_of_life_bg.wasm site/generated/wgpu_game_of_life_bg.wasm

wasm-size: generate-wasm
	ls -la site/generated/wgpu_game_of_life_bg.wasm

--run-devserver:
	cd site && npm run webpack serve -- --mode=development --open

--watch-and-build-wasm:
	cargo watch --ignore crates/wasm/site --shell '$(MAKE) generate-wasm'

serve-site: --run-devserver --watch-and-build-wasm ;

build-site: generate-wasm
	cd site && rm -Rf dist && npm install && NODE_ENV=production npm run webpack -- --mode=production

.PHONY: check macos-app android-apk run-app build-ios-simulator-app run-ios-simulator generate-wasm wasm-size --run-devserver --watch-and-build-wasm serve-site
