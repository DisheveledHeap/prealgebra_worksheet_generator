mod problems;
mod file_work;
mod display;
use display::TypstApp;
use file_work::check_dependencies;

fn main() -> Result<(), eframe::Error> {
    check_dependencies();
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Typst Worksheet GUI",
        options,
        Box::new(|_cc| Ok(Box::new(TypstApp::default()))),
    )
}