//! Safe Rust bindings for the [Esri LEPCC](https://github.com/Esri/lepcc)
//! point-cloud compression library.
//!
//! LEPCC (Limited Error Point Cloud Compression) is the codec used by the I3S
//! PointCloud layer type to compress:
//!
//! * **XYZ** — 3-D coordinates (`lepcc-xyz` blobs)
//! * **RGB** — 8-bit-per-channel colour (`lepcc-rgb` blobs)
//! * **Intensity** — 16-bit intensity (`lepcc-intensity` blobs)
//! * **FlagBytes** — per-point classification flags
//!
//! # Usage
//!
//! ```no_run
//! use lepcc::Context;
//!
//! let blob: Vec<u8> = todo!("fetch lepcc-xyz blob from I3S");
//! let ctx = Context::new();
//! let points = ctx.decode_xyz(&blob).unwrap();
//! // points: Vec<[f64; 3]>
//! ```

mod sys;

use std::ffi::c_int;

use thiserror::Error;

/// Errors returned by LEPCC operations.
#[derive(Debug, Error)]
pub enum LepccError {
    #[error("LEPCC returned status code {0}")]
    Status(u32),
    #[error("input buffer too large for c_int")]
    BufferTooLarge,
}

pub type Result<T> = std::result::Result<T, LepccError>;

fn buf_len(data: &[u8]) -> Result<c_int> {
    c_int::try_from(data.len()).map_err(|_| LepccError::BufferTooLarge)
}

fn check(status: u32) -> Result<()> {
    if status == 0 {
        Ok(())
    } else {
        Err(LepccError::Status(status))
    }
}

/// RAII wrapper around a `lepcc_ContextHdl`.
///
/// Each method operates on a fresh decode pass — the underlying C context is
/// reset by each call.  `Context` is **not** `Send + Sync` because the raw
/// pointer is single-threaded; create one per thread / per decode call.
pub struct Context {
    hdl: sys::lepcc_ContextHdl,
}

impl Context {
    /// Create a new LEPCC context.
    ///
    /// # Panics
    ///
    /// Panics if the underlying C allocation fails (returns null).
    pub fn new() -> Self {
        let hdl = unsafe { sys::lepcc_createContext() };
        assert!(!hdl.is_null(), "lepcc_createContext returned null");
        Self { hdl }
    }

    /// Identify the type and byte-length of the next blob in `data`.
    ///
    /// Returns `(blob_type, blob_size_in_bytes)`.
    pub fn blob_info(&self, data: &[u8]) -> Result<(u32, u32)> {
        let mut blob_type = 0u32;
        let mut blob_size = 0u32;
        let status = unsafe {
            sys::lepcc_getBlobInfo(
                self.hdl,
                data.as_ptr(),
                buf_len(data)?,
                &mut blob_type,
                &mut blob_size,
            )
        };
        check(status)?;
        Ok((blob_type, blob_size))
    }

    /// Decode a `lepcc-xyz` blob into a vector of `[x, y, z]` coordinates.
    pub fn decode_xyz(&self, data: &[u8]) -> Result<Vec<[f64; 3]>> {
        let len = buf_len(data)?;

        // Query point count
        let mut n_pts = 0u32;
        let status = unsafe { sys::lepcc_getPointCount(self.hdl, data.as_ptr(), len, &mut n_pts) };
        check(status)?;

        let mut out = vec![[0.0f64; 3]; n_pts as usize];
        let mut ptr = data.as_ptr();
        let mut n_out = n_pts;
        let status = unsafe {
            sys::lepcc_decodeXYZ(
                self.hdl,
                &mut ptr,
                len,
                &mut n_out,
                out.as_mut_ptr() as *mut f64,
            )
        };
        check(status)?;
        Ok(out)
    }

    /// Decode a `lepcc-rgb` blob into a vector of `[r, g, b]` byte triples.
    pub fn decode_rgb(&self, data: &[u8]) -> Result<Vec<[u8; 3]>> {
        let len = buf_len(data)?;

        let mut n_rgb = 0u32;
        let status = unsafe { sys::lepcc_getRGBCount(self.hdl, data.as_ptr(), len, &mut n_rgb) };
        check(status)?;

        let mut out = vec![[0u8; 3]; n_rgb as usize];
        let mut ptr = data.as_ptr();
        let mut n_out = n_rgb;
        let status = unsafe {
            sys::lepcc_decodeRGB(
                self.hdl,
                &mut ptr,
                len,
                &mut n_out,
                out.as_mut_ptr() as *mut u8,
            )
        };
        check(status)?;
        Ok(out)
    }

    /// Decode a `lepcc-intensity` blob into a vector of `u16` intensity values.
    pub fn decode_intensity(&self, data: &[u8]) -> Result<Vec<u16>> {
        let len = buf_len(data)?;

        let mut n_vals = 0u32;
        let status =
            unsafe { sys::lepcc_getIntensityCount(self.hdl, data.as_ptr(), len, &mut n_vals) };
        check(status)?;

        let mut out = vec![0u16; n_vals as usize];
        let mut ptr = data.as_ptr();
        let mut n_out = n_vals;
        let status = unsafe {
            sys::lepcc_decodeIntensity(self.hdl, &mut ptr, len, &mut n_out, out.as_mut_ptr())
        };
        check(status)?;
        Ok(out)
    }

    /// Decode a `lepcc-flagbytes` blob into a vector of classification bytes.
    pub fn decode_flag_bytes(&self, data: &[u8]) -> Result<Vec<u8>> {
        let len = buf_len(data)?;

        let mut n_vals = 0u32;
        let status =
            unsafe { sys::lepcc_getFlagByteCount(self.hdl, data.as_ptr(), len, &mut n_vals) };
        check(status)?;

        let mut out = vec![0u8; n_vals as usize];
        let mut ptr = data.as_ptr();
        let mut n_out = n_vals;
        let status = unsafe {
            sys::lepcc_decodeFlagBytes(self.hdl, &mut ptr, len, &mut n_out, out.as_mut_ptr())
        };
        check(status)?;
        Ok(out)
    }

    /// Encode XYZ coordinates into a LEPCC blob.
    ///
    /// `max_err` is the maximum absolute error per axis (e.g. `0.001` metres).
    pub fn encode_xyz(&self, points: &[[f64; 3]], max_err: f64) -> Result<Vec<u8>> {
        let n = points.len() as u32;
        let raw = points.as_ptr() as *const f64;

        let mut n_bytes = 0u32;
        let status = unsafe {
            sys::lepcc_computeCompressedSizeXYZ(
                self.hdl,
                n,
                raw,
                max_err,
                max_err,
                max_err,
                &mut n_bytes,
                std::ptr::null_mut(), // order_out — not needed for decode-only use
            )
        };
        check(status)?;

        let mut buf = vec![0u8; n_bytes as usize];
        let mut ptr = buf.as_mut_ptr();
        let status = unsafe { sys::lepcc_encodeXYZ(self.hdl, &mut ptr, n_bytes as c_int) };
        check(status)?;
        Ok(buf)
    }

    /// Encode RGB colours into a LEPCC blob.
    pub fn encode_rgb(&self, colours: &[[u8; 3]]) -> Result<Vec<u8>> {
        let n = colours.len() as u32;
        let raw = colours.as_ptr() as *const u8;

        let mut n_bytes = 0u32;
        let status = unsafe { sys::lepcc_computeCompressedSizeRGB(self.hdl, n, raw, &mut n_bytes) };
        check(status)?;

        let mut buf = vec![0u8; n_bytes as usize];
        let mut ptr = buf.as_mut_ptr();
        let status = unsafe { sys::lepcc_encodeRGB(self.hdl, &mut ptr, n_bytes as c_int) };
        check(status)?;
        Ok(buf)
    }

    /// Encode intensity values into a LEPCC blob.
    pub fn encode_intensity(&self, values: &[u16]) -> Result<Vec<u8>> {
        let n = values.len() as u32;

        let mut n_bytes = 0u32;
        let status = unsafe {
            sys::lepcc_computeCompressedSizeIntensity(self.hdl, n, values.as_ptr(), &mut n_bytes)
        };
        check(status)?;

        let mut buf = vec![0u8; n_bytes as usize];
        let mut ptr = buf.as_mut_ptr();
        let status = unsafe {
            sys::lepcc_encodeIntensity(self.hdl, &mut ptr, n_bytes as c_int, values.as_ptr(), n)
        };
        check(status)?;
        Ok(buf)
    }

    /// Encode classification flag bytes into a LEPCC blob.
    pub fn encode_flag_bytes(&self, flags: &[u8]) -> Result<Vec<u8>> {
        let n = flags.len() as u32;

        let mut n_bytes = 0u32;
        let status = unsafe {
            sys::lepcc_computeCompressedSizeFlagBytes(self.hdl, n, flags.as_ptr(), &mut n_bytes)
        };
        check(status)?;

        let mut buf = vec![0u8; n_bytes as usize];
        let mut ptr = buf.as_mut_ptr();
        let status = unsafe {
            sys::lepcc_encodeFlagBytes(self.hdl, &mut ptr, n_bytes as c_int, flags.as_ptr(), n)
        };
        check(status)?;
        Ok(buf)
    }
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe { sys::lepcc_deleteContext(&mut self.hdl) };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_points() -> Vec<[f64; 3]> {
        vec![
            [0.0, 0.0, 0.0],
            [1.0, 2.0, 3.0],
            [100.0, 200.0, 50.5],
            [-10.5, 300.0, 0.001],
        ]
    }

    #[test]
    fn roundtrip_xyz() {
        let ctx = Context::new();
        let pts = sample_points();
        let blob = ctx.encode_xyz(&pts, 0.001).expect("encode_xyz failed");
        assert!(!blob.is_empty());

        let ctx2 = Context::new();
        let decoded = ctx2.decode_xyz(&blob).expect("decode_xyz failed");
        assert_eq!(decoded.len(), pts.len());
        for (orig, dec) in pts.iter().zip(decoded.iter()) {
            assert!(
                (orig[0] - dec[0]).abs() <= 0.002,
                "x mismatch: {} vs {}",
                orig[0],
                dec[0]
            );
            assert!(
                (orig[1] - dec[1]).abs() <= 0.002,
                "y mismatch: {} vs {}",
                orig[1],
                dec[1]
            );
            assert!(
                (orig[2] - dec[2]).abs() <= 0.002,
                "z mismatch: {} vs {}",
                orig[2],
                dec[2]
            );
        }
    }

    #[test]
    fn roundtrip_rgb() {
        let ctx = Context::new();
        let colours: Vec<[u8; 3]> = vec![[255, 0, 0], [0, 255, 0], [0, 0, 255], [128, 128, 128]];
        let blob = ctx.encode_rgb(&colours).expect("encode_rgb failed");
        assert!(!blob.is_empty());

        let ctx2 = Context::new();
        let decoded = ctx2.decode_rgb(&blob).expect("decode_rgb failed");
        assert_eq!(decoded, colours);
    }

    #[test]
    fn roundtrip_intensity() {
        let ctx = Context::new();
        let values: Vec<u16> = vec![0, 1000, 32768, 65535, 512];
        let blob = ctx
            .encode_intensity(&values)
            .expect("encode_intensity failed");
        assert!(!blob.is_empty());

        let ctx2 = Context::new();
        let decoded = ctx2
            .decode_intensity(&blob)
            .expect("decode_intensity failed");
        assert_eq!(decoded, values);
    }

    #[test]
    fn roundtrip_flag_bytes() {
        let ctx = Context::new();
        let flags: Vec<u8> = vec![0, 1, 2, 64, 128, 255];
        let blob = ctx
            .encode_flag_bytes(&flags)
            .expect("encode_flag_bytes failed");
        assert!(!blob.is_empty());

        let ctx2 = Context::new();
        let decoded = ctx2
            .decode_flag_bytes(&blob)
            .expect("decode_flag_bytes failed");
        assert_eq!(decoded, flags);
    }

    #[test]
    fn blob_info_xyz() {
        let ctx = Context::new();
        let pts = sample_points();
        let blob = ctx.encode_xyz(&pts, 0.001).unwrap();

        let ctx2 = Context::new();
        let (blob_type, blob_size) = ctx2.blob_info(&blob).unwrap();
        assert!(blob_size > 0);
        // blob_type 0 = XYZ (indexed in encounter order: 0=XYZ, 1=RGB, 2=Intensity)
        assert_eq!(blob_type, 0, "unexpected blob type {blob_type}");
    }

    /// Paths to the test files bundled under `extern/lepcc/testData/`.
    const SLPK: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/extern/lepcc/testData/SMALL_AUTZEN_LAS_All.slpk"
    );
    const GT_BIN: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/extern/lepcc/testData/SMALL_AUTZEN_LAS_All.bin"
    );

    fn read_file(path: &str) -> Vec<u8> {
        use std::io::Read;
        let mut f =
            std::fs::File::open(path).unwrap_or_else(|_| panic!("test data not found: {path}"));
        let mut buf = Vec::new();
        f.read_to_end(&mut buf).unwrap();
        buf
    }

    // The `.bin` ground-truth file stores results in SLPK blob-encounter order.
    // Each block is:  u32 count (little-endian)  +  count × stride raw bytes.
    fn read_gt_block<'a>(cursor: &mut &'a [u8], stride: usize) -> (u32, &'a [u8]) {
        let n = u32::from_le_bytes(cursor[..4].try_into().unwrap());
        *cursor = &cursor[4..];
        let bytes = n as usize * stride;
        let block = &cursor[..bytes];
        *cursor = &cursor[bytes..];
        (n, block)
    }

    fn bytes_as_f64_le(bytes: &[u8]) -> Vec<f64> {
        bytes
            .chunks_exact(8)
            .map(|b| f64::from_le_bytes(b.try_into().unwrap()))
            .collect()
    }

    fn bytes_as_u16_le(bytes: &[u8]) -> Vec<u16> {
        bytes
            .chunks_exact(2)
            .map(|b| u16::from_le_bytes(b.try_into().unwrap()))
            .collect()
    }

    /// Mirrors `Test_C_Api.cpp`: scan the SLPK for XYZ / RGB / Intensity blobs,
    /// decode each one, and compare against the pre-computed `.bin` ground truth.
    ///
    /// The scan uses `blob_info()` to jump over each blob after decoding it,
    /// preventing false-positive magic-string matches inside compressed payloads.
    ///
    /// Requires the `extern/lepcc/testData/` files (present when the git submodule
    /// is initialised). Run with `cargo test -- --ignored` to include this test.
    #[test]
    #[ignore = "requires extern/lepcc/testData/ from the git submodule"]
    fn decode_slpk_matches_ground_truth() {
        let slpk = read_file(SLPK);
        let gt = read_file(GT_BIN);
        let mut gt_cursor: &[u8] = &gt;

        // The three magic strings the C++ test searches for (10 bytes each).
        const MAGIC_LEN: usize = 10;
        const MAGICS: [(&[u8; MAGIC_LEN], &str); 3] = [
            (b"LEPCC     ", "xyz"),
            (b"ClusterRGB", "rgb"),
            (b"Intensity ", "intensity"),
        ];

        let info_size = unsafe { sys::lepcc_getBlobInfoSize() } as usize;

        let mut pos = 0usize;
        let mut blob_count = 0u32;

        while pos + MAGIC_LEN <= slpk.len() {
            let mut matched = false;
            for &(magic, kind) in &MAGICS {
                if &slpk[pos..pos + MAGIC_LEN] != magic.as_ref() {
                    continue;
                }

                let remaining = &slpk[pos..];
                let ctx = Context::new();

                // Get the blob size so we can jump past it.
                let blob_size = if remaining.len() >= info_size {
                    ctx.blob_info(remaining)
                        .map(|(_, sz)| sz as usize)
                        .unwrap_or(MAGIC_LEN)
                } else {
                    MAGIC_LEN
                };

                match kind {
                    "xyz" => {
                        let pts = ctx.decode_xyz(remaining).expect("decode_xyz failed");
                        let (n_gt, gt_bytes) = read_gt_block(&mut gt_cursor, 24); // 3 × f64
                        let gt_flat = bytes_as_f64_le(gt_bytes);
                        assert_eq!(pts.len(), n_gt as usize, "XYZ point count mismatch");
                        let max_err = pts
                            .iter()
                            .zip(gt_flat.chunks_exact(3))
                            .flat_map(|(p, g)| {
                                [
                                    (p[0] - g[0]).abs(),
                                    (p[1] - g[1]).abs(),
                                    (p[2] - g[2]).abs(),
                                ]
                            })
                            .fold(0.0_f64, f64::max);
                        // LEPCC is lossy; the C++ test just prints the max error.
                        assert!(
                            max_err < 1e-4,
                            "XYZ max error {max_err:.2e} exceeds 1e-4 tolerance"
                        );
                    }
                    "rgb" => {
                        let rgb = ctx.decode_rgb(remaining).expect("decode_rgb failed");
                        let (n_gt, gt_bytes) = read_gt_block(&mut gt_cursor, 3); // 3 × u8
                        assert_eq!(rgb.len(), n_gt as usize, "RGB count mismatch");
                        // Flatten Vec<[u8;3]> to &[u8] for comparison.
                        let rgb_flat: Vec<u8> = rgb.iter().flat_map(|p| *p).collect();
                        assert_eq!(rgb_flat.as_slice(), gt_bytes, "RGB values mismatch");
                    }
                    "intensity" => {
                        let intensity = ctx
                            .decode_intensity(remaining)
                            .expect("decode_intensity failed");
                        let (n_gt, gt_bytes) = read_gt_block(&mut gt_cursor, 2); // u16
                        let gt_vals = bytes_as_u16_le(gt_bytes);
                        assert_eq!(intensity.len(), n_gt as usize, "Intensity count mismatch");
                        assert_eq!(intensity, gt_vals, "Intensity values mismatch");
                    }
                    _ => unreachable!(),
                }

                blob_count += 1;
                pos += blob_size.max(MAGIC_LEN);
                matched = true;
                break;
            }
            if !matched {
                pos += 1;
            }
        }

        assert!(blob_count > 0, "no LEPCC blobs found in the SLPK");
        assert!(
            gt_cursor.is_empty(),
            "{} bytes of ground truth not consumed — blob count mismatch",
            gt_cursor.len()
        );
    }
}
