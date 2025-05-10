use std::collections::HashSet;

use crate::models::statement_value::StatementValue;

const INIT_FUNCTIONS: [&str; 2] = ["__init__", "__new__"];

pub fn fix_assignments(mut statement_value: StatementValue) -> StatementValue {
    let mut used_functions = HashSet::new();
    let old_used_functions = statement_value.expression_value.used_functions.clone();

    for function in old_used_functions {
        if !function.contains(".") {
            used_functions.insert(function);
            continue;
        }

        let mut parts = function.split(".");

        if parts.clone().count() != 2 {
            dbg!(&parts);

            used_functions.insert(function);
            continue;
        }

        let mut class_name = parts.next().unwrap().to_string();
        let function_name = parts.next().unwrap();

        if !statement_value.assignments.contains_key(&class_name) {
            used_functions.insert(format!("{}.{}", class_name, function_name));

            continue;
        }

        class_name = statement_value
            .assignments
            .get(&class_name)
            .unwrap()
            .to_string();

        for init_function in INIT_FUNCTIONS {
            let name = format!("{}.{}", class_name, init_function);

            if !used_functions.contains(&name) {
                used_functions.insert(name);
            }
        }

        used_functions.insert(format!("{}.{}", class_name, function_name));
    }

    let functions_name = statement_value.build_full_name_functions();

    for attribute in &statement_value.expression_value.used_attributes {
        let attribute_is_function = functions_name.contains(attribute);
        let attribute_seen_already = used_functions.contains(attribute);

        if attribute_is_function && !attribute_seen_already {
            used_functions.insert(attribute.to_string());
        }
    }

    statement_value.expression_value.used_functions = used_functions;

    statement_value
}
