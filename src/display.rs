use crate::problems::*;
use crate::file_work::compile_typst_to_pdf;
use eframe::egui;
use egui::{vec2, Align2};
use std::usize;
use tempfile::TempDir;
use std::vec::Vec;

pub struct TypstApp {
    preview: Option<egui::TextureHandle>,
    error: Option<String>,
    tmp_dir: TempDir,
    problems: Vec<MathProblem>,
    randomizing: bool,

    problem_editing: usize,
    editing_problem: bool,
    temp_problem: MathProblem,
    copies: String,
}

impl Default for TypstApp {
    fn default() -> Self {
        Self {
            preview: None,
            error: None,
            tmp_dir: TempDir::new().expect("failed to create temporary directory"),
            problems: Vec::new(),
            randomizing: true,
            problem_editing: 0,
            editing_problem: false,
            temp_problem: MathProblem::new(),
            copies: String::new(),
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
                           

                        ui.add_space(4.0);

                        ui.group(|ui| {
                            ui.label("Problems Input (just needs to add and edit existing problems)");
                            if ui.button("Add new Problem").clicked() {
                                self.problem_editing = usize::MAX;
                                self.editing_problem = true;
                                self.temp_problem = MathProblem::new();
                                self.temp_problem.randomized = self.randomizing;
                                self.copies = String::from("1");
                            }
                            ui.label("Problems Created:");
                            egui::Grid::new("problems_grid")
                                .num_columns(4)
                                .striped(true)
                                .show(ui, |ui| {
                                    for i in 0..self.problems.len() {
                                        ui.label(format!("Problem {i}"));
                                        
                                        if ui.button("Edit").clicked() {
                                            self.problem_editing = i;
                                            self.editing_problem = true;
                                            self.temp_problem = self.problems[i].clone();
                                        }

                                        if ui.button("Dup").clicked() {
                                            let temp_problem = self.problems[i].clone();
                                            self.problems.insert(i, temp_problem);
                                        }

                                        if ui.button("Delete").clicked() {
                                            self.problems.remove(i);
                                        }

                                        ui.end_row();
                                    }
                                });
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
                                    std::process::Command::new("lp")
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

                        ui.add(egui::Image::new(texture).max_size(available_size));
                    } else if let Some(error) = &self.error {
                        ui.colored_label(egui::Color32::RED, error);
                    }
                });
            });
        });

        if self.editing_problem {
            egui::Window::new(format!("Editing Problem {}",self.problem_editing)) // turn this into another 2-column window, left column should just be a radio menu of different problem types, right column should be a responsive menu with a button randomize terms, and a series of input boxes based on how many terms are needed for each problem type
                .collapsible(false) // potential future functionality, have a real-time display of how the problem will look on-page?
                .resizable(false)
                .anchor(Align2::CENTER_CENTER,vec2(0.0, 0.0))
                .show(ctx, |ui| {
                    ui.columns(2, |columns| {
                        egui::ScrollArea::vertical()
                            .show(&mut columns[0], |ui| {
                                ui.label("Problem Types");
                                for &pt in ImplementedProblem::iterator() {
                                    if ui.radio_value(&mut self.temp_problem.problem_type, pt, format!("{pt}")).clicked() {
                                        while self.temp_problem.terms.len() < pt.required_operands() {
                                            self.temp_problem.terms.push(String::new());
                                        }
                                    }
                                }
                            });
                        
                        columns[1].group(|ui| {
                            if self.temp_problem.problem_type == ImplementedProblem::MoreToCome {
                                ui.label("Choose a Problem Type from the radio menu");
                            }

                            if ui.checkbox(&mut self.temp_problem.randomized, "Randomize Terms").clicked() {
                                self.randomizing = self.temp_problem.randomized;
                            }

                            if self.randomizing {

                                if self.temp_problem.problem_type.requires_operation() {
                                    ui.label("Choose Operation");

                                    let mut current_op = match self.temp_problem.problem_type {
                                        ImplementedProblem::LargeFormatFourOp(o)
                                        | ImplementedProblem::MissingOperand(o)
                                        | ImplementedProblem::FourOp(o) => o,
                                        _ => BasicOperation::Undecided,
                                    };

                                    for &op in BasicOperation::iterator() {
                                        if ui.radio_value(&mut current_op, op, format!("{op}")).clicked() {
                                            self.temp_problem.problem_type.set_operation(current_op);
                                        }
                                    }
                                }

                                for (i, l) in self.temp_problem.problem_type.fields().iter().enumerate() {
                                    ui.label(l);
                                    ui.add(egui::TextEdit::singleline(&mut self.temp_problem.terms[i]));
                                    //try to implement some form of input sanitization here
                                }
                            } else {
                                
                            }


                        });
                    });

                    if ui.button("Update Problem").clicked() {
                        if self.problem_editing == usize::MAX {
                            self.problems.append(&mut vec![self.temp_problem.clone(); self.copies.parse::<usize>().unwrap_or(1)]);
                            self.editing_problem = false;
                        } else {
                            self.problems[self.problem_editing] = self.temp_problem.clone();
                            self.editing_problem = false;
                        }
                    }
                });
        }
    }
}