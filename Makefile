TYPST ?= typst
PROLOG ?= scryer-prolog

SRC_DIR := src
BUILD_DIR := build
INTERMEDIATE_DIR := $(BUILD_DIR)/html
PUBLIC_DIR := public

.PHONY: build clean test

build: $(PUBLIC_DIR)/index.html $(PUBLIC_DIR)/site.pdf

$(INTERMEDIATE_DIR)/index.html: $(SRC_DIR)/index.typ
	mkdir -p $(INTERMEDIATE_DIR)
	$(TYPST) compile --features html $< $@

$(PUBLIC_DIR)/index.html: $(INTERMEDIATE_DIR)/index.html site.pl
	mkdir -p $(PUBLIC_DIR)
	$(PROLOG) site.pl -- $(INTERMEDIATE_DIR) $(PUBLIC_DIR)

$(PUBLIC_DIR)/site.pdf: $(SRC_DIR)/_site.typ $(SRC_DIR)/index.typ
	mkdir -p $(PUBLIC_DIR)
	$(TYPST) compile $(SRC_DIR)/_site.typ $@

test: build
	./test/build.sh

clean:
	rm -rf $(BUILD_DIR) $(PUBLIC_DIR)

