use rustpython_parser::ast;
use rustpython_parser::{parse, Mode};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

use crate::errors::py_space::PySpaceError;
use crate::models::file_data::FileData;
use crate::models::function_def::FunctionDef;
use crate::models::imports::Imports;
use crate::models::statement_value::StatementValue;
use crate::parse_expression::handle_expression;

enum FunctionDefOptions {
    StmtFunctionDef(ast::StmtFunctionDef),
    StmtAsyncFunctionDef(ast::StmtAsyncFunctionDef),
}

fn handle_function_def(
    data: FunctionDefOptions,
    path: &PathBuf,
    class_name: Option<String>,
) -> StatementValue {
    let mut statement_value = StatementValue::new();

    let (body, name, decorator_list) = match data {
        FunctionDefOptions::StmtFunctionDef(value) => {
            (value.body, value.name, value.decorator_list)
        }
        FunctionDefOptions::StmtAsyncFunctionDef(value) => {
            (value.body, value.name, value.decorator_list)
        }
    };

    statement_value.insert_function(FunctionDef::new(
        name.to_string(),
        path.to_path_buf(),
        class_name.clone(),
    ));

    for statement in body {
        statement_value.merge_statement_value(handle_statement(
            statement,
            path,
            class_name.clone(),
        ));
    }

    for decorator in decorator_list {
        let new_used_functions = handle_expression(&decorator, path, class_name.clone());

        statement_value.merge_expression_value(new_used_functions);
    }

    statement_value
}

fn handle_class_def(value: ast::StmtClassDef, path: &PathBuf) -> StatementValue {
    let mut statement_value = StatementValue::new();

    for statement in value.body {
        statement_value.merge_statement_value(handle_statement(
            statement,
            &path,
            Some(value.name.to_string()),
        ));

        statement_value.insert_class(value.name.to_string());
    }

    statement_value
}

fn get_empty_result() -> StatementValue {
    let statement_value = StatementValue::new();

    statement_value
}

fn handle_statement(
    statement: ast::Stmt,
    path: &PathBuf,
    class_name: Option<String>,
) -> StatementValue {
    dbg!(&statement);

    match statement {
        ast::Stmt::FunctionDef(value) => handle_function_def(
            FunctionDefOptions::StmtFunctionDef(value),
            &path,
            class_name,
        ),
        ast::Stmt::AsyncFunctionDef(value) => handle_function_def(
            FunctionDefOptions::StmtAsyncFunctionDef(value),
            &path,
            class_name,
        ),
        ast::Stmt::Expr(value) => {
            let mut statement_value = StatementValue::new();
            statement_value.merge_expression_value(handle_expression(
                &value.value,
                path,
                class_name,
            ));

            statement_value
        }
        ast::Stmt::ClassDef(value) => handle_class_def(value, &path),
        ast::Stmt::Return(value) => {
            let mut statement_value = StatementValue::new();

            let expr = value.value;

            if expr.is_none() {
                return statement_value;
            }

            let expr = *expr.unwrap();

            statement_value.merge_expression_value(handle_expression(&expr, path, class_name));

            statement_value
        }
        ast::Stmt::Assign(value) => {
            let mut statement_value = StatementValue::new();
            let mut left_side_values = HashSet::new();

            let right_side = handle_expression(&value.value, path, class_name.clone());

            dbg!(&right_side.used_names);
            let right_side_values = right_side.used_names.clone();

            statement_value.merge_expression_value(right_side);

            for target in &value.targets {
                let left_side = handle_expression(target, path, class_name.clone());

                left_side_values.extend(left_side.used_names.clone());

                dbg!(&left_side.used_names);
                statement_value.merge_expression_value(left_side);
            }

            if right_side_values.len() == 1 {
                for left_side in left_side_values {
                    statement_value.assignments.insert(
                        left_side,
                        right_side_values.iter().next().clone().unwrap().to_string(),
                    );
                }
            }

            statement_value
        }
        ast::Stmt::AugAssign(value) => {
            let mut statement_value = StatementValue::new();

            statement_value.merge_expression_value(handle_expression(
                &value.value,
                path,
                class_name.clone(),
            ));
            statement_value.merge_expression_value(handle_expression(
                &value.target,
                path,
                class_name.clone(),
            ));

            statement_value
        }
        ast::Stmt::AnnAssign(value) => {
            let mut statement_value = StatementValue::new();

            if value.value.is_some() {
                statement_value.merge_expression_value(handle_expression(
                    &value.value.clone().unwrap(),
                    path,
                    class_name.clone(),
                ));
            }

            statement_value.merge_expression_value(handle_expression(
                &value.annotation,
                path,
                class_name.clone(),
            ));
            statement_value.merge_expression_value(handle_expression(
                &value.target,
                path,
                class_name.clone(),
            ));

            statement_value
        }

        ast::Stmt::Delete(value) => {
            let mut statement_value = StatementValue::new();

            for target in &value.targets {
                statement_value.merge_expression_value(handle_expression(
                    target,
                    path,
                    class_name.clone(),
                ));
            }

            statement_value
        }
        ast::Stmt::For(value) => {
            let mut statement_value = StatementValue::new();

            statement_value.merge_expression_value(handle_expression(
                &value.target,
                path,
                class_name.clone(),
            ));
            statement_value.merge_expression_value(handle_expression(
                &value.iter,
                path,
                class_name.clone(),
            ));

            for statement in value.body {
                statement_value.merge_statement_value(handle_statement(
                    statement,
                    path,
                    class_name.clone(),
                ));
            }

            for statement in value.orelse {
                statement_value.merge_statement_value(handle_statement(
                    statement,
                    path,
                    class_name.clone(),
                ));
            }

            statement_value
        }
        ast::Stmt::AsyncFor(value) => {
            let mut statement_value = StatementValue::new();

            statement_value.merge_expression_value(handle_expression(
                &value.target,
                path,
                class_name.clone(),
            ));
            statement_value.merge_expression_value(handle_expression(
                &value.iter,
                path,
                class_name.clone(),
            ));

            for statement in value.body {
                statement_value.merge_statement_value(handle_statement(
                    statement,
                    path,
                    class_name.clone(),
                ));
            }

            for statement in value.orelse {
                statement_value.merge_statement_value(handle_statement(
                    statement,
                    path,
                    class_name.clone(),
                ));
            }

            statement_value
        }
        ast::Stmt::While(value) => {
            let mut statement_value = StatementValue::new();

            statement_value.merge_expression_value(handle_expression(
                &value.test,
                path,
                class_name.clone(),
            ));

            for statement in value.body {
                statement_value.merge_statement_value(handle_statement(
                    statement,
                    path,
                    class_name.clone(),
                ));
            }

            for statement in value.orelse {
                statement_value.merge_statement_value(handle_statement(
                    statement,
                    path,
                    class_name.clone(),
                ));
            }

            statement_value
        }
        ast::Stmt::If(value) => {
            let mut statement_value = StatementValue::new();

            statement_value.merge_expression_value(handle_expression(
                &value.test,
                path,
                class_name.clone(),
            ));

            for statement in value.body {
                statement_value.merge_statement_value(handle_statement(
                    statement,
                    path,
                    class_name.clone(),
                ));
            }

            for statement in value.orelse {
                statement_value.merge_statement_value(handle_statement(
                    statement,
                    path,
                    class_name.clone(),
                ));
            }

            statement_value
        }
        ast::Stmt::With(value) => {
            let mut statement_value = StatementValue::new();

            for statement in value.body {
                statement_value.merge_statement_value(handle_statement(
                    statement,
                    path,
                    class_name.clone(),
                ));
            }

            for item in value.items {
                statement_value.merge_expression_value(handle_expression(
                    &item.context_expr,
                    path,
                    class_name.clone(),
                ));

                if item.optional_vars.is_some() {
                    statement_value.merge_expression_value(handle_expression(
                        &item.optional_vars.clone().unwrap(),
                        path,
                        class_name.clone(),
                    ));
                }
            }

            statement_value
        }
        ast::Stmt::AsyncWith(value) => {
            let mut statement_value = StatementValue::new();

            for statement in value.body {
                statement_value.merge_statement_value(handle_statement(
                    statement,
                    path,
                    class_name.clone(),
                ));
            }

            for item in value.items {
                statement_value.merge_expression_value(handle_expression(
                    &item.context_expr,
                    path,
                    class_name.clone(),
                ));

                if item.optional_vars.is_some() {
                    statement_value.merge_expression_value(handle_expression(
                        &item.optional_vars.clone().unwrap(),
                        path,
                        class_name.clone(),
                    ));
                }
            }

            statement_value
        }
        ast::Stmt::Match(value) => {
            let mut statement_value = StatementValue::new();

            statement_value.merge_expression_value(handle_expression(
                &value.subject,
                path,
                class_name.clone(),
            ));

            for case in &value.cases {
                if case.guard.is_some() {
                    statement_value.merge_expression_value(handle_expression(
                        &case.guard.clone().unwrap(),
                        path,
                        class_name.clone(),
                    ));
                }

                for statement in case.body.clone() {
                    statement_value.merge_statement_value(handle_statement(
                        statement,
                        path,
                        class_name.clone(),
                    ));
                }
            }

            statement_value
        }
        ast::Stmt::Raise(value) => {
            let mut statement_value = StatementValue::new();

            if value.exc.is_some() {
                statement_value.merge_expression_value(handle_expression(
                    &value.exc.clone().unwrap(),
                    path,
                    class_name.clone(),
                ));
            }

            if value.cause.is_some() {
                statement_value.merge_expression_value(handle_expression(
                    &value.cause.clone().unwrap(),
                    path,
                    class_name.clone(),
                ));
            }

            statement_value
        }
        ast::Stmt::Try(value) => {
            let mut statement_value = StatementValue::new();

            for statement in value.body {
                statement_value.merge_statement_value(handle_statement(
                    statement,
                    path,
                    class_name.clone(),
                ));
            }

            for statement in value.orelse {
                statement_value.merge_statement_value(handle_statement(
                    statement,
                    path,
                    class_name.clone(),
                ));
            }

            for statement in value.finalbody {
                statement_value.merge_statement_value(handle_statement(
                    statement,
                    path,
                    class_name.clone(),
                ));
            }

            for handler in value.handlers {
                match handler {
                    ast::ExceptHandler::ExceptHandler(except_handler) => {
                        if except_handler.type_.is_some() {
                            statement_value.merge_expression_value(handle_expression(
                                &except_handler.type_.clone().unwrap(),
                                path,
                                class_name.clone(),
                            ));
                        }

                        for statement in except_handler.body {
                            statement_value.merge_statement_value(handle_statement(
                                statement,
                                path,
                                class_name.clone(),
                            ));
                        }
                    }
                }
            }

            statement_value
        }
        ast::Stmt::TryStar(value) => {
            let mut statement_value = StatementValue::new();

            for statement in value.body {
                statement_value.merge_statement_value(handle_statement(
                    statement,
                    path,
                    class_name.clone(),
                ));
            }

            for statement in value.orelse {
                statement_value.merge_statement_value(handle_statement(
                    statement,
                    path,
                    class_name.clone(),
                ));
            }

            for statement in value.finalbody {
                statement_value.merge_statement_value(handle_statement(
                    statement,
                    path,
                    class_name.clone(),
                ));
            }

            for handler in value.handlers {
                match handler {
                    ast::ExceptHandler::ExceptHandler(except_handler) => {
                        if except_handler.type_.is_some() {
                            statement_value.merge_expression_value(handle_expression(
                                &except_handler.type_.clone().unwrap(),
                                path,
                                class_name.clone(),
                            ));
                        }

                        for statement in except_handler.body {
                            statement_value.merge_statement_value(handle_statement(
                                statement,
                                path,
                                class_name.clone(),
                            ));
                        }
                    }
                }
            }

            statement_value
        }
        ast::Stmt::Assert(value) => {
            let mut statement_value = StatementValue::new();

            if value.msg.is_some() {
                statement_value.merge_expression_value(handle_expression(
                    &value.msg.clone().unwrap(),
                    path,
                    class_name.clone(),
                ));
            }

            statement_value.merge_expression_value(handle_expression(
                &value.test,
                path,
                class_name.clone(),
            ));

            statement_value
        }
        ast::Stmt::Import(value) => {
            let mut statement_value = StatementValue::new();

            let mut file_data = FileData::new(path.to_path_buf());

            for name in &value.names {
                let path = name.name.to_string();

                let relevant_name = match name.name.split(".").last() {
                    None => name.name.to_string(),
                    Some(inner_name) => inner_name.to_string(),
                };

                let alias = name
                    .asname
                    .clone()
                    .map_or(None, |inner| Some(inner.to_string()));

                let import_data = Imports::new(path, relevant_name, alias);

                file_data
                    .imports
                    .entry(name.name.to_string())
                    .and_modify(|list| list.push(import_data.clone()))
                    .or_insert(vec![import_data]);
            }

            statement_value.files.insert(path.to_path_buf(), file_data);

            // [/home/saar/dev/rust/py-space/py-space-utils/src/parse_statement.rs:83:5] &statement = ImportFrom(
            //     StmtImportFrom {
            //         range: 13..56,
            //         module: Some(
            //             Identifier(
            //                 "tests.unused_two",
            //             ),
            //         ),
            //         names: [
            //             Alias {
            //                 range: 42..56,
            //                 name: Identifier(
            //                     "SAVE_FUNCTIONS",
            //                 ),
            //                 asname: None,
            //             },
            //         ],
            //         level: Some(
            //             Int(
            //                 0,
            //             ),
            //         ),
            //     },
            // )

            dbg!(value.names);

            statement_value
        }
        ast::Stmt::ImportFrom(_value) => get_empty_result(),
        ast::Stmt::Global(_value) => get_empty_result(),
        ast::Stmt::Nonlocal(_value) => get_empty_result(),
        ast::Stmt::Pass(_value) => get_empty_result(),
        ast::Stmt::Break(_value) => get_empty_result(),
        ast::Stmt::Continue(_value) => get_empty_result(),

        _ => get_empty_result(),
    }
}

pub fn extract_file_data(path: PathBuf) -> Result<StatementValue, PySpaceError> {
    let mut statement_value = StatementValue::new();

    let content = match fs::read_to_string(&path) {
        Ok(value) => value,
        Err(error) => {
            dbg!("faild to read file: {}", &error);

            return Err(PySpaceError::FailedToReadFile(error.to_string()));
        }
    };

    let program = match parse(&content, Mode::Module, "<embedded>") {
        Ok(value) => value,
        Err(error) => {
            dbg!("failed to parse file: {}", &error);

            return Err(PySpaceError::FailedToParseFile(error.to_string()));
        }
    };

    let body = match program {
        ast::Mod::Module(module) => module.body,
        _ => {
            return Err(PySpaceError::FailedToParseFile(
                "Expected module".to_string(),
            ));
        }
    };

    for statement in body {
        statement_value.merge_statement_value(handle_statement(statement, &path, None));
    }

    Ok(statement_value)
}
