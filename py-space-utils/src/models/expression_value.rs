use std::collections::HashSet;

use super::function_def::FunctionDef;

#[derive(Debug, Clone)]
pub struct ExpressionValue {
    pub used_functions: HashSet<String>,
    pub used_names: HashSet<String>,
    pub used_attributes: HashSet<String>,
    pub possible_functions: HashSet<FunctionDef>,
}

impl ExpressionValue {
    pub fn new() -> Self {
        Self {
            used_functions: HashSet::new(),
            used_names: HashSet::new(),
            used_attributes: HashSet::new(),
            possible_functions: HashSet::new(),
        }
    }

    pub fn merge_expression_result(&mut self, expression_value: ExpressionValue) {
        self.used_functions.extend(expression_value.used_functions);
        self.used_names.extend(expression_value.used_names);
        self.used_attributes
            .extend(expression_value.used_attributes);

        self.possible_functions
            .extend(expression_value.possible_functions);
    }

    pub fn insert(&mut self, value: String) -> bool {
        self.used_functions.insert(value)
    }

    pub fn insert_name(&mut self, value: String) -> bool {
        self.used_names.insert(value)
    }

    pub fn insert_attribute(&mut self, value: String) -> bool {
        self.used_attributes.insert(value)
    }

    pub fn insert_possible_function(&mut self, value: FunctionDef) -> bool {
        self.possible_functions.insert(value)
    }
}
