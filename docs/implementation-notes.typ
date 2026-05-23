= Implementation notes

== Core entities inspect
- Attempted `cargo test --test inspect_api_sketch` before any accepted `insta` snapshot existed. It compiled and produced `tests/snapshots/inspect_api_sketch__inspect_api_sketch_snapshot_is_stable.snap.new`, then failed because the snapshot was unreviewed. Correct approach: accept the generated snapshot with `INSTA_UPDATE=always cargo test --test inspect_api_sketch`, then rerun the focused test normally.
- PR #57 review follow-up: `docs/api-sketch.typ` is an aspirational API sketch, so it can show the intended `#let nav = publisher.scope(...)` / `#nav.suppress()` shape even while the milestone parser only implements `publisher.nav.suppress()`. Parser call dispatch now routes exact `publisher.*` calls through a `PublisherCallHandler` trait boundary instead of a widening `match` over every supported call.
