use std::{env, path::PathBuf, process::Command};
fn main() {
    println!("cargo:rerun-if-changed=src");
    println!("cargo:rerun-if-changed=tools/prepare_font.py");
    println!("cargo:rerun-if-changed=.build-tools/NotoSansSC.ttf");
    println!("cargo:rerun-if-changed=.build-tools/python-path.txt");
    // The subset also depends on the installed fonttools version.
    println!("cargo:rerun-if-changed=.build-tools/site");
    let root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let cache = root.join(".build-tools");
    let python = std::fs::read_to_string(cache.join("python-path.txt"))
        .expect("Run python tools/build.py once to initialize the font toolchain");
    let result = Command::new(python.trim())
        .env("PYTHONPATH", cache.join("site"))
        .arg(root.join("tools/prepare_font.py"))
        .arg(cache.join("NotoSansSC.ttf"))
        .arg(PathBuf::from(env::var("OUT_DIR").unwrap()).join("ui-subset.ttf"))
        .status()
        .expect("Cannot execute font subset builder; run tools/build.py to repair");
    assert!(
        result.success(),
        "Font subsetting failed; check missing glyphs and font toolchain"
    );
}
