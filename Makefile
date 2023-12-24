VERSION=$(shell grep ^version Cargo.toml|cut -d\" -f2)

all: test

tag:
	git tag -a v${VERSION} -m v${VERSION}
	git push origin --tags

release: tag pkg

pkg:
	rm -rf _build
	mkdir -p _build
	cross build --target x86_64-unknown-linux-musl --release
	cross build --target aarch64-unknown-linux-musl --release
	cd target/x86_64-unknown-linux-musl/release && cp latencymon ../../../_build/latencymon-${VERSION}-x86_64
	cd target/aarch64-unknown-linux-musl/release && \
		aarch64-linux-gnu-strip latencymon && \
		cp latencymon ../../../_build/latencymon-${VERSION}-aarch64
	cd _build && echo "" | gh release create v$(VERSION) -t "v$(VERSION)" \
		latencymon-${VERSION}-x86_64 \
		latencymon-${VERSION}-aarch64
