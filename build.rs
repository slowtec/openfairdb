use std::{env, process::Command};

const CLEARANCE_NAME: &str = "clearance";
const CLEARANCE_PKG_DIR: &str = "ofdb-app-clearance";
const CLEARANCE_PKG_SRC: &str = "ofdb-app-clearance/src";
const CLEARANCE_FEATURE_NAME: &str = "clearance";

const APP_NAME: &str = "ofdb_app";
const APP_PKG_DIR: &str = "ofdb-app";
const APP_PKG_SRC: &str = "ofdb-app/src";
const APP_FEATURE_NAME: &str = "app";

fn main() {
    let build_clearance = env::var(format!(
        "CARGO_FEATURE_{}",
        CLEARANCE_FEATURE_NAME.to_uppercase()
    ))
    .is_ok();
    let build_app = env::var(format!("CARGO_FEATURE_{}", APP_FEATURE_NAME.to_uppercase())).is_ok();

    if build_clearance || build_app {
        assert_wasm_pack_is_installed();
    }

    let apps = &[
        (
            build_clearance,
            CLEARANCE_NAME,
            CLEARANCE_PKG_DIR,
            CLEARANCE_PKG_SRC,
        ),
        (build_app, APP_NAME, APP_PKG_DIR, APP_PKG_SRC),
    ];

    for (build, name, dir, src) in apps {
        if *build {
            Command::new("wasm-pack")
                .args(&[
                    "build",
                    "--target",
                    "web",
                    "--release",
                    "--out-name",
                    name,
                    dir,
                ])
                .status()
                .expect("Unable to successfully execute wasm-pack");
            println!("cargo:rerun-if-changed=\"{}\"", src);
            for entry in walkdir::WalkDir::new(src)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                println!("cargo:rerun-if-changed=\"{}\"", entry.path().display());
            }
        }
    }
}

fn assert_wasm_pack_is_installed() {
    let output = Command::new("cargo")
        .args(&["install", "--list"])
        .output()
        .expect("Unable to check wasm-pack installation");
    let output_string = String::from_utf8(output.stdout).unwrap();
    if !output_string.contains("wasm-pack") {
        Command::new("cargo")
            .args(&["install", "wasm-pack"])
            .status()
            .expect("Unable install wasm-pack");
    }
}
