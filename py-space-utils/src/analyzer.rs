use std::collections::HashSet;

use crate::{
    errors::py_space::PySpaceError, file_utils::get_files_iterator, parser::extract_file_data,
};

pub fn analyze_project() -> Result<bool, PySpaceError> {
    let mut functions = HashSet::new();
    let mut used_functions = HashSet::new();
    let mut unused_function = HashSet::new();

    let walker = get_files_iterator()?;

    for file_result in walker {
        let file = match file_result {
            Ok(value) => value,
            Err(error) => {
                dbg!("error parsing file: {}", error);

                continue;
            }
        };

        let path = file.path();
        let is_python_file = path
            .extension()
            .map_or(false, |extension| extension == "py");

        if !is_python_file {
            continue;
        }

        let file_data = match extract_file_data(path.to_path_buf()) {
            Ok(value) => value,
            Err(error) => {
                dbg!("error getting file data: {}", error);

                continue;
            }
        };

        let (new_functions, new_used_functions) = file_data;

        functions.extend(new_functions);
        used_functions.extend(new_used_functions);
    }

    dbg!(&functions);

    for function in &functions {
        let name = function.full_name();
        let path = function.file.to_str().map_or("", |value| value);

        if !used_functions.contains(&name) {
            unused_function.insert((name, path));
        }
    }

    dbg!(&used_functions, &unused_function);

    let mut result_string = String::from("");

    for value in unused_function {
        result_string += &format!("{}: {}\n", value.0, value.1);
    }

    print!("{}", result_string);

    Ok(true)
}
