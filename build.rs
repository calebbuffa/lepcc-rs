fn main() {
    let src = "extern/lepcc/src";

    // C++ sources that make up the lepcc static library.
    // Excludes Test_C_Api.cpp (test program) and any encoder path we don't need.
    let sources = [
        "BitMask.cpp",
        "BitStuffer2.cpp",
        "ClusterRGB.cpp",
        "Common.cpp",
        "FlagBytes.cpp",
        "Huffman.cpp",
        "Intensity.cpp",
        "LEPCC.cpp",
        "lepcc_c_api_impl.cpp",
    ];

    let mut build = cc::Build::new();
    build.cpp(true).include(src);

    // Suppress the LEPCC_EXPORTS define so the header uses plain symbols —
    // we're linking statically, not building a DLL.
    build.define("LEPCC_STATIC", None);

    for file in &sources {
        build.file(format!("{src}/{file}"));
    }

    build.compile("lepcc");

    // Tell cargo to re-run this script if any C++ source or header changes.
    println!("cargo:rerun-if-changed={src}");
}
