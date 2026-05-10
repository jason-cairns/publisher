TYPST ?= typst
PROLOG ?= scryer-prolog
PYTHON ?= python3
SERVE_PORT ?= 8000

SRC_DIR := src
BUILD_DIR := build
INTERMEDIATE_DIR := $(BUILD_DIR)/html
PUBLIC_DIR := public
ROOT_SOURCE := index.typ
PDF_TYP_SOURCE := $(BUILD_DIR)/site.typ
PUBLICATION_TYP_FILES := $(shell find $(SRC_DIR) -name '*.typ' ! -name '_*' -print | sort)
PUBLICATION_SOURCES := $(patsubst $(SRC_DIR)/%,%,$(PUBLICATION_TYP_FILES))

export TYPST

.PHONY: build clean serve test

build: $(PUBLIC_DIR)/.stamp $(PUBLIC_DIR)/site.pdf

$(INTERMEDIATE_DIR)/.stamp: $(PUBLICATION_TYP_FILES) $(SRC_DIR)/_publication.typ
	rm -rf $(INTERMEDIATE_DIR)
	mkdir -p $(INTERMEDIATE_DIR)
	set -eu; for src in $(PUBLICATION_SOURCES); do \
		out="$(INTERMEDIATE_DIR)/$${src%.typ}.html"; \
		mkdir -p "$$(dirname "$$out")"; \
		$(TYPST) compile --features html --input render-target=html --root $(SRC_DIR) "$(SRC_DIR)/$$src" "$$out"; \
	done
	touch $@

$(PUBLIC_DIR)/.stamp: $(INTERMEDIATE_DIR)/.stamp site.pl
	rm -rf $(PUBLIC_DIR)
	mkdir -p $(PUBLIC_DIR)
	set -eu; for src in $(PUBLICATION_SOURCES); do \
		if [ "$$src" = "$(ROOT_SOURCE)" ]; then out="$(PUBLIC_DIR)/index.html"; else out="$(PUBLIC_DIR)/$${src%.typ}/index.html"; fi; \
		mkdir -p "$$(dirname "$$out")"; \
	done
	$(PROLOG) site.pl -- $(SRC_DIR) $(INTERMEDIATE_DIR) $(PUBLIC_DIR) $(PDF_TYP_SOURCE) $(ROOT_SOURCE) $(PUBLICATION_SOURCES)
	touch $@

$(PDF_TYP_SOURCE): $(PUBLIC_DIR)/.stamp

$(PUBLIC_DIR)/site.pdf: $(PDF_TYP_SOURCE) $(PUBLICATION_TYP_FILES) $(SRC_DIR)/_publication.typ
	mkdir -p $(PUBLIC_DIR)
	$(TYPST) compile --root . $(PDF_TYP_SOURCE) $@

test: build
	./test/build.sh

serve: build
	@printf 'Serving public at http://localhost:%s/\n' "$(SERVE_PORT)"
	cd $(PUBLIC_DIR) && $(PYTHON) -m http.server $(SERVE_PORT)

clean:
	rm -rf $(BUILD_DIR) $(PUBLIC_DIR)
