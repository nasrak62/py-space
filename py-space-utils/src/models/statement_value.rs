use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

use super::{expression_value::ExpressionValue, file_data::FileData, function_def::FunctionDef};

pub struct StatementValue {
    pub functions: HashSet<FunctionDef>,
    pub expression_value: ExpressionValue,
    pub classes: HashSet<String>,
    pub assignments: HashMap<String, String>,
    pub files: HashMap<PathBuf, FileData>,
}

impl StatementValue {
    pub fn new() -> Self {
        Self {
            functions: HashSet::new(),
            expression_value: ExpressionValue::new(),
            classes: HashSet::new(),
            assignments: HashMap::new(),
            files: HashMap::new(),
        }
    }

    pub fn merge_expression_value(&mut self, expression_value: ExpressionValue) {
        self.expression_value
            .merge_expression_result(expression_value);
    }

    pub fn insert_function(&mut self, value: FunctionDef) -> bool {
        self.functions.insert(value)
    }

    pub fn insert_class(&mut self, value: String) -> bool {
        self.classes.insert(value)
    }

    pub fn merge_statement_value(&mut self, statement_value: StatementValue) {
        self.functions.extend(statement_value.functions);

        self.expression_value
            .merge_expression_result(statement_value.expression_value);

        self.classes.extend(statement_value.classes);
        self.assignments.extend(statement_value.assignments);
        self.files.extend(statement_value.files);
    }

    pub fn build_full_name_functions(&self) -> HashSet<String> {
        let mut functions = HashSet::new();

        for function in self.functions.clone() {
            let name = function.full_name();

            functions.insert(name);
        }

        functions
    }
}
