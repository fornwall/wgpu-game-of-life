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

ifneq ($(OS),Windows_NT)
    UNAME_S := $(shell uname -s)
    ifeq ($(UNAME_S),Darwin)
        CLIPPY_TARGETS += --target aarch64-apple-darwin
    endif
endif

CARGO_COMMAND = cargo

check:
	$(CARGO_COMMAND) fmt --all --check
	$(CARGO_COMMAND) clippy $(CLIPPY_TARGETS) $(CLIPPY_PARAMS)

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
	sleep 2
	adb logcat -v color --pid=`adb shell pidof -s net.fornwall.wgpugameoflife`

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

build-wasm:
	RUSTFLAGS="-C target-feature=$(WASM_TARGET_FEATURES)" \
		cargo build $(CARGO_BUILD_PROFILE) --target wasm32-unknown-unknown
	rm -Rf site/generated
	$(WASM_BINDGEN) --out-dir site/generated target/wasm32-unknown-unknown/$(WASM_DIR)/wgpu_game_of_life.wasm
	$(WASM_OPT) -o site/generated/wgpu_game_of_life_bg.wasm site/generated/wgpu_game_of_life_bg.wasm

wasm-size: build-wasm
	ls -la site/generated/wgpu_game_of_life_bg.wasm

--run-devserver:
	cd site && npm run webpack serve -- --mode=development --open

--watch-and-build-wasm:
	cargo watch --ignore crates/wasm/site --shell '$(MAKE) build-wasm'

build-web: build-wasm
	cd site && rm -Rf dist && npm install && NODE_ENV=production npm run webpack -- --mode=production

run-web: --run-devserver --watch-and-build-wasm ;

.PHONY: check check-js format-js macos-app build-android run-android uninstall-android run-app build-ios-simulator-app run-ios-simulator build-wasm wasm-size --run-devserver --watch-and-build-wasm build-web run-web
