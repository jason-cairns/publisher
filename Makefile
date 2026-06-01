.PHONY: install install-bin install-typst

# Install both the publisher binary and the Typst marker helpers.
install: install-bin install-typst

# Install the `publisher` command from this checkout.
install-bin:
	cargo install --path . --locked

# Install the @local/publisher Typst package into Typst's local package dir.
# Override the data dir with PUBLISHER_TYPST_DATA_DIR.
install-typst:
	./scripts/install-typst-package.sh
