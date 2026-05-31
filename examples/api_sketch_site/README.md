# Legacy API sketch fixture

This directory is retained as legacy regression coverage for PRD-001 and PRD-002 parser/rendering behavior.

It still uses older authoring forms such as `publisher.child`, `publisher.children`, `publisher.outline`, `publisher.bibliography`, and `publisher.ref`.
Those forms are not the PRD-003 target authoring model.

New PRD-003 examples should use `examples/prd003_discovery_site/` and ordinary Typst calls with `#publish(...)` and `#scope(...)`.
