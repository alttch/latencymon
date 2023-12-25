VERSION=$(shell grep ^version Cargo.toml|cut -d\" -f2)

all: test

tag:
	git tag -a v${VERSION} -m v${VERSION}
	git push origin --tags

release: tag pkg

clean:
	cargo clean
	CARGO_TARGET_DIR=target-aarch64-musl ~/.cargo/bin/cargo clean
	CARGO_TARGET_DIR=target-x86_64-musl ~/.cargo/bin/cargo clean

pkg:
	rm -rf _build
	mkdir -p _build
	CARGO_TARGET_DIR=target-x86_64-musl cross build --target x86_64-unknown-linux-musl --release
	CARGO_TARGET_DIR=target-aarch64-musl cross build --target aarch64-unknown-linux-musl --release
	cd target-x86_64-musl/x86_64-unknown-linux-musl/release && \
		cp latencymon ../../../_build/latencymon-${VERSION}-linux-x86_64
	cd target-aarch64-musl/aarch64-unknown-linux-musl/release && \
		aarch64-linux-gnu-strip latencymon && \
		cp latencymon ../../../_build/latencymon-${VERSION}-linux-aarch64
	cd _build && echo "" | gh release create v$(VERSION) -t "v$(VERSION)" \
		latencymon-${VERSION}-linux-x86_64 \
		latencymon-${VERSION}-linux-aarch64
