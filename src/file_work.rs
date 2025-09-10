use crate::problems::MathProblem;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use pdfium_render::prelude::*;

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
    let problems_typst = problems.iter()
        .map(|p| format!("#block[{}]", p.display())).collect::<Vec<_>>().join("\n");
    let content = format!(
        "#import \"worksheet.typ\": *\n\n#set text(size: 15pt)\n\n#columns(2)[\n{}\n]",
        problems_typst
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