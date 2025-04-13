# dirhash-rs

## Testing
``` bash
cargo test
cargo test --lib # only unit tests
cargo test --no-fail-fast # don't stop after first failed test
cargo test -- --show-output # Show captured stdout of successful tests

cargo nextest run
cargo nextest run --lib # only unit tests
cargo nextest run --no-fail-fast # don't stop after first failed test
```

## Benchmarking
Mocks for testing are normally included when during testing. Benchmarking doesn't enable the `test` and therefore the mocks are not available. To enable them, the feature `test-mocks` must be enabled:
``` bash
cargo bench --features test-mocks
```

## Test Coverage
``` bash
cargo llvm-cov # text output
cargo llvm-cov --open # HTML output
cargo llvm-cov --lib # only unit tests

cargo llvm-cov nextest # text output
cargo llvm-cov nextest --open # HTML output
cargo llvm-cov nextest --lib # only unit tests
```

## Profiling
Grant temporary access for perf events:
``` bash
echo '1' | sudo tee /proc/sys/kernel/perf_event_paranoid
```

Compile and start recording:
``` bash
cargo build --profile profiling # maybe some feature needs to be enabled as well
samply record ./target/profiling/
```
