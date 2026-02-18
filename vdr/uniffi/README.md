# Uni FFI for Indy Besu VDR

`Indy-VDR` library uses [uniffi](https://mozilla.github.io/uniffi-rs/) to generate bindings for **Kotlin**, **Python**, **Swift** languages.

### Requirements

* Rust of version 1.79.0 or higher.

### Build

In order to generate language specific bindings, run the following commands with replacing a target
language `<kotlin|python|swift>`:

```
cargo build --release
cargo run --bin uniffi-bindgen generate --library target/release/libindy_besu_vdr_uniffi.dylib --language <kotlin|python|swift> --out-dir out
```

The check `out` directory which will contain generated bindings.
