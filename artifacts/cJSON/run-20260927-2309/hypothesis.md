# hypothesis: cJSON run-20260927-2309

## target
- behavioral oracle: DaveGamble/cJSON (C → Rust)
- host repo: https://github.com/TheOnlyFusionCube/cjson-rust-rewrite
- starting_ref: main @ e5a852c425b39ed0e18e58235f983b604c991e45 (README seed)
- scope: full public API surface of cJSON.c / cJSON.h as one crate (parse, print, minify, get/set, array/object helpers, delete). no extras beyond upstream API.
- out of scope: streaming parsers, serde interop, async, Origin new_repo.

## model
- CloudAgent default / inherit-parent.
- style bar: poteto-mode + unslop when writing.

## chunking
- single chunk: one rust crate mirroring cJSON public API.
- vendor/fetch upstream cJSON sources into the host only as needed for the C oracle / differential harness.

## success gate (must all hold)
1. `cargo build` clean.
2. `cargo test` green.
3. differential: ≥20 fixtures (null/bool/number/string/array/object, nested, escaped strings, upstream tests/ if present) — rust parse→canonical-print matches original cJSON C binary (byte-stable or agreed canonical form in test.log).
4. no panics on those fixtures (miri optional; note if skipped).

## cost objective
- meet the gate. if artifacts/cJSON/best.json exists, beat it on tokens/$/wall-clock. else candidate for best.json only after rust-eval keep.

## artifacts
- artifacts/cJSON/run-20260927-2309/{hypothesis.md, diff.patch, build.log, test.log, metrics.json}
- metrics.json: model, tokens_in, tokens_out, usd, wall_s, compile_ok, tests_passed, tests_failed[], cloudagent_id — never invent.

## handoff
- rust-runner: ONE CloudAgent on https://github.com/TheOnlyFusionCube/cjson-rust-rewrite (starting_ref main). not Origin. if refuse: stop + exact error, no loop.
- rust-eval: score only after real artifacts (ignore prior launch_refused stubs).
