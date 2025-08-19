use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    // Typst binary
    #[cfg(target_os = "windows")]
    let typst_src = PathBuf::from("assets/bin/typst.exe");
    #[cfg(not(target_os = "windows"))]
    let typst_src = PathBuf::from("assets/bin/typst");

    let typst_dst = out_dir.join(typst_src.file_name().unwrap());
    fs::copy(&typst_src, &typst_dst).expect("Failed to copy Typst binary");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&typst_dst).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&typst_dst, perms).unwrap();
    }

    println!("cargo:rustc-env=TYPST_PATH={}", typst_dst.display());

    // PDFium binary
    #[cfg(target_os = "windows")]
    let pdfium_src = PathBuf::from("assets/bin/pdfium.dll");
    #[cfg(target_os = "linux")]
    let pdfium_src = PathBuf::from("assets/bin/libpdfium.so");
    #[cfg(target_os = "macos")]
    let pdfium_src = PathBuf::from("assets/bin/libpdfium.dylib");

    let pdfium_dst = out_dir.join(pdfium_src.file_name().unwrap());
    fs::copy(&pdfium_src, &pdfium_dst).expect("Failed to copy PDFium binary");

    println!("cargo:rustc-env=PDFIUM_PATH={}", pdfium_dst.display());
}
