# cjson-rust-rewrite

`cjson` is a Rust port of the cJSON 1.7.19 public API in `vendor/cJSON/cJSON.h`. Parse, print, minify, get, set, array helpers, object helpers, and delete live on one owned `Json` tree.

## Build

```
cargo build
cargo test
```

## Differential

`vendor/cJSON` is DaveGamble/cJSON commit `6d9f2443ab071f86e5d9b43025a40929ec41c46c`. `differential/run.sh` compiles `differential/oracle.c` against that source and compares each fixture with `cjson_print`.

The canonical bytes are `cJSON_PrintUnformatted`. That printer emits compact JSON and does not insert a space after `:` or `,`. Run the script with `LC_ALL=C`. The script sets that locale itself.

Reference nodes copy their children. `malloc` and `free` honor per-thread hooks. The tree allocates with the Rust allocator.
