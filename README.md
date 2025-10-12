Clean Line
===
<img width="1553" height="880" alt="2025-10-05_19-25" src="https://github.com/user-attachments/assets/0bc5f903-5d01-40ca-ba18-c03c456fd400" />

Game made for the Ludum Dare 58 Compo.

Build railroads, collect garbage, and take it to the recycling centers.

Fill all recycling centers to win.

- Mouse controls to build.
- Space to start/stop your train.
- R to reset the current level.
- H for in-game help.


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
# Serve debug build
./dev-server.sh

# Serve release build
./dev-server.sh --release
```

This will start the development web server, watch for builds and reload the browser window if there is a new one.

## Creating a release
To build and package a release:

```bash
chmod +x release.sh
./release.sh <target>
```

Available targets:
- `wasm` - WebAssembly build for browsers
- `windows` - Windows executable (x86_64-pc-windows-gnu)
- `linux-musl` - Linux with musl libc (x86_64-unknown-linux-musl)
- `linux-gnu` - Linux with glibc (x86_64-unknown-linux-gnu)

Example:
```bash
./release.sh wasm
./release.sh windows
./release.sh linux-gnu
```

This will build a release version and create a zip package in `build/<target>/` with the version from `Cargo.toml`.
