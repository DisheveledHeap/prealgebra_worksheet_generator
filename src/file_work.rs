use crate::problems::MathProblem;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use pdfium_render::prelude::*;
use tempfile::TempDir;

pub fn check_dependencies() {
    //check if the build_file did anything
    let typst_path = env!("TYPST_PATH");
    let pdfium_path = env!("PDFIUM_PATH");

    println!("Found:\n{}\n{}", typst_path, pdfium_path);
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

    let typst = PathBuf::from(env!("TYPST_PATH"));

    let output = Command::new(&typst)
        .args(["compile", typst_path.to_str().unwrap(), pdf_path.to_str().unwrap()])
        .output()
        .map_err(|e| format!("Failed to execute Typst: {}", e))?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }

    // Render the first page of the PDF as an image
    let pdfium_path = std::env::var("PDFIUM_PATH").map_err(|e| e.to_string())?;
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

pub fn print_sheet(temp_dir:&TempDir) {
    if let Some(path) = rfd::FileDialog::new().set_file_name("worksheet.pdf").save_file() {
        let cur = temp_dir.path().join("output.pdf");
        println!("{:?}", temp_dir);
    }
}