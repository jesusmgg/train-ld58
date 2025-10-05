Clean Line
===

Game made for the Ludum Dare 58 Compo.

Description will go here.

## Building
The game requires only the standard Rust toolchain with the `wasm32-unknown-unknown` target installed.

```bash
# Produces a debug build at `target/wasm32-unknown-unknown/debug`
cargo build

 # Produces a production build at `target/wasm32-unknown-unknown/release`
cargo build --release
```

## Development server
Requires [live-server](https://www.npmjs.com/package/live-server).

```bash
chmod +x dev-server.sh
./dev-server.sh
```

This will start the development web server, watch for debug builds and reload the browser window if there is a new one.
