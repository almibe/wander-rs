// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::collections::{HashMap, HashSet};
use std::fmt::Display;

use serde::{Serialize, Deserialize};

use crate::bindings::Bindings;

use crate::parser::Element;
use crate::translation::express;
use crate::{WanderError, WanderType, WanderValue};

#[doc(hidden)]
#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
pub enum Expression {
    Boolean(bool),
    Int(i64),
    String(String),
    Name(String),
    HostFunction(String),
    Let(Vec<(String, Expression)>, Box<Expression>),
    Application(Vec<Expression>),
    Conditional(Box<Expression>, Box<Expression>, Box<Expression>),
    Lambda(String, WanderType, WanderType, Box<Element>),
    Tuple(Vec<Expression>),
    List(Vec<Expression>),
    Set(HashSet<Expression>),
    Record(HashMap<String, Expression>),
    Nothing,
}

impl core::hash::Hash for Expression {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);
    }
}

pub fn eval<T: Clone + Display + PartialEq + Eq + std::fmt::Debug>(
    expression: &Expression,
    bindings: &mut Bindings<T>,
) -> Result<WanderValue<T>, WanderError> {
    match expression {
        Expression::Boolean(value) => Ok(WanderValue::Boolean(*value)),
        Expression::Int(value) => Ok(WanderValue::Int(*value)),
        Expression::String(value) => Ok(WanderValue::String(unescape_string(value.to_string()))),
        Expression::Let(decls, body) => handle_let(decls.clone(), *body.clone(), bindings),
        Expression::Name(name) => read_name(name, bindings),
        Expression::Application(expressions) => handle_function_call(expressions, bindings),
        Expression::Conditional(c, i, e) => handle_conditional(c, i, e, bindings),
        Expression::List(values) => handle_list(values, bindings),
        Expression::Nothing => Ok(WanderValue::Nothing),
        Expression::Tuple(values) => handle_tuple(values, bindings),
        Expression::Record(values) => handle_record(values, bindings),
        Expression::Lambda(name, input, output, body) => handle_lambda(name, input, output, body),
        Expression::Set(values) => handle_set(values, bindings),
        Expression::HostFunction(name) => handle_host_function(name, bindings),
        // Expression::Grouping(expressions) => handle_grouping(expressions.clone(), bindings),
    }
}

fn unescape_string(value: String) -> String {
    let mut result = String::new();
    let mut last_char = ' ';
    let mut idx = 0;
    value.chars().for_each(|c| {
        if idx == 0 || idx == value.chars().count() - 1 {
            idx += 1;
        } else {
            idx += 1;
            if last_char == '\\' {
                match c {
                    'n' => {
                        result.push('\n');
                        last_char = c
                    }
                    '\\' => {
                        result.push('\\');
                        last_char = ' '
                    }
                    't' => {
                        result.push('\t');
                        last_char = c
                    }
                    '"' => {
                        result.push(c);
                        last_char = c
                    }
                    _ => todo!(),
                }
            } else if c == '\\' {
                last_char = c
            } else {
                result.push(c);
                last_char = c
            }
        }
    });
    if last_char == '\\' {
        panic!()
    }
    result
}

fn handle_host_function<T: Clone + Display + PartialEq + Eq>(name: &str, bindings: &mut Bindings<T>) -> Result<WanderValue<T>, WanderError> {
    let host_function = bindings.read_host_function(&name.to_owned()).unwrap();
    let params = host_function.binding().parameters;
    let mut arguments = vec![];
    for (name, wander_type) in params {
        match bindings.read(&name) {
            Some(value) => arguments.push(value),
            None => return Err(WanderError(format!("Could not read {}", name))),
        }
    }
    host_function.run(&arguments, bindings)
}

fn handle_set<T: Clone + Display + PartialEq + Eq + std::fmt::Debug>(
    expressions: &HashSet<Expression>,
    bindings: &mut Bindings<T>,
) -> Result<WanderValue<T>, WanderError> {
    let mut results = HashSet::new();
    for expression in expressions {
        match eval(expression, bindings) {
            Ok(value) => results.insert(value),
            Err(err) => return Err(err),
        };
    }
    Ok(WanderValue::Set(results))
}

fn handle_tuple<T: Clone + Display + PartialEq + Eq + std::fmt::Debug>(
    expressions: &Vec<Expression>,
    bindings: &mut Bindings<T>,
) -> Result<WanderValue<T>, WanderError> {
    let mut results = vec![];
    for expression in expressions {
        match eval(expression, bindings) {
            Ok(value) => results.push(value),
            Err(err) => return Err(err),
        }
    }
    Ok(WanderValue::Tuple(results))
}

fn handle_record<T: Clone + Display + PartialEq + Eq + std::fmt::Debug>(
    expressions: &HashMap<String, Expression>,
    bindings: &mut Bindings<T>,
) -> Result<WanderValue<T>, WanderError> {
    let mut results = HashMap::new();
    for (name, expression) in expressions {
        match eval(expression, bindings) {
            Ok(value) => results.insert(name.to_owned(), value),
            Err(err) => return Err(err),
        };
    }
    Ok(WanderValue::Record(results))
}

fn handle_list<T: Clone + Display + PartialEq + Eq + std::fmt::Debug>(
    expressions: &Vec<Expression>,
    bindings: &mut Bindings<T>,
) -> Result<WanderValue<T>, WanderError> {
    let mut results = vec![];
    for expression in expressions {
        match eval(expression, bindings) {
            Ok(value) => results.push(value),
            Err(err) => return Err(err),
        }
    }
    Ok(WanderValue::List(results))
}

fn handle_lambda<T: Clone + PartialEq + Eq>(
    name: &str,
    input: &WanderType,
    output: &WanderType,
    body: &Element,
) -> Result<WanderValue<T>, WanderError> {
    Ok(WanderValue::Lambda(
        name.to_owned(),
        input.clone(),
        output.clone(),
        Box::new(body.clone()),
    ))
}

fn handle_conditional<T: Clone + Display + PartialEq + Eq + std::fmt::Debug>(
    cond: &Expression,
    ife: &Expression,
    elsee: &Expression,
    bindings: &mut Bindings<T>,
) -> Result<WanderValue<T>, WanderError> {
    match eval(cond, bindings)? {
        WanderValue::Boolean(true) => eval(ife, bindings),
        WanderValue::Boolean(false) => eval(elsee, bindings),
        value => Err(WanderError(format!(
            "Conditionals require a bool value found, {value}"
        ))),
    }
}

fn handle_function_call<T: Clone + Display + PartialEq + Eq + std::fmt::Debug>(
    expressions: &Vec<Expression>,
    bindings: &mut Bindings<T>,
) -> Result<WanderValue<T>, WanderError> {
    let mut expressions = expressions.clone();
    expressions.reverse();
    while !expressions.is_empty() {
        let expression = expressions.pop().unwrap();
        match expression {
            Expression::Lambda(name, input, output, lambda_body) => {
                if expressions.is_empty() {
                    return Ok(WanderValue::Lambda(name, input, output, lambda_body))
                } else {
                    let argument_expression = expressions.pop().unwrap();
                    let argument_value = eval(&argument_expression, bindings)?;
                    bindings.bind(name, argument_value);
                    let function = eval(&express(&lambda_body)?, bindings)?;
                    match function {
                        WanderValue::Lambda(_, _, _, b) => {
                            match eval(&express(&b)?, bindings) {
                                Ok(value) => expressions.push(value_to_expression(value)),
                                Err(err) => return Err(err),
                            }                            
                        },
                        _ => if expressions.is_empty() {
                            return Ok(function)
                        } else {
                            return Err(WanderError(format!("Invalid function call.")))
                        },
                    }
                }
            },
            Expression::Name(name) => {
                match eval(&Expression::Name(name), bindings) {
                    Ok(value) => match value {
                        WanderValue::Lambda(p, i, o, b) => {
                            let argument_expression = expressions.pop().unwrap();
                            let argument_value = eval(&argument_expression, bindings)?;
                            bindings.bind(p, argument_value);        
                            match eval(&express(&b)?, bindings) {
                                Ok(value) => expressions.push(value_to_expression(value)),
                                Err(err) => return Err(err),
                            }
                        },
                        _ => return Err(WanderError(format!("Invalid function call.")))
                    },
                    Err(err) => return Err(err),
                }
            },
            value => {
                if expressions.is_empty() {
                    return eval(&value, bindings)
                } else {
                    return Err(WanderError(format!("Invalid function call {value:?}.")))
                }
            }
        };
    }
    panic!()
}

fn value_to_expression<T: Clone + Display + PartialEq + Eq>(value: WanderValue<T>) -> Expression {
    match value {
        WanderValue::Boolean(value) => Expression::Boolean(value),
        WanderValue::Int(value) => Expression::Int(value),
        WanderValue::String(value) => Expression::String(value),
        WanderValue::Nothing => Expression::Nothing,
        WanderValue::Lambda(p, i, o, b) => Expression::Lambda(p, i, o, b),
        WanderValue::List(value) => todo!(),
        WanderValue::Tuple(value) => todo!(),
        WanderValue::Set(value) => todo!(),
        WanderValue::Record(value) => todo!(),
        WanderValue::HostValue(value) => todo!(),
    }
}

fn handle_let<T: Clone + Display + PartialEq + Eq + std::fmt::Debug>(
    decls: Vec<(String, Expression)>,
    body: Expression,
    bindings: &mut Bindings<T>,
) -> Result<WanderValue<T>, WanderError> {

    for (name, body) in decls {
        handle_decl(name, body, bindings)?;
    }
    eval(&body, bindings)
}

fn handle_decl<T: Clone + Display + PartialEq + Eq + std::fmt::Debug>(
    name: String,
    body: Expression,
    bindings: &mut Bindings<T>,
) -> Result<(), WanderError> {
    match eval(&body, bindings) {
        Ok(value) => {
            bindings.bind(name.to_string(), value);
            Ok(())        
        },
        Err(err) => return Err(err),
    }
}

fn read_name<T: Clone + PartialEq + Display + Eq + std::fmt::Debug>(
    name: &String,
    bindings: &mut Bindings<T>,
) -> Result<WanderValue<T>, WanderError> {
    if let Some(value) = bindings.read(name) {
        Ok(value)
    } else {
        match bindings.read_host_function(name) {
            Some(_) => todo!(), //Ok(WanderValue::HostedFunction(name.to_owned())),
            None => read_field(name, bindings),
        }
    }
}

fn read_field<T: Clone + PartialEq + Display + Eq + std::fmt::Debug>(
    name: &str,
    bindings: &mut Bindings<T>,
) -> Result<WanderValue<T>, WanderError> {
    let t = name
        .split('.')
        .map(|e| e.to_string())
        .collect::<Vec<String>>();
    let mut result = None;
    let (name, fields) = t.split_first().unwrap();
    if let Some(WanderValue::Record(value)) = bindings.read(&name.to_string()) {
        for field in fields {
            match result {
                Some(WanderValue::Record(r)) => result = Some(r.get(field).unwrap().clone()),
                Some(x) => {
                    return Err(WanderError(format!(
                        "Could not access field {field} in {x}."
                    )))
                }
                None => match value.get(field) {
                    Some(r) => result = Some(r.clone()),
                    None => return Err(WanderError(format!("Could not read field {name}"))),
                },
            }
        }
        Ok(result.unwrap().clone())
    } else {
        Err(WanderError(format!("Error looking up {name}")))
    }
}

fn call_function<T: Clone + Display + PartialEq + Eq + std::fmt::Debug>(
    name: &String,
    arguments: &Vec<Expression>,
    bindings: &mut Bindings<T>,
) -> Result<WanderValue<T>, WanderError> {
    let mut argument_values = vec![];
    for argument in arguments {
        match eval(argument, bindings) {
            Ok(value) => argument_values.push(value),
            Err(err) => return Err(err),
        }
    }
    match bindings.read(name) {
        //found other value (err), will evntually handle lambdas here
        Some(_) => Err(WanderError(format!("Function {} is not defined.", &name))),
        None => match bindings.read_host_function(name) {
            None => Err(WanderError(format!("Function {} is not defined.", name))),
            Some(function) => {
                if argument_values.len() == function.binding().parameters.len() {
                    function.run(&argument_values, bindings)
                } else {
                    // Ok(WanderValue::PartialApplication(Box::new(
                    //     PartialApplication {
                    //         arguments: argument_values,
                    //         callee: WanderValue::HostedFunction(name.clone()),
                    //     },
                    // )))
                    todo!()
                }
            }
        },
    }
}
