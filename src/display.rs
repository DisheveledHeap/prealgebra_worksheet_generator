use crate::problems::*;
use crate::file_work::*;
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
    copies: usize,
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
            copies: 1,
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
                                self.copies = 1;
                            }
                            if ui.button("Regenerate All Terms").clicked() {
                                for i in 0..self.problems.len() {
                                    if self.problems[i].randomized {self.problems[i].generate();}
                                }
                            }
                            ui.label("Problems Created:");
                            egui::Grid::new("problems_grid")
                                .num_columns(5)
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
                                            let mut temp_problem = self.problems[i].clone();
                                            if temp_problem.randomized {temp_problem.generate();}
                                            self.problems.insert(i, temp_problem);
                                        }

                                        if ui.button("Delete").clicked() {
                                            self.problems.remove(i);
                                        }

                                        if self.problems[i].randomized {
                                            if ui.button("Regenerate Terms").clicked() {
                                                self.problems[i].generate();
                                            }
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
                            save_sheet(&self.tmp_dir, &mut self.error);
                        }

                        if columns[1].button("Print PDF").clicked() {
                            print_sheet(&self.tmp_dir);
                            // let result = {
                            //     #[cfg(target_os = "windows")]
                            //     {
                            //         std::process::Command::new("rundll32")
                            //             .args(["shell32.dll,PrintTo", cur.to_str().unwrap()])
                            //             .spawn()
                            //     }

                            //     #[cfg(target_os = "macos")]
                            //     {
                            //         std::process::Command::new("lp")
                            //             .arg(cur.to_str().unwrap())
                            //             .spawn()
                            //     }

                            //     #[cfg(target_os = "linux")]
                            //     {
                            //         std::process::Command::new("lp")
                            //             .arg(cur.to_str().unwrap())
                            //             .spawn()
                            //     }
                            // };

                            // if let Err(e) = result {
                            //     self.error = Some(format!("Failed to print: {}", e));
                            // }
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
                                            self.temp_problem.terms.push(Term::default());
                                        }
                                    }
                                }
                            });
                        
                        columns[1].group(|ui| {
                            if self.temp_problem.problem_type == ImplementedProblem::MoreToCome {
                                ui.label("Choose a Problem Type from the radio menu");
                            } else {

                                if ui.checkbox(&mut self.temp_problem.randomized, "Randomize Terms").clicked() {
                                    self.randomizing = self.temp_problem.randomized;
                                }
                                if ui.checkbox(&mut self.temp_problem.allow_fractions, "Allow Fractions").clicked() {
                                    self.temp_problem.allow_decimals = false;
                                }

                                if self.temp_problem.randomized {
                                    if self.temp_problem.allow_decimals {
                                        ui.label("Amount of digits after decimal:");
                                        ui.add(egui::DragValue::new(&mut self.temp_problem.digits_after_decimal));
                                        if self.temp_problem.digits_after_decimal == 0 {self.temp_problem.allow_decimals = false;}
                                    } else {
                                        if ui.checkbox(&mut self.temp_problem.allow_decimals, "Allow Decimals").clicked() {
                                            self.temp_problem.allow_fractions = false;
                                            self.temp_problem.digits_after_decimal = 2;
                                        }
                                    }
                                }

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

                                if self.randomizing {
                                    match self.temp_problem.problem_type {
                                        ImplementedProblem::LargeFormatFourOp(BasicOperation::Subtraction)
                                        | ImplementedProblem::MissingOperand(BasicOperation::Subtraction)
                                        | ImplementedProblem::FourOp(BasicOperation::Subtraction) => {ui.checkbox(&mut self.temp_problem.auxillary, "Allow Negative Result");},
                                        ImplementedProblem::ShortDiv | ImplementedProblem::LongDiv => {if !(self.temp_problem.allow_decimals | self.temp_problem.allow_fractions) {ui.checkbox(&mut self.temp_problem.auxillary, "Allow Remainder");}},
                                        ImplementedProblem::DirectPercent => {ui.checkbox(&mut self.temp_problem.auxillary, "For Each 100");},
                                        _ => {}
                                    };
                                    ui.label("Lower Bound");
                                    ui.add(egui::DragValue::new(&mut self.temp_problem.lower_bound));
                                    ui.label("Upper Bound");
                                    ui.add(egui::DragValue::new(&mut self.temp_problem.upper_bound));
                                    ui.label("Copies of this problem");
                                    ui.add(egui::DragValue::new(&mut self.copies).range(1..=30));
                                } else {
                                    for (i, l) in self.temp_problem.problem_type.fields().iter().enumerate() {
                                        ui.label(l);
                                        ui.add(egui::TextEdit::singleline(&mut self.temp_problem.terms[i].whole));
                                        if self.temp_problem.allow_fractions {
                                            ui.label("Numerator");
                                            ui.add(egui::DragValue::new(&mut self.temp_problem.terms[i].numerator));
                                            ui.label("Denominator");
                                            ui.add(egui::DragValue::new(&mut self.temp_problem.terms[i].denominator));
                                        }
                                    }
                                }
                            }
                        });
                    });

                    if ui.button("Update Problem").clicked() {
                        if self.problem_editing == usize::MAX {
                            let mut to_add = vec![self.temp_problem.clone(); self.copies];
                            if self.temp_problem.randomized {
                                for i in 0..to_add.len() {
                                    to_add[i].generate();
                                }
                            }
                            self.problems.append(&mut to_add);
                            self.editing_problem = false;
                        } else {
                            if self.temp_problem.randomized {self.temp_problem.generate();}
                            self.problems[self.problem_editing] = self.temp_problem.clone();
                            self.editing_problem = false;
                        }
                    }
                });
        }
    }
}