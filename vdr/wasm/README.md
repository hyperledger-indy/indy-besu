# WASM FFI for Indy Besu VDR

### Requirements

* Rust of version 1.79.0 or higher.
* Wasm-pack: https://rustwasm.github.io/wasm-pack/installer/

### Build

#### NodeJS bindings

```
wasm-pack build --target=nodejs
```

#### WASM (consumed in browser without bundler usage) bindings

```
wasm-pack build --target=web
```

#### Bindings to use with webpack

```
wasm-pack build
```

### Run demo

#### NodeJS demo

* Build NodeJS bindings from `wasm` directory
  ```
  wasm-pack build --target=nodejs
  ```
* Install dependencies
  ```
  cd demo/node
  yarn install
  ```
* Run demo
  ```
  yarn start
  ```