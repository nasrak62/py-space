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

fn extract_called_function_id(data: ast::ExprCall) -> Option<String> {
    let func_value = *data.func;

    match func_value {
        ast::Expr::Name(value) => Some(value.id.to_string()),
        ast::Expr::Attribute(value) => Some(value.attr.to_string()),
        _ => None,
    }
}

fn handle_expression(expression: &ast::Expr) -> HashSet<String> {
    let mut used_functions = HashSet::new();

    dbg!(&expression);

    match expression {
        ast::Expr::Call(value) => {
            dbg!(&value);

            let name = extract_called_function_id(value.clone());

            if name.is_some() {
                used_functions.insert(name.unwrap());
            }

            for arg in &value.args {
                used_functions.extend(handle_expression(&arg));
            }

            for keyword in &value.keywords {
                used_functions.extend(handle_expression(&keyword.value));
            }
        }

        ast::Expr::BoolOp(value) => {
            for data in &value.values {
                used_functions.extend(handle_expression(data));
            }
        }

        ast::Expr::NamedExpr(value) => {
            used_functions.extend(handle_expression(&value.target));
            used_functions.extend(handle_expression(&value.value));
        }

        ast::Expr::BinOp(value) => {
            used_functions.extend(handle_expression(&value.left));
            used_functions.extend(handle_expression(&value.right));
        }

        ast::Expr::UnaryOp(value) => {
            used_functions.extend(handle_expression(&value.operand));
        }

        ast::Expr::Lambda(value) => {
            used_functions.extend(handle_expression(&value.body));
        }

        ast::Expr::IfExp(value) => {
            used_functions.extend(handle_expression(&value.test));
            used_functions.extend(handle_expression(&value.orelse));
            used_functions.extend(handle_expression(&value.body));
        }

        ast::Expr::Dict(value) => {
            for key in &value.keys {
                if key.is_some() {
                    used_functions.extend(handle_expression(&key.clone().unwrap()));
                }
            }

            for dict_value in &value.values {
                used_functions.extend(handle_expression(&dict_value));
            }
        }

        ast::Expr::Set(value) => {
            for element in &value.elts {
                used_functions.extend(handle_expression(&element));
            }
        }

        ast::Expr::ListComp(value) => {
            used_functions.extend(handle_expression(&value.elt));

            for generator in &value.generators {
                used_functions.extend(handle_expression(&generator.target));
                used_functions.extend(handle_expression(&generator.iter));

                for inner_value in &generator.ifs {
                    used_functions.extend(handle_expression(&inner_value));
                }
            }
        }

        ast::Expr::SetComp(value) => {
            used_functions.extend(handle_expression(&value.elt));

            for generator in &value.generators {
                used_functions.extend(handle_expression(&generator.target));
                used_functions.extend(handle_expression(&generator.iter));

                for inner_value in &generator.ifs {
                    used_functions.extend(handle_expression(&inner_value));
                }
            }
        }
        ast::Expr::DictComp(value) => {
            used_functions.extend(handle_expression(&value.key));
            used_functions.extend(handle_expression(&value.value));

            for generator in &value.generators {
                used_functions.extend(handle_expression(&generator.target));
                used_functions.extend(handle_expression(&generator.iter));

                for inner_value in &generator.ifs {
                    used_functions.extend(handle_expression(&inner_value));
                }
            }
        }
        ast::Expr::GeneratorExp(value) => {
            used_functions.extend(handle_expression(&value.elt));

            for generator in &value.generators {
                used_functions.extend(handle_expression(&generator.target));
                used_functions.extend(handle_expression(&generator.iter));

                for inner_value in &generator.ifs {
                    used_functions.extend(handle_expression(&inner_value));
                }
            }
        }
        ast::Expr::Await(value) => {
            used_functions.extend(handle_expression(&value.value));
        }
        ast::Expr::Yield(value) => {
            if value.value.is_some() {
                used_functions.extend(handle_expression(&value.value.clone().unwrap()));
            }
        }
        ast::Expr::YieldFrom(value) => {
            used_functions.extend(handle_expression(&value.value));
        }
        ast::Expr::Compare(value) => {
            used_functions.extend(handle_expression(&value.left));

            for comparator in &value.comparators {
                used_functions.extend(handle_expression(&comparator));
            }
        }
        ast::Expr::FormattedValue(value) => {
            used_functions.extend(handle_expression(&value.value));

            if value.format_spec.is_some() {
                used_functions.extend(handle_expression(&value.format_spec.clone().unwrap()));
            }
        }
        ast::Expr::JoinedStr(value) => {
            for inner_value in &value.values {
                used_functions.extend(handle_expression(&inner_value));
            }
        }
        ast::Expr::Constant(_value) => {}
        ast::Expr::Attribute(value) => {
            used_functions.extend(handle_expression(&value.value));
        }
        ast::Expr::Subscript(value) => {
            used_functions.extend(handle_expression(&value.value));
            used_functions.extend(handle_expression(&value.slice));
        }
        ast::Expr::Starred(value) => {
            used_functions.extend(handle_expression(&value.value));
        }
        ast::Expr::Name(_value) => {}
        ast::Expr::List(value) => {
            for inner_value in &value.elts {
                used_functions.extend(handle_expression(&inner_value));
            }
        }
        ast::Expr::Tuple(value) => {
            for inner_value in &value.elts {
                used_functions.extend(handle_expression(&inner_value));
            }
        }
        ast::Expr::Slice(value) => {
            if value.lower.is_some() {
                used_functions.extend(handle_expression(&value.lower.clone().unwrap()));
            }

            if value.upper.is_some() {
                used_functions.extend(handle_expression(&value.upper.clone().unwrap()));
            }

            if value.step.is_some() {
                used_functions.extend(handle_expression(&value.step.clone().unwrap()));
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
        let new_used_functions = handle_expression(&decorator);

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
    dbg!(statement.clone());

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
            let new_used_functions = handle_expression(&value.value);

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
