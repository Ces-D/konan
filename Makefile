# Requires: cargo, zip
TARGET_ARCH = x86_64-unknown-linux-musl
PREFIX = lambda_konan

ifneq (,$(wildcard ./.env))
    include .env
    export
endif
# Adjust this if your workspace root is different from the Makefile location
# In this config, we assume Makefile is at the root with Cargo.toml
WORKSPACE_ROOT = .

build-all: clean
	mkdir -p build
	$(MAKE) build-web 
	
	@echo "Building Functions..."
	$(MAKE) build-func FUNC=habits
	$(MAKE) build-func FUNC=message
	$(MAKE) build-func FUNC=outline

build-func:
	@echo "Building $(PREFIX)_$(FUNC)..."
	
	# 1. Build specific package (-p) from workspace root
	cargo build --release --target $(TARGET_ARCH) -p $(PREFIX)_$(FUNC)
	
	# 2. Copy binary from SHARED target directory to build/bootstrap
	cp target/$(TARGET_ARCH)/release/$(PREFIX)_$(FUNC) build/bootstrap
	
	# 3. Zip it
	cd build && zip $(FUNC).zip bootstrap && rm bootstrap

# Build the Svelte app and place static output under build/site
build-web:
	@echo "Building web app.."
	cd konan_web && pnpm build 
	
clean:
	rm -rf build
	cargo clean
