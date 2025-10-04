Planets!
===

Puzzle game made completely from scratch in 48 hours for the [Alakajam 21](https://alakajam.com/21st-alakajam) game jam. The theme was "Gravity".

The objective is to create stable planet systems. Each planet has different gravity directions that affect each other planet in range. When every planet is clear of every other planet's influence, the system is considered stable.

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
