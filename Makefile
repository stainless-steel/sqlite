.PHONY: all
all:

.PHONY: publish
publish:
	git tag --force v$$(cat Cargo.toml | grep version | head -1 | cut -d\" -f2)
	git push --tags
	cargo publish
