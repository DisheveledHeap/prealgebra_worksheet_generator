mod problems;
mod file_work;
mod display;
use display::TypstApp;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Typst Worksheet GUI",
        options,
        Box::new(|_cc| Ok(Box::new(TypstApp::default()))),
    )
}