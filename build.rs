#[cfg(windows)]
fn main() {
    use std::{env, path::PathBuf, process::Command};

    let manifest_dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let resource_file = manifest_dir.join("assets").join("windows-resource.rc");
    let resource_object = out_dir.join("windows-resource.o");
    let windres = env::var_os("WINDRES").unwrap_or_else(|| "windres".into());

    let status = Command::new(&windres)
        .arg("--input")
        .arg(&resource_file)
        .arg("--output")
        .arg(&resource_object)
        .arg("--output-format=coff")
        .status()
        .unwrap_or_else(|error| panic!("failed to run {:?}: {error}", windres));

    assert!(status.success(), "windres failed with status {status}");
    println!("cargo:rustc-link-arg={}", resource_object.display());
    println!("cargo:rerun-if-changed={}", resource_file.display());
    println!(
        "cargo:rerun-if-changed={}",
        manifest_dir.join("assets").join("app-icon.ico").display()
    );
}

#[cfg(not(windows))]
fn main() {}
