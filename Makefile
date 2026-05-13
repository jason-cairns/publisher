TYPST ?= typst
PYTHON ?= python3
SERVE_PORT ?= 8000

SRC_DIR := src
BUILD_DIR := build
INTERMEDIATE_DIR := $(BUILD_DIR)/html
PUBLIC_DIR := public
ROOT_SOURCE := index.typ
PDF_TYP_SOURCE := $(BUILD_DIR)/site.typ

export TYPST

.PHONY: build clean serve test

build:
	uv run site build --src $(SRC_DIR) --html $(INTERMEDIATE_DIR) --out $(PUBLIC_DIR) --pdf-typ $(PDF_TYP_SOURCE) --pdf-out $(PUBLIC_DIR)/site.pdf --root $(ROOT_SOURCE)

test:
	uv run pytest

serve: build
	@printf 'Serving public at http://localhost:%s/\n' "$(SERVE_PORT)"
	cd $(PUBLIC_DIR) && $(PYTHON) -m http.server $(SERVE_PORT)

clean:
	rm -rf $(BUILD_DIR) $(PUBLIC_DIR)
