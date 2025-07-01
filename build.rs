use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    // Where Cargo will store built artifacts
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    // Path to your pre-bundled Typst binary (in assets/bin/)
    #[cfg(target_os = "windows")]
    let typst_src = PathBuf::from("assets/bin/typst.exe");
    #[cfg(not(target_os = "windows"))]
    let typst_src = PathBuf::from("assets/bin/typst");

    // Destination path inside the build output directory
    let typst_dst = out_dir.join(typst_src.file_name().unwrap());

    // Copy the Typst binary
    fs::copy(&typst_src, &typst_dst)
        .expect("Failed to copy Typst binary to OUT_DIR");

    // Make executable (Unix only)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&typst_dst).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&typst_dst, perms).unwrap();
    }

    // Make this path available to your main program
    println!("cargo:rustc-env=TYPST_PATH={}", typst_dst.display());
}