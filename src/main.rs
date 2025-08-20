use eframe::egui;
use egui::{vec2, Align2, Image};
use core::fmt;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use pdfium_render::prelude::*;
use tempfile::TempDir;
use std::vec::Vec;

#[derive(Clone, Copy)]
enum BasicOperation {
    Undecided,
    Addition,
    Subtraction,
    Multiplication,
}

impl fmt::Display for BasicOperation {
    fn fmt(&self, f:&mut fmt::Formatter) -> fmt::Result {
        match *self {
            BasicOperation::Addition => write!(f, "+"),
            BasicOperation::Subtraction => write!(f, "-"),
            BasicOperation::Multiplication => write!(f,"*"),
            BasicOperation::Undecided => write!(f,"◦"),
        }
    }
}

#[derive(Clone, Copy)]
enum ImplementedProblem {
    LargeFormatFourOp(BasicOperation),
    MissingOperand(BasicOperation),
    FourOp(BasicOperation),
    LongDiv,
    ShortDiv,
    Proportions,
    MoreToCome,
}

impl fmt::Display for ImplementedProblem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ImplementedProblem::LargeFormatFourOp(o) => write!(f,"Large Format Basic Arithmetic ({})", o),
            ImplementedProblem::MissingOperand(o) => write!(f,"Missing Operand ({})", o),
            ImplementedProblem::FourOp(o) => write!(f,"Basic Arithmetic ({})", o),
            ImplementedProblem::LongDiv => write!(f,"Long Division"),
            ImplementedProblem::ShortDiv => write!(f,"Horizontal Division"),
            ImplementedProblem::Proportions => write!(f,"Proportions"),
            _ => write!(f,"Unimplemented"),
        }
    }
}

impl ImplementedProblem {
    pub fn iterator() -> std::slice::Iter<'static, ImplementedProblem> {
        static IMPLEMENTED_PROBLEMS:[ImplementedProblem; 6] = [ImplementedProblem::LargeFormatFourOp(BasicOperation::Undecided),ImplementedProblem::MissingOperand(BasicOperation::Undecided),ImplementedProblem::FourOp(BasicOperation::Undecided),ImplementedProblem::LongDiv,ImplementedProblem::ShortDiv,ImplementedProblem::Proportions];
        IMPLEMENTED_PROBLEMS.iter()
    }
    fn required_operands(&self) -> usize {
        match *self {
            ImplementedProblem::LargeFormatFourOp(_) => 2,
            ImplementedProblem::MissingOperand(_) => 2,
            ImplementedProblem::FourOp(_) => 2,
            ImplementedProblem::LongDiv => 2,
            ImplementedProblem::ShortDiv => 2,
            ImplementedProblem::Proportions => 3,
            _ => 0,
        }
    }
}

#[derive(Clone)]
pub struct MathProblem {
    problem_type: ImplementedProblem,
    numbers: Vec<i32>,
}

impl MathProblem {
    fn new() -> MathProblem {
        MathProblem { problem_type: ImplementedProblem::MoreToCome, numbers: vec![1,2] }
    }

    fn display(&self) -> String {
        if self.problem_type.required_operands() > self.numbers.len() {return String::from("Insufficient Operands\n\n");}
        match self.problem_type {
            ImplementedProblem::FourOp(o) => format!("{} {} {} = \\_\\_\\_\n\n", self.numbers[0], o, self.numbers[1]),
            _ => format!("Problem of type {} with parameters {:?}\n\n", self.problem_type, self.numbers)
        }
    }
}

pub struct TypstApp {
    input: String,
    preview: Option<egui::TextureHandle>,
    error: Option<String>,
    tmp_dir: TempDir,
    problems: Vec<MathProblem>,

    problem_editing: usize,
    editing_problem: bool,
    temp_type_input: String,
    temp_nums_input: String,
    temp_problem: MathProblem,
}

impl Default for TypstApp {
    fn default() -> Self {
        Self {
            input: String::new(),
            preview: None,
            error: None,
            tmp_dir: TempDir::new().expect("failed to create temporary directory"),
            problems: Vec::new(),
            problem_editing: 0,
            editing_problem: false,
            temp_type_input: String::new(),
            temp_nums_input: String::new(),
            temp_problem: MathProblem::new(),
        }
    }
}

impl eframe::App for TypstApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("Typst Worksheet GUI");
            });

            ui.add_space(10.0);

            ui.columns(2, |columns| {
                // Left: Typst input
                egui::ScrollArea::vertical()
                    .show(&mut columns[0], |ui| {
                        ui.group(|ui| {
                            ui.label("temporary typst input for testing");
                            ui.add(
                                egui::TextEdit::multiline(&mut self.input)
                                    .font(egui::TextStyle::Monospace)
                                    .desired_rows(15),
                            );
                            ui.add_space(4.0);
                            
                            if ui.button("Compile").clicked() {
                                match compile_typst_to_pdf(self.tmp_dir.path(), &self.problems) {
                                    Ok(image) => {
                                        self.preview = Some(ctx.load_texture(
                                            "preview",
                                            egui::ColorImage::from_rgba_unmultiplied(
                                                [image.width() as usize, image.height() as usize],
                                                image.as_raw(),
                                            ),
                                            Default::default(),
                                        ));
                                        self.error = None;
                                    }
                                    Err(e) => {
                                        self.preview = None;
                                        self.error = Some(e);
                                    }
                                }
                            }
                            
                        });

                        ui.add_space(4.0);

                        ui.group(|ui| {
                            ui.label("Problems Input (just needs to add and edit existing problems)");
                            if ui.button("Add new Problem").clicked() {
                                println!("detected press");
                                self.problem_editing = self.problems.len();
                                self.problems.push(MathProblem::new());
                                self.editing_problem = true;
                                self.temp_type_input = String::new();
                                self.temp_nums_input = String::new();
                                self.temp_problem = MathProblem::new();
                            }
                            for i in 0..self.problems.len() { // need two more buttons, one to duplicate a problem, and one to delete a problem, should be in-line with the first button
                                if ui.button(format!("Edit Problem {}", i)).clicked() { // note - maybe make problem descriptor more descriptive so that after deleting/copying problems, user can still tell which is which
                                    self.problem_editing = i;
                                    self.editing_problem = true;
                                    self.temp_problem = self.problems[i].clone();
                                    self.temp_nums_input = self.problems[i].numbers.iter().fold(String::new(), |mut acc, x| {
                                        acc.push_str(&format!(" {}", x));
                                        acc
                                    });
                                }
                            }
                        });
                    });

                // Right: PDF preview
                columns[1].group(|ui| {
                    ui.label("PDF Preview");
                    ui.columns(2, |columns| {
                        if columns[0].button("Download PDF").clicked() {
                            if let Some(path) = rfd::FileDialog::new().set_file_name("worksheet.pdf").save_file() {
                                let cur = self.tmp_dir.path().join("output.pdf");
                                if let Err(e) = std::fs::copy(&cur,&path) {
                                    self.error = Some(format!("Failed to save pdf {}",e));
                                }
                            }
                        }

                        if columns[1].button("Print PDF").clicked() {
                            let cur = self.tmp_dir.path().join("output.pdf");

                            let result = {
                                #[cfg(target_os = "windows")]
                                {
                                    std::process::Command::new("rundll32")
                                        .args(["shell32.dll,PrintTo", cur.to_str().unwrap()])
                                        .spawn()
                                }

                                #[cfg(target_os = "macos")]
                                {
                                    std::process::Command::new("open")
                                        .arg(cur.to_str().unwrap())
                                        .spawn()
                                }

                                #[cfg(target_os = "linux")]
                                {
                                    std::process::Command::new("lp")
                                        .arg(cur.to_str().unwrap())
                                        .spawn()
                                }
                            };

                            if let Err(e) = result {
                                self.error = Some(format!("Failed to print: {}", e));
                            }
                        }
                    });
                    if let Some(texture) = &self.preview {
                        let available_size = ui.available_size();
                        let original_size = texture.size_vec2();

                        let scale = (available_size.x / original_size.x).min(available_size.y / original_size.y).min(1.0);
                        let scaled_size = original_size * scale;

                        ui.add(egui::Image::new(texture).max_size(available_size));
                    } else if let Some(error) = &self.error {
                        ui.colored_label(egui::Color32::RED, error);
                    }
                });
            });
        });

        if self.editing_problem {
            egui::Window::new(format!("Editing Problem {}",self.problem_editing)) // turn this into another 2-column window, left column should just be a radio menu of different problem types, right column should be a responsive menu with a button randomize numbers, and a series of number input boxes based on how many numbers are needed for each problem type
                .collapsible(false) // potential future functionality, have a real-time display of how the problem will look on-page?
                .resizable(false)
                .anchor(Align2::CENTER_CENTER,vec2(0.0, 0.0))
                .show(ctx, |ui| {
                    ui.label(format!("Editing Problem {}", self.problem_editing));

                    ui.columns(2, |columns| {
                        egui::ScrollArea::vertical()
                            .show(&mut columns[0], |ui| {
                                ui.label("List of incorporated problem types below here (time to see how radio buttons work");
                                for pt in ImplementedProblem::iterator() {
                                    if ui.button(format!("{}", pt)).clicked() {
                                        self.temp_problem.problem_type = *pt;
                                    }
                                }
                            });
                        
                        columns[1].group(|ui| {
                            ui.label(format!("This menu needs to respond to the active problem type {}", self.temp_problem.problem_type));
                        });
                    });

                    if ui.button("Update Problem").clicked() {
                        self.problems[self.problem_editing] = self.temp_problem.clone();
                        self.editing_problem = false;
                    }
                });
        }
    }
}

fn compile_typst_to_pdf(tmp: &Path, problems: &Vec<MathProblem>) -> Result<image::RgbaImage, String> {
    println!("path is {}",tmp.display());
    fs::create_dir_all(&tmp).map_err(|e| e.to_string())?;

    let typst_path = tmp.join("input.typ");
    let pdf_path = tmp.join("output.pdf");
    let template_path = Path::new("assets/templates/worksheet.typ");
    let template_dst = tmp.join("worksheet.typ");

    // worksheet template needs to be avalible in dir where typst file is compiled to bypass internal typst security
    fs::copy(&template_path, &template_dst)
        .map_err(|e| format!("Failed to copy template {e}"))?;


    // Build the Typst source
    let content = format!(
        "#import \"worksheet.typ\": *\n\n{}",
        problems.iter().map(|p| p.display()).collect::<Vec<String>>().iter().flat_map(|s| s.chars()).collect::<String>()
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

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Typst Worksheet GUI",
        options,
        Box::new(|_cc| Ok(Box::new(TypstApp::default()))),
    )
}