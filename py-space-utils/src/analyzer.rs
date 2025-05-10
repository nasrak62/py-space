use std::collections::HashSet;

use crate::{
    assignments::fix_assignments, errors::py_space::PySpaceError, file_utils::get_files_iterator,
    models::statement_value::StatementValue, parse_statement::extract_file_data,
    possible_functions::handle_possible_functions,
};

pub fn analyze_project() -> Result<bool, PySpaceError> {
    let mut statement_value = StatementValue::new();
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
        let path_str = path.to_str().map_or("", |value| value);
        let is_venv = path_str.contains("venv");
        let is_python_file = path
            .extension()
            .map_or(false, |extension| extension == "py");

        if !is_python_file || is_venv {
            continue;
        }

        let new_statement_value = match extract_file_data(path.to_path_buf()) {
            Ok(value) => value,
            Err(error) => {
                dbg!("error getting file data: {}", error);

                continue;
            }
        };

        statement_value.merge_statement_value(new_statement_value);
    }

    let statement_value = fix_assignments(statement_value);
    let statement_value = handle_possible_functions(statement_value);

    for function in &statement_value.functions {
        let name = function.full_name();
        let path = function.file.to_str().map_or("", |value| value);

        if !statement_value
            .expression_value
            .used_functions
            .contains(&name)
        {
            unused_function.insert((name, path));
        }
    }

    let mut result_string = String::from("");

    for value in unused_function {
        result_string += &format!("{}: {}\n", value.0, value.1);
    }

    print!("{}", result_string);
    dbg!(statement_value.expression_value.possible_functions);

    Ok(true)
}
