# lepcc

Rust bindings for [Esri's LEPCC](https://github.com/Esri/lepcc) (Limited Error Point Cloud Compression) library.

LEPCC is the codec behind I3S PointCloud layers. When you fetch a geometry or attribute blob from a `lepcc-xyz`, `lepcc-rgb`, or `lepcc-intensity` resource, this is what decodes it.

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
lepcc = { git = "https://github.com/calebbuffa/lepcc-rs" }
```

Then decode blobs you've fetched from an I3S service:

```rust
use lepcc::Context;

// XYZ positions
let ctx = Context::new();
let points: Vec<[f64; 3]> = ctx.decode_xyz(&blob)?;

// RGB colors
let ctx = Context::new();
let colors: Vec<[u8; 3]> = ctx.decode_rgb(&blob)?;

// Intensity
let ctx = Context::new();
let intensity: Vec<u16> = ctx.decode_intensity(&blob)?;
```

The `Context` handles the underlying C allocation and frees it on drop.

## Building

The crate compiles the LEPCC C++ sources directly via the `cc` crate, so you just need a C++ compiler on your PATH — no separate install of the LEPCC library required.

```sh
git clone --recurse-submodules https://github.com/calebbuffa/lepcc-rs
cd lepcc-rs
cargo build
```

## License

Apache-2.0 — same as the bundled LEPCC library. See [`extern/lepcc/LICENSE.TXT`](extern/lepcc/LICENSE.TXT).
