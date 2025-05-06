use rustpython_parser::ast;
use rustpython_parser::{parse, Mode};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

use crate::errors::py_space::PySpaceError;
use crate::modals::function_def::FunctionDef;

type FunctionsValue = (HashSet<FunctionDef>, HashSet<String>);

enum FunctionDefOptions {
    StmtFunctionDef(ast::StmtFunctionDef),
    StmtAsyncFunctionDef(ast::StmtAsyncFunctionDef),
}

fn extract_inner_caller_id(value: ast::ExprAttribute) -> Option<String> {
    let id = match *value.value {
        ast::Expr::Call(inner_call) => match *inner_call.func {
            ast::Expr::Name(inner_name) => inner_name.id.to_string(),
            _ => return None,
        },

        ast::Expr::Name(inner_name) => inner_name.id.to_string(),

        _ => return None,
    };

    Some(format!("{}.{}", id, value.attr))
}

fn extract_called_function_id(data: ast::ExprCall) -> Option<String> {
    let func_value = *data.func;

    match func_value {
        ast::Expr::Name(value) => Some(value.id.to_string()),
        ast::Expr::Attribute(value) => extract_inner_caller_id(value),
        _ => None,
    }
}

fn handle_expression(expression: &ast::Expr, class_name: Option<String>) -> HashSet<String> {
    let mut used_functions = HashSet::new();

    match expression {
        ast::Expr::Call(value) => {
            dbg!("expr::call", &value, &class_name);

            let name = extract_called_function_id(value.clone());

            if name.is_some() {
                let name_value = name.unwrap();
                let full_name = match class_name.clone() {
                    Some(value) => value + "." + &name_value,
                    None => name_value,
                };

                used_functions.insert(full_name);
            }

            for arg in &value.args {
                used_functions.extend(handle_expression(&arg, class_name.clone()));
            }

            for keyword in &value.keywords {
                used_functions.extend(handle_expression(&keyword.value, class_name.clone()));
            }
        }

        ast::Expr::BoolOp(value) => {
            for data in &value.values {
                used_functions.extend(handle_expression(data, class_name.clone()));
            }
        }

        ast::Expr::NamedExpr(value) => {
            used_functions.extend(handle_expression(&value.target, class_name.clone()));
            used_functions.extend(handle_expression(&value.value, class_name.clone()));
        }

        ast::Expr::BinOp(value) => {
            used_functions.extend(handle_expression(&value.left, class_name.clone()));
            used_functions.extend(handle_expression(&value.right, class_name.clone()));
        }

        ast::Expr::UnaryOp(value) => {
            used_functions.extend(handle_expression(&value.operand, class_name.clone()));
        }

        ast::Expr::Lambda(value) => {
            used_functions.extend(handle_expression(&value.body, class_name.clone()));
        }

        ast::Expr::IfExp(value) => {
            used_functions.extend(handle_expression(&value.test, class_name.clone()));
            used_functions.extend(handle_expression(&value.orelse, class_name.clone()));
            used_functions.extend(handle_expression(&value.body, class_name.clone()));
        }

        ast::Expr::Dict(value) => {
            for key in &value.keys {
                if key.is_some() {
                    used_functions
                        .extend(handle_expression(&key.clone().unwrap(), class_name.clone()));
                }
            }

            for dict_value in &value.values {
                used_functions.extend(handle_expression(&dict_value, class_name.clone()));
            }
        }

        ast::Expr::Set(value) => {
            for element in &value.elts {
                used_functions.extend(handle_expression(&element, class_name.clone()));
            }
        }

        ast::Expr::ListComp(value) => {
            used_functions.extend(handle_expression(&value.elt, class_name.clone()));

            for generator in &value.generators {
                used_functions.extend(handle_expression(&generator.target, class_name.clone()));
                used_functions.extend(handle_expression(&generator.iter, class_name.clone()));

                for inner_value in &generator.ifs {
                    used_functions.extend(handle_expression(&inner_value, class_name.clone()));
                }
            }
        }

        ast::Expr::SetComp(value) => {
            used_functions.extend(handle_expression(&value.elt, class_name.clone()));

            for generator in &value.generators {
                used_functions.extend(handle_expression(&generator.target, class_name.clone()));
                used_functions.extend(handle_expression(&generator.iter, class_name.clone()));

                for inner_value in &generator.ifs {
                    used_functions.extend(handle_expression(&inner_value, class_name.clone()));
                }
            }
        }
        ast::Expr::DictComp(value) => {
            used_functions.extend(handle_expression(&value.key, class_name.clone()));
            used_functions.extend(handle_expression(&value.value, class_name.clone()));

            for generator in &value.generators {
                used_functions.extend(handle_expression(&generator.target, class_name.clone()));
                used_functions.extend(handle_expression(&generator.iter, class_name.clone()));

                for inner_value in &generator.ifs {
                    used_functions.extend(handle_expression(&inner_value, class_name.clone()));
                }
            }
        }
        ast::Expr::GeneratorExp(value) => {
            used_functions.extend(handle_expression(&value.elt, class_name.clone()));

            for generator in &value.generators {
                used_functions.extend(handle_expression(&generator.target, class_name.clone()));
                used_functions.extend(handle_expression(&generator.iter, class_name.clone()));

                for inner_value in &generator.ifs {
                    used_functions.extend(handle_expression(&inner_value, class_name.clone()));
                }
            }
        }
        ast::Expr::Await(value) => {
            used_functions.extend(handle_expression(&value.value, class_name.clone()));
        }
        ast::Expr::Yield(value) => {
            if value.value.is_some() {
                used_functions.extend(handle_expression(
                    &value.value.clone().unwrap(),
                    class_name.clone(),
                ));
            }
        }
        ast::Expr::YieldFrom(value) => {
            used_functions.extend(handle_expression(&value.value, class_name.clone()));
        }
        ast::Expr::Compare(value) => {
            used_functions.extend(handle_expression(&value.left, class_name.clone()));

            for comparator in &value.comparators {
                used_functions.extend(handle_expression(&comparator, class_name.clone()));
            }
        }
        ast::Expr::FormattedValue(value) => {
            used_functions.extend(handle_expression(&value.value, class_name.clone()));

            if value.format_spec.is_some() {
                used_functions.extend(handle_expression(
                    &value.format_spec.clone().unwrap(),
                    class_name.clone(),
                ));
            }
        }
        ast::Expr::JoinedStr(value) => {
            for inner_value in &value.values {
                used_functions.extend(handle_expression(&inner_value, class_name.clone()));
            }
        }
        ast::Expr::Constant(_value) => {}
        ast::Expr::Attribute(value) => {
            used_functions.extend(handle_expression(&value.value, class_name.clone()));
        }
        ast::Expr::Subscript(value) => {
            used_functions.extend(handle_expression(&value.value, class_name.clone()));
            used_functions.extend(handle_expression(&value.slice, class_name.clone()));
        }
        ast::Expr::Starred(value) => {
            used_functions.extend(handle_expression(&value.value, class_name));
        }
        ast::Expr::Name(_value) => {}
        ast::Expr::List(value) => {
            for inner_value in &value.elts {
                used_functions.extend(handle_expression(&inner_value, class_name.clone()));
            }
        }
        ast::Expr::Tuple(value) => {
            for inner_value in &value.elts {
                used_functions.extend(handle_expression(&inner_value, class_name.clone()));
            }
        }
        ast::Expr::Slice(value) => {
            if value.lower.is_some() {
                used_functions.extend(handle_expression(
                    &value.lower.clone().unwrap(),
                    class_name.clone(),
                ));
            }

            if value.upper.is_some() {
                used_functions.extend(handle_expression(
                    &value.upper.clone().unwrap(),
                    class_name.clone(),
                ));
            }

            if value.step.is_some() {
                used_functions.extend(handle_expression(
                    &value.step.clone().unwrap(),
                    class_name.clone(),
                ));
            }
        }
    };

    used_functions
}

fn handle_function_def(
    data: FunctionDefOptions,
    path: &PathBuf,
    class_name: Option<String>,
) -> FunctionsValue {
    let mut functions = HashSet::new();
    let mut used_functions = HashSet::new();

    let (body, name, decorator_list) = match data {
        FunctionDefOptions::StmtFunctionDef(value) => {
            (value.body, value.name, value.decorator_list)
        }
        FunctionDefOptions::StmtAsyncFunctionDef(value) => {
            (value.body, value.name, value.decorator_list)
        }
    };

    functions.insert(FunctionDef::new(
        name.to_string(),
        path.to_path_buf(),
        class_name.clone(),
    ));

    for statement in body {
        let (new_functions, new_used_functions) =
            handle_statement(statement, path, class_name.clone());

        functions.extend(new_functions);
        used_functions.extend(new_used_functions);
    }

    for decorator in decorator_list {
        let new_used_functions = handle_expression(&decorator, class_name.clone());

        used_functions.extend(new_used_functions);
    }

    (functions, used_functions)
}

fn handle_class_def(value: ast::StmtClassDef, path: &PathBuf) -> FunctionsValue {
    let mut functions = HashSet::new();
    let mut used_functions = HashSet::new();

    for statement in value.body {
        let (new_functions, new_used_functions) =
            handle_statement(statement, &path, Some(value.name.to_string()));

        functions.extend(new_functions);
        used_functions.extend(new_used_functions);
    }

    (functions, used_functions)
}

fn handle_statement(
    statement: ast::Stmt,
    path: &PathBuf,
    class_name: Option<String>,
) -> FunctionsValue {
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
            let functions = HashSet::new();
            let new_used_functions = handle_expression(&value.value, class_name);

            (functions, new_used_functions)
        }
        ast::Stmt::ClassDef(value) => handle_class_def(value, &path),
        _ => {
            let functions = HashSet::new();
            let used_functions = HashSet::new();

            (functions, used_functions)
        }
    }
}

pub fn extract_file_data(path: PathBuf) -> Result<FunctionsValue, PySpaceError> {
    let mut functions = HashSet::new();
    let mut used_functions = HashSet::new();

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
            ))
        }
    };

    for statement in body {
        let (new_functions, new_used_functions) = handle_statement(statement, &path, None);

        functions.extend(new_functions);
        used_functions.extend(new_used_functions);
    }

    Ok((functions, used_functions))
}
