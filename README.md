Create test file:

```
cargo run --bin create_test_file
```


Run the script:

``` 
cargo build --release && ./target/release/parser
``` 


Generate flramegraph:
code
```
sudo sysctl kernel.perf_event_paranoid=1

cargo flamegraph --dev --bin parser
```

Memory profiling with ´dhat´. The resulting JSON file can be viewed on https://nnethercote.github.io/dh_view/dh_view.html.






SIMD

```
rustup toolchain install nightly





rustup run nightly cargo build --release && ./target/release/parser

```