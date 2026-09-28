#!/bin/sh
set -u
root=$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)
cd "$root"
export LC_ALL=C

echo "canonical form: cJSON_PrintUnformatted bytes. Compact JSON with no space after ':' or ','."

gcc -O2 -std=c11 -o differential/cjson_oracle differential/oracle.c vendor/cJSON/cJSON.c -Ivendor/cJSON -lm
gcc_ec=$?
if [ "$gcc_ec" -ne 0 ]; then
    echo "gcc exit $gcc_ec"
    exit "$gcc_ec"
fi

cargo build --bin cjson_print
cargo_ec=$?
if [ "$cargo_ec" -ne 0 ]; then
    echo "cargo build exit $cargo_ec"
    exit "$cargo_ec"
fi

pass=0
fail=0
panic=0
for file in differential/fixtures/*.json differential/fixtures/upstream/*.json; do
    rust_out=$(mktemp)
    c_out=$(mktemp)
    ./target/debug/cjson_print < "$file" > "$rust_out"
    rust_ec=$?
    ./differential/cjson_oracle < "$file" > "$c_out"
    c_ec=$?
    if [ "$rust_ec" -eq 101 ]; then
        panic=$((panic + 1))
    fi
    if [ "$rust_ec" -eq "$c_ec" ] && cmp -s "$rust_out" "$c_out"; then
        pass=$((pass + 1))
        printf 'PASS %s\n' "$file"
    else
        fail=$((fail + 1))
        printf 'FAIL %s rust_ec=%s c_ec=%s\n' "$file" "$rust_ec" "$c_ec"
        diff -u "$c_out" "$rust_out" || true
    fi
    rm -f "$rust_out" "$c_out"
done

printf 'pass=%s fail=%s panic=%s\n' "$pass" "$fail" "$panic"
if [ "$fail" -ne 0 ] || [ "$panic" -ne 0 ]; then
    exit 1
fi
exit 0
