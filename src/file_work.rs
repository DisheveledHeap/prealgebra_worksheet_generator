use crate::problems::MathProblem;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use pdfium_render::prelude::*;
use tempfile::TempDir;


fn project_root() -> PathBuf {
    if cfg!(debug_assertions) {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    } else {
        std::env::current_exe()
            .ok()
            .and_then(|exe| exe.parent().map(|p| p.to_path_buf()))
            .unwrap_or_else(|| PathBuf::from("."))
    }
}


/// Returns the path to the Typst binary for the current OS.
/// Looks inside `assets/bin/` relative to the executable.
pub fn get_typst_path() -> PathBuf {
    let base = project_root();
    let bin_dir = base.join("assets").join("bin");

    #[cfg(target_os = "windows")]
    return bin_dir.join("typst.exe");

    #[cfg(not(target_os = "windows"))]
    return bin_dir.join("typst");
}

/// Returns the path to the PDFium library for the current OS.
/// Looks inside `assets/bin/` relative to the executable.
pub fn get_pdfium_path() -> PathBuf {
    let base = project_root();
    let bin_dir = base.join("assets").join("bin");

    #[cfg(target_os = "windows")]
    return bin_dir.join("pdfium.dll");

    #[cfg(target_os = "linux")]
    return bin_dir.join("libpdfium.so");

    #[cfg(target_os = "macos")]
    return bin_dir.join("libpdfium.dylib");
}

pub fn compile_typst_to_pdf(tmp: &Path, problems: &Vec<MathProblem>) -> Result<image::RgbaImage, String> {
    // println!("path is {}",tmp.display());
    fs::create_dir_all(&tmp).map_err(|e| e.to_string())?;

    let typst_path = tmp.join("input.typ");
    let pdf_path = tmp.join("output.pdf");
    let template_path = Path::new("assets/templates/worksheet.typ");
    let template_dst = tmp.join("worksheet.typ");

    // worksheet template needs to be avalible in dir where typst file is compiled to bypass internal typst security
    fs::copy(&template_path, &template_dst)
        .map_err(|e| format!("Failed to copy template {e}"))?;


    // Build the Typst source
    let (left_problems, right_problems):(Vec<String>,Vec<String>) = problems.iter().enumerate()
        .fold((Vec::new(), Vec::new()), |(mut l,mut r),(i,p)| {
            let block = format!("#block[\n{}\n]", p.display());
            if (i % 2) == 0 { l.push(block); }
            else {r.push(block);}
            (l,r)
        });
        
    let (left_typst, right_typst) = (left_problems.join("\n"), right_problems.join("\n"));
        
    // Now wrap each cell in `#grid` in a 2-column layout
    // Typst will automatically fill rows left-to-right
    let content = format!(
        "#import \"worksheet.typ\": *\n\n\
        #set text(size: 25pt)\n\n\
        #stack(dir: ltr, [#box(width: 50%)[\n{}\n]],\n[#box(width: 50%)[\n{}\n]])",
        left_typst, right_typst
    );

    let mut file = File::create(&typst_path).map_err(|e| e.to_string())?;
    file.write_all(content.as_bytes()).map_err(|e| e.to_string())?;

    let typst:PathBuf = get_typst_path();
    println!("{typst:?}");

    let output = Command::new(&typst)
        .args(["compile", typst_path.to_str().unwrap(), pdf_path.to_str().unwrap()])
        .output()
        .map_err(|e| format!("Failed to execute Typst: {}", e))?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }

    // Render the first page of the PDF as an image
    let pdfium_path = get_pdfium_path();
    let pdfium = Pdfium::new(Pdfium::bind_to_library(pdfium_path).map_err(|e| e.to_string())?);
    let doc = pdfium.load_pdf_from_file(&pdf_path, None).map_err(|e| e.to_string())?;
    let page = doc.pages().get(0).map_err(|e| e.to_string())?;
    let image = page.render(1275,1650,None).map_err(|e| e.to_string())?.as_image();

    Ok(image.to_rgba8())
}

pub fn save_sheet(temp_dir:&TempDir, error:&mut Option<String>) {
    if let Some(path) = rfd::FileDialog::new().set_file_name("worksheet.pdf").save_file() {
        let cur = temp_dir.path().join("output.pdf");
        if let Err(e) = std::fs::copy(&cur,&path) {
            *error = Some(format!("Failed to save pdf {}",e));
        }
    }
}

pub fn print_sheet(temp_dir: &TempDir, error:&mut Option<String>) {
    let file = temp_dir.path().join("output.pdf");
    if let Err(e) = print_dialogue(&file) {
        *error = Some(e);
    } else {*error = None;}
}

#[cfg(target_os = "windows")]
pub fn print_dialog(file: &std::path::PathBuf) -> Result<(), String> {
    use std::os::windows::ffi::OsStrExt;
    use windows::Win32::Foundation::HWND;
    use windows::Win32::UI::Shell::ShellExecuteW;
    use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;
    use windows::core::PCWSTR;

    let path: Vec<u16> = file.as_os_str().encode_wide().chain(Some(0)).collect();
    let operation: Vec<u16> = "print".encode_utf16().chain(Some(0)).collect();

    unsafe {
        let result = ShellExecuteW(
            HWND(0),
            PCWSTR(operation.as_ptr()),
            PCWSTR(path.as_ptr()),
            PCWSTR::null(),
            PCWSTR::null(),
            SW_SHOWNORMAL,
        );

        if result.0 as usize <= 32 {
            Err(format!("Failed to open print dialog (error code {:?})", result.0))
        } else {
            Ok(())
        }
    }
}


#[cfg(target_os = "macos")]
fn print_dialogue(file: &PathBuf) -> Result<(), String> {
    Command::new("open")
        .args(["-a", "Preview", file.to_str().ok_or("Invalid path")?])
        .status()
        .map_err(|e| format!("Failed to run open: {}", e))?;
    Ok(())
}

#[cfg(target_os = "linux")]
fn print_dialogue(file: &PathBuf) -> Result<(), String> {
    // Try CUPS lp with dialog
    let status = Command::new("lp")
        .arg(file.to_str().ok_or("Invalid path")?)
        .status();

    match status {
        Ok(s) if s.success() => Ok(()),
        _ => {
            // fallback: open in default PDF viewer
            Command::new("xdg-open")
                .arg(file.to_str().ok_or("Invalid path")?)
                .status()
                .map_err(|e| format!("Failed to run xdg-open: {}", e))?;
            Ok(())
        }
    }
}