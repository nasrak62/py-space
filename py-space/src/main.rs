use py_space_utils::analyzer::analyze_project;

fn main() {
    let result = analyze_project();

    match result {
        Ok(_value) => print!("success"),
        Err(error) => print!("{}", error),
    }
}
