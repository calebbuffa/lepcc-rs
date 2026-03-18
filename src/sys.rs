//! Raw unsafe FFI declarations for the LEPCC C API.
//!
//! Every symbol is declared `extern "C"` exactly as in `lepcc_c_api.h`.
//! Do not call these directly — use the safe wrappers in [`crate`].

#![allow(non_camel_case_types, dead_code)]

use std::ffi::c_int;
use std::ffi::c_uint;

/// Opaque context handle returned by [`lepcc_createContext`].
pub type lepcc_ContextHdl = *mut std::ffi::c_void;

/// Status code returned by most LEPCC functions.
/// `0` means success; any other value is an error.
pub type lepcc_status = c_uint;

/// Identifies which codec produced a blob (XYZ, RGB, Intensity, FlagBytes).
pub type lepcc_blobType = c_uint;

unsafe extern "C" {
    pub fn lepcc_createContext() -> lepcc_ContextHdl;
    pub fn lepcc_deleteContext(ctx: *mut lepcc_ContextHdl);

    /// Returns the number of bytes needed to call `lepcc_getBlobInfo`.
    pub fn lepcc_getBlobInfoSize() -> c_int;

    pub fn lepcc_getBlobInfo(
        ctx: lepcc_ContextHdl,
        packed: *const u8,
        buffer_size: c_int,
        blob_type: *mut lepcc_blobType,
        blob_size: *mut c_uint,
    ) -> lepcc_status;

    pub fn lepcc_getPointCount(
        ctx: lepcc_ContextHdl,
        packed: *const u8,
        buffer_size: c_int,
        count_out: *mut c_uint,
    ) -> lepcc_status;

    pub fn lepcc_getRGBCount(
        ctx: lepcc_ContextHdl,
        packed: *const u8,
        buffer_size: c_int,
        count_out: *mut c_uint,
    ) -> lepcc_status;

    pub fn lepcc_getIntensityCount(
        ctx: lepcc_ContextHdl,
        packed: *const u8,
        buffer_size: c_int,
        count_out: *mut c_uint,
    ) -> lepcc_status;

    pub fn lepcc_getFlagByteCount(
        ctx: lepcc_ContextHdl,
        packed: *const u8,
        buffer_size: c_int,
        count_out: *mut c_uint,
    ) -> lepcc_status;

    /// Decode XYZ coordinates.  `xyz_buf_out` must hold `n_pts * 3` doubles.
    pub fn lepcc_decodeXYZ(
        ctx: lepcc_ContextHdl,
        pp_byte: *mut *const u8,
        buffer_size: c_int,
        n_pts_in_out: *mut c_uint,
        xyz_buf_out: *mut f64,
    ) -> lepcc_status;

    /// Decode RGB colours.  `rgb_buf_out` must hold `n_rgb * 3` bytes.
    pub fn lepcc_decodeRGB(
        ctx: lepcc_ContextHdl,
        pp_byte: *mut *const u8,
        buffer_size: c_int,
        n_rgb_in_out: *mut c_uint,
        rgb_buf_out: *mut u8,
    ) -> lepcc_status;

    /// Decode intensity values (u16).  `intensity_buf_out` must hold `n_values` shorts.
    pub fn lepcc_decodeIntensity(
        ctx: lepcc_ContextHdl,
        pp_byte: *mut *const u8,
        buffer_size: c_int,
        n_values: *mut c_uint,
        intensity_buf_out: *mut u16,
    ) -> lepcc_status;

    /// Decode flag bytes.  `flag_bytes_buf_out` must hold `n_values` bytes.
    pub fn lepcc_decodeFlagBytes(
        ctx: lepcc_ContextHdl,
        pp_byte: *mut *const u8,
        buffer_size: c_int,
        n_values: *mut c_uint,
        flag_bytes_buf_out: *mut u8,
    ) -> lepcc_status;

    pub fn lepcc_computeCompressedSizeXYZ(
        ctx: lepcc_ContextHdl,
        n_pts: c_uint,
        xyz_array: *const f64,
        max_x_err: f64,
        max_y_err: f64,
        max_z_err: f64,
        n_bytes_out: *mut c_uint,
        order_out: *mut c_uint,
    ) -> lepcc_status;

    pub fn lepcc_computeCompressedSizeRGB(
        ctx: lepcc_ContextHdl,
        n_rgb: c_uint,
        rgb_array: *const u8,
        n_bytes_out: *mut c_uint,
    ) -> lepcc_status;

    pub fn lepcc_computeCompressedSizeIntensity(
        ctx: lepcc_ContextHdl,
        n_values: c_uint,
        val_array: *const u16,
        n_bytes_out: *mut c_uint,
    ) -> lepcc_status;

    pub fn lepcc_computeCompressedSizeFlagBytes(
        ctx: lepcc_ContextHdl,
        n_values: c_uint,
        val_array: *const u8,
        n_bytes_out: *mut c_uint,
    ) -> lepcc_status;

    pub fn lepcc_encodeXYZ(
        ctx: lepcc_ContextHdl,
        pp_byte_out: *mut *mut u8,
        buffer_size_out: c_int,
    ) -> lepcc_status;

    pub fn lepcc_encodeRGB(
        ctx: lepcc_ContextHdl,
        pp_byte_out: *mut *mut u8,
        buffer_size_out: c_int,
    ) -> lepcc_status;

    pub fn lepcc_encodeIntensity(
        ctx: lepcc_ContextHdl,
        pp_byte_out: *mut *mut u8,
        buffer_size_out: c_int,
        intensity_array: *const u16,
        n_values: c_uint,
    ) -> lepcc_status;

    pub fn lepcc_encodeFlagBytes(
        ctx: lepcc_ContextHdl,
        pp_byte_out: *mut *mut u8,
        buffer_size_out: c_int,
        flag_byte_array: *const u8,
        n_values: c_uint,
    ) -> lepcc_status;
}
