#!/bin/bash
cargo clean -Z unstable-options --profile coverage
RUSTFLAGS='-Zinstrument-coverage' cargo build -Z unstable-options --profile coverage
target=coverage
out_dir=target/$target/results
binname=portevalider

cover_case(){
    LLVM_PROFILE_FILE="$out_dir/json_test_suite$1.profraw" target/$target/$binname test_suite/test_parsing/$1
}
for f in $(ls test_suite/test_parsing | cat)
do
    cover_case $f > /dev/null 2> /dev/null
done
cargo-profdata -- merge -sparse $out_dir/*.profraw -o $out_dir/collected.profdata
rm $out_dir/*.profraw
cargo cov -- export \
    -Xdemangler=rustfilt \
    target/$target/portevalider \
    -instr-profile=$out_dir/collected.profdata \
    --format=lcov \
    > $out_dir/json_test_suite.info
cargo cov -- report \
    -Xdemangler=rustfilt \
    target/$target/portevalider \
    -instr-profile=$out_dir/collected.profdata \
    | grep TOTAL | awk '{print $4}'
 cargo tarpaulin --ignore-tests --out Lcov --output-dir target/tarpaulin
 cp target/tarpaulin/lcov.info $out_dir/tarpaulin.info
 lcov \
    -a $out_dir/json_test_suite.info \
    -a $out_dir/tarpaulin.info \
    -o lcov.info