use std::path::PathBuf;

fn get_typst_path() -> PathBuf {
    PathBuf::from(env!("TYPST_PATH"))
}

fn main() {
    let typst = get_typst_path();
    let status = std::process::Command::new(&typst)
        .args(["compile","test.typ","output.pdf"])
        .status();
    println!("Doing this from scratch now");
}