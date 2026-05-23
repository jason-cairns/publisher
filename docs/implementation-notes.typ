= Implementation notes

== Core entities inspect
- Attempted `cargo test --test inspect_api_sketch` before any accepted `insta` snapshot existed. It compiled and produced `tests/snapshots/inspect_api_sketch__inspect_api_sketch_snapshot_is_stable.snap.new`, then failed because the snapshot was unreviewed. Correct approach: accept the generated snapshot with `INSTA_UPDATE=always cargo test --test inspect_api_sketch`, then rerun the focused test normally.
