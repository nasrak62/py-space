use std::path::PathBuf;

use crate::models::{expression_value::ExpressionValue, function_def::FunctionDef};
use rustpython_parser::ast;

const POSSIBLE_FUNCTIONS_IGNORE: [&str; 1] = ["staticmethod"];

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

fn extract_from_generators(
    generators: &Vec<ast::Comprehension>,
    path: &PathBuf,
    class_name: Option<String>,
) -> ExpressionValue {
    let mut expression_value = ExpressionValue::new();

    for generator in generators {
        expression_value.merge_expression_result(handle_expression(
            &generator.target,
            path,
            class_name.clone(),
        ));

        expression_value.merge_expression_result(handle_expression(
            &generator.iter,
            path,
            class_name.clone(),
        ));

        for inner_value in &generator.ifs {
            expression_value.merge_expression_result(handle_expression(
                &inner_value,
                path,
                class_name.clone(),
            ));
        }
    }

    expression_value
}

pub fn handle_expression(
    expression: &ast::Expr,
    path: &PathBuf,
    class_name: Option<String>,
) -> ExpressionValue {
    let mut expression_value = ExpressionValue::new();

    dbg!(&expression);

    match expression {
        ast::Expr::Call(value) => {
            let name = extract_called_function_id(value.clone());

            if name.is_some() {
                let name_value = name.unwrap();
                let full_name = match class_name.clone() {
                    Some(class_value) => class_value + "." + &name_value,
                    None => name_value,
                };

                expression_value.insert(full_name.clone());
                expression_value.insert_name(full_name);
            }

            for arg in &value.args {
                expression_value.merge_expression_result(handle_expression(
                    &arg,
                    path,
                    class_name.clone(),
                ));
            }

            for keyword in &value.keywords {
                expression_value.merge_expression_result(handle_expression(
                    &keyword.value,
                    path,
                    class_name.clone(),
                ));
            }
        }

        ast::Expr::BoolOp(value) => {
            for data in &value.values {
                expression_value.merge_expression_result(handle_expression(
                    data,
                    path,
                    class_name.clone(),
                ));
            }
        }

        ast::Expr::NamedExpr(value) => {
            expression_value.merge_expression_result(handle_expression(
                &value.target,
                path,
                class_name.clone(),
            ));

            expression_value.merge_expression_result(handle_expression(
                &value.value,
                path,
                class_name.clone(),
            ));
        }
        ast::Expr::BinOp(value) => {
            expression_value.merge_expression_result(handle_expression(
                &value.left,
                path,
                class_name.clone(),
            ));

            expression_value.merge_expression_result(handle_expression(
                &value.right,
                path,
                class_name.clone(),
            ));
        }
        ast::Expr::UnaryOp(value) => {
            expression_value.merge_expression_result(handle_expression(
                &value.operand,
                path,
                class_name.clone(),
            ));
        }
        ast::Expr::Lambda(value) => {
            expression_value.merge_expression_result(handle_expression(
                &value.body,
                path,
                class_name.clone(),
            ));
        }
        ast::Expr::IfExp(value) => {
            expression_value.merge_expression_result(handle_expression(
                &value.test,
                path,
                class_name.clone(),
            ));

            expression_value.merge_expression_result(handle_expression(
                &value.orelse,
                path,
                class_name.clone(),
            ));

            expression_value.merge_expression_result(handle_expression(
                &value.body,
                path,
                class_name.clone(),
            ));
        }

        ast::Expr::Dict(value) => {
            for key in &value.keys {
                if key.is_none() {
                    continue;
                }

                expression_value.merge_expression_result(handle_expression(
                    &key.clone().unwrap(),
                    path,
                    class_name.clone(),
                ));
            }

            for dict_value in &value.values {
                expression_value.merge_expression_result(handle_expression(
                    &dict_value,
                    path,
                    class_name.clone(),
                ));
            }
        }

        ast::Expr::Set(value) => {
            for element in &value.elts {
                expression_value.merge_expression_result(handle_expression(
                    &element,
                    path,
                    class_name.clone(),
                ));
            }
        }

        ast::Expr::ListComp(value) => {
            expression_value.merge_expression_result(handle_expression(
                &value.elt,
                path,
                class_name.clone(),
            ));

            expression_value.merge_expression_result(extract_from_generators(
                &value.generators,
                path,
                class_name.clone(),
            ));
        }
        ast::Expr::SetComp(value) => {
            expression_value.merge_expression_result(handle_expression(
                &value.elt,
                path,
                class_name.clone(),
            ));

            expression_value.merge_expression_result(extract_from_generators(
                &value.generators,
                path,
                class_name.clone(),
            ));
        }
        ast::Expr::DictComp(value) => {
            expression_value.merge_expression_result(handle_expression(
                &value.key,
                path,
                class_name.clone(),
            ));

            expression_value.merge_expression_result(handle_expression(
                &value.value,
                path,
                class_name.clone(),
            ));

            expression_value.merge_expression_result(extract_from_generators(
                &value.generators,
                path,
                class_name.clone(),
            ));
        }

        ast::Expr::GeneratorExp(value) => {
            expression_value.merge_expression_result(handle_expression(
                &value.elt,
                path,
                class_name.clone(),
            ));

            expression_value.merge_expression_result(extract_from_generators(
                &value.generators,
                path,
                class_name.clone(),
            ));
        }

        ast::Expr::Await(value) => {
            expression_value.merge_expression_result(handle_expression(
                &value.value,
                path,
                class_name.clone(),
            ));
        }

        ast::Expr::Yield(value) => {
            if value.value.is_some() {
                expression_value.merge_expression_result(handle_expression(
                    &value.value.clone().unwrap(),
                    path,
                    class_name.clone(),
                ));
            }
        }

        ast::Expr::YieldFrom(value) => {
            expression_value.merge_expression_result(handle_expression(
                &value.value,
                path,
                class_name.clone(),
            ));
        }

        ast::Expr::Compare(value) => {
            expression_value.merge_expression_result(handle_expression(
                &value.left,
                path,
                class_name.clone(),
            ));

            for comparator in &value.comparators {
                expression_value.merge_expression_result(handle_expression(
                    &comparator,
                    path,
                    class_name.clone(),
                ));
            }
        }

        ast::Expr::FormattedValue(value) => {
            expression_value.merge_expression_result(handle_expression(
                &value.value,
                path,
                class_name.clone(),
            ));

            if value.format_spec.is_some() {
                expression_value.merge_expression_result(handle_expression(
                    &value.format_spec.clone().unwrap(),
                    path,
                    class_name.clone(),
                ));
            }
        }

        ast::Expr::JoinedStr(value) => {
            for inner_value in &value.values {
                expression_value.merge_expression_result(handle_expression(
                    &inner_value,
                    path,
                    class_name.clone(),
                ));
            }
        }

        ast::Expr::Constant(_value) => {}
        ast::Expr::Attribute(value) => {
            let current_value = handle_expression(&value.value, path, class_name.clone());
            let used_names = current_value.used_names.clone();
            expression_value.merge_expression_result(current_value);

            let first_name = used_names.iter().next();

            if first_name.is_some() {
                let full_name = format!("{}.{}", first_name.unwrap(), value.attr);

                expression_value.insert_attribute(full_name);
            }
        }

        ast::Expr::Subscript(value) => {
            expression_value.merge_expression_result(handle_expression(
                &value.value,
                path,
                class_name.clone(),
            ));

            expression_value.merge_expression_result(handle_expression(
                &value.slice,
                path,
                class_name.clone(),
            ));
        }

        ast::Expr::Starred(value) => {
            expression_value.merge_expression_result(handle_expression(
                &value.value,
                path,
                class_name.clone(),
            ));
        }

        ast::Expr::Name(value) => {
            let name = value.id.to_string();
            expression_value.insert_name(name.clone());

            if !POSSIBLE_FUNCTIONS_IGNORE.contains(&name.as_str()) {
                expression_value.insert_possible_function(FunctionDef::new(
                    name,
                    path.to_path_buf(),
                    class_name.clone(),
                ));
            }
        }
        ast::Expr::List(value) => {
            for inner_value in &value.elts {
                expression_value.merge_expression_result(handle_expression(
                    &inner_value,
                    path,
                    class_name.clone(),
                ));
            }
        }

        ast::Expr::Tuple(value) => {
            for inner_value in &value.elts {
                expression_value.merge_expression_result(handle_expression(
                    &inner_value,
                    path,
                    class_name.clone(),
                ));
            }
        }

        ast::Expr::Slice(value) => {
            if value.lower.is_some() {
                expression_value.merge_expression_result(handle_expression(
                    &value.lower.clone().unwrap(),
                    path,
                    class_name.clone(),
                ));
            }
            if value.upper.is_some() {
                expression_value.merge_expression_result(handle_expression(
                    &value.upper.clone().unwrap(),
                    path,
                    class_name.clone(),
                ));
            }

            if value.step.is_some() {
                expression_value.merge_expression_result(handle_expression(
                    &value.step.clone().unwrap(),
                    path,
                    class_name.clone(),
                ));
            }
        }
    };

    expression_value
}
