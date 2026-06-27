fn main() {
    let out_dir = std::path::PathBuf::from(
        std::env::var("CARGO_MANIFEST_DIR").unwrap(),
    )
    .parent()
    .unwrap()
    .parent()
    .unwrap()
    .join("Moonshine")
    .join("Bridge")
    .join("Generated");

    std::fs::create_dir_all(&out_dir).unwrap();

    let bridges = swift_bridge_build::parse_bridges(&[
        std::path::PathBuf::from("src/lib.rs"),
    ]);

    bridges.write_all_concatenated(&out_dir, "libmoonshine_ffi");
}
