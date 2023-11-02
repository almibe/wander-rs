// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::collections::{HashMap, HashSet};
use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::environment::Environment;

use crate::identifier::Identifier;
use crate::parser::Element;
use crate::translation::express;
use crate::{HostType, WanderError, WanderValue};

#[doc(hidden)]
#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
pub enum Expression {
    Boolean(bool),
    Int(i64),
    String(String),
    Identifier(Identifier),
    Name(String),
    TaggedName(String, Box<Expression>),
    HostFunction(String),
    Let(
        Vec<(String, Option<Expression>, Expression)>,
        Box<Expression>,
    ),
    Application(Vec<Expression>),
    Conditional(Box<Expression>, Box<Expression>, Box<Expression>),
    Lambda(String, Option<String>, Option<String>, Box<Element>),
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

pub fn eval<T: Clone + Display + PartialEq + Eq + std::fmt::Debug + Serialize>(
    expression: &Expression,
    environment: &mut Environment<T>,
) -> Result<WanderValue<T>, WanderError> {
    match expression {
        Expression::Boolean(value) => Ok(WanderValue::Bool(*value)),
        Expression::Int(value) => Ok(WanderValue::Int(*value)),
        Expression::String(value) => Ok(WanderValue::String(unescape_string(value.to_string()))),
        Expression::Identifier(value) => Ok(WanderValue::Identifier(value.clone())),
        Expression::Let(decls, body) => handle_let(decls.clone(), *body.clone(), environment),
        Expression::Name(name) => read_name(name, environment),
        Expression::TaggedName(name, tag) => read_tagged_name(name, tag, environment),
        Expression::Application(expressions) => handle_function_call(expressions, environment),
        Expression::Conditional(c, i, e) => handle_conditional(c, i, e, environment),
        Expression::List(values) => handle_list(values, environment),
        Expression::Nothing => Ok(WanderValue::Nothing),
        Expression::Tuple(values) => handle_tuple(values, environment),
        Expression::Record(values) => handle_record(values, environment),
        Expression::Lambda(name, input, output, body) => {
            handle_lambda(name.clone(), input.clone(), output.clone(), body)
        }
        Expression::Set(values) => handle_set(values, environment),
        Expression::HostFunction(name) => handle_host_function(name, environment),
        // Expression::Grouping(expressions) => handle_grouping(expressions.clone(), environment),
    }
}

fn unescape_string(value: String) -> String {
    let mut result = String::new();
    let mut last_char = ' ';
    let mut idx = 0;
    value.chars().for_each(|c| {
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
    });
    if last_char == '\\' {
        panic!()
    }
    result
}

fn handle_host_function<T: HostType>(
    name: &str,
    environment: &mut Environment<T>,
) -> Result<WanderValue<T>, WanderError> {
    let host_function = environment.read_host_function(&name.to_owned()).unwrap();
    let params = host_function.binding().parameters;
    let mut arguments = vec![];
    for (name, wander_type) in params {
        match environment.read(&name) {
            Some(value) => arguments.push(value),
            None => return Err(WanderError(format!("Could not read {}", name))),
        }
    }
    host_function.run(&arguments, environment)
}

fn handle_set<T: HostType + Display>(
    expressions: &HashSet<Expression>,
    environment: &mut Environment<T>,
) -> Result<WanderValue<T>, WanderError> {
    let mut results = HashSet::new();
    for expression in expressions {
        match eval(expression, environment) {
            Ok(value) => results.insert(value),
            Err(err) => return Err(err),
        };
    }
    Ok(WanderValue::Set(results))
}

fn handle_tuple<T: HostType>(
    expressions: &Vec<Expression>,
    environment: &mut Environment<T>,
) -> Result<WanderValue<T>, WanderError> {
    let mut results = vec![];
    for expression in expressions {
        match eval(expression, environment) {
            Ok(value) => results.push(value),
            Err(err) => return Err(err),
        }
    }
    Ok(WanderValue::Tuple(results))
}

fn handle_record<T: HostType>(
    expressions: &HashMap<String, Expression>,
    environment: &mut Environment<T>,
) -> Result<WanderValue<T>, WanderError> {
    let mut results = HashMap::new();
    for (name, expression) in expressions {
        match eval(expression, environment) {
            Ok(value) => results.insert(name.to_owned(), value),
            Err(err) => return Err(err),
        };
    }
    Ok(WanderValue::Record(results))
}

fn handle_list<T: HostType>(
    expressions: &Vec<Expression>,
    environment: &mut Environment<T>,
) -> Result<WanderValue<T>, WanderError> {
    let mut results = vec![];
    for expression in expressions {
        match eval(expression, environment) {
            Ok(value) => results.push(value),
            Err(err) => return Err(err),
        }
    }
    Ok(WanderValue::List(results))
}

fn handle_lambda<T: Clone + PartialEq + Eq>(
    name: String,
    input: Option<String>,
    output: Option<String>,
    body: &Element,
) -> Result<WanderValue<T>, WanderError> {
    Ok(WanderValue::Lambda(
        name,
        input.clone(),
        output.clone(),
        Box::new(body.clone()),
    ))
}

fn handle_conditional<T: HostType + Display>(
    cond: &Expression,
    ife: &Expression,
    elsee: &Expression,
    environment: &mut Environment<T>,
) -> Result<WanderValue<T>, WanderError> {
    match eval(cond, environment)? {
        WanderValue::Bool(true) => eval(ife, environment),
        WanderValue::Bool(false) => eval(elsee, environment),
        value => Err(WanderError(format!(
            "Conditionals require a bool value found, {value}"
        ))),
    }
}

fn run_lambda<T: HostType + Display>(
    name: String,
    input: Option<String>,
    output: Option<String>,
    lambda_body: Element,
    expressions: &mut Vec<Expression>,
    environment: &mut Environment<T>,
) -> Option<Result<WanderValue<T>, WanderError>> {
    if expressions.is_empty() {
        Some(Ok(WanderValue::Lambda(
            name,
            input,
            output,
            Box::new(lambda_body),
        )))
    } else {
        let argument_expression = expressions.pop().unwrap();
        let argument_value = match eval(&argument_expression, environment) {
            Err(e) => return Some(Err(e)),
            Ok(e) => e,
        };
        environment.bind(name, argument_value);
        let expression = match express(&lambda_body) {
            Ok(e) => e,
            Err(e) => return Some(Err(e)),
        };
        let function = match eval(&expression, environment) {
            Ok(e) => e,
            Err(err) => return Some(Err(err)),
        };
        match function {
            WanderValue::Lambda(_, _, _, b) => {
                let Ok(expression) = express(&b) else {
                    return None;
                };
                match eval(&expression, environment) {
                    Ok(value) => {
                        expressions.push(value_to_expression(value));
                        None
                    }
                    Err(err) => Some(Err(err)),
                }
            }
            _ => {
                if expressions.is_empty() {
                    Some(Ok(function))
                } else {
                    Some(Err(WanderError(format!(
                        "Invalid function call, expected expressions {expressions:?}."
                    ))))
                }
            }
        }
    }
}

fn handle_function_call<T: HostType>(
    expressions: &Vec<Expression>,
    environment: &mut Environment<T>,
) -> Result<WanderValue<T>, WanderError> {
    if expressions.len() == 1 {
        let expression = expressions.first().unwrap();
        return eval(expression, environment);
    }
    let mut expressions = expressions.clone();
    expressions.reverse();
    while let Some(expression) = expressions.pop() {
        match expression {
            Expression::Application(contents) => {
                match handle_function_call(&contents, environment)? {
                    WanderValue::Lambda(name, input, output, element) => {
                        if let Some(res) =
                            run_lambda(name, input, output, *element, &mut expressions, environment)
                        {
                            return res;
                        }
                    }
                    e => return Ok(e),
                }
            }
            Expression::Lambda(name, input, output, lambda_body) => {
                if let Some(res) = run_lambda(
                    name,
                    input,
                    output,
                    *lambda_body,
                    &mut expressions,
                    environment,
                ) {
                    return res;
                }
            }
            Expression::Name(name) => match eval(&Expression::Name(name), environment) {
                Ok(value) => match value {
                    WanderValue::Lambda(p, i, o, b) => {
                        let argument_expression = expressions.pop().unwrap();
                        let argument_value = eval(&argument_expression, environment)?;
                        environment.bind(p, argument_value);
                        match eval(&express(&b)?, environment) {
                            Ok(value) => expressions.push(value_to_expression(value)),
                            Err(err) => return Err(err),
                        }
                    }
                    _ => {
                        return Err(WanderError(format!(
                            "Invalid function call, was expecting a lambda and found {value}."
                        )))
                    }
                },
                Err(err) => return Err(err),
            },
            value => {
                if expressions.is_empty() {
                    return eval(&value, environment);
                } else {
                    return Err(WanderError(format!("Invalid function call {value:?}.")));
                }
            }
        };
    }
    panic!()
}

fn value_to_expression<T: Clone + Display + PartialEq + Eq>(value: WanderValue<T>) -> Expression {
    match value {
        WanderValue::Bool(value) => Expression::Boolean(value),
        WanderValue::Int(value) => Expression::Int(value),
        WanderValue::String(value) => Expression::String(value),
        WanderValue::Identifier(value) => Expression::Identifier(value),
        WanderValue::Nothing => Expression::Nothing,
        WanderValue::Lambda(p, i, o, b) => Expression::Lambda(p, i, o, b),
        WanderValue::List(values) => {
            let mut expressions = vec![];
            for value in values {
                expressions.push(value_to_expression(value).clone());
            }
            Expression::List(expressions)
        }
        WanderValue::Tuple(values) => {
            let mut expressions = vec![];
            for value in values {
                expressions.push(value_to_expression(value).clone());
            }
            Expression::Tuple(expressions)
        }
        WanderValue::Set(values) => {
            let mut expressions = HashSet::new();
            for value in values {
                expressions.insert(value_to_expression(value).clone());
            }
            Expression::Set(expressions)
        }
        WanderValue::Record(value_record) => {
            let mut record = HashMap::new();
            for (name, value) in value_record {
                record.insert(name, value_to_expression(value));
            }
            Expression::Record(record)
        }
        WanderValue::HostValue(value) => todo!(),
    }
}

fn handle_let<T: HostType + Display>(
    decls: Vec<(String, Option<Expression>, Expression)>,
    body: Expression,
    environment: &mut Environment<T>,
) -> Result<WanderValue<T>, WanderError> {
    for (name, tag, body) in decls {
        handle_decl(name, tag, body, environment)?;
    }
    eval(&body, environment)
}

fn handle_decl<T: HostType + Display>(
    name: String,
    tag: Option<Expression>,
    body: Expression,
    environment: &mut Environment<T>,
) -> Result<(), WanderError> {
    //TODO handle tag checking here
    match eval(&body, environment) {
        Ok(value) => {
            environment.bind(name.to_string(), value);
            Ok(())
        }
        Err(err) => Err(err),
    }
}

fn read_name<T: HostType>(
    name: &String,
    environment: &mut Environment<T>,
) -> Result<WanderValue<T>, WanderError> {
    if let Some(value) = environment.read(name) {
        Ok(value)
    } else {
        match environment.read_host_function(name) {
            Some(_) => todo!(), //Ok(WanderValue::HostedFunction(name.to_owned())),
            None => read_field(name, environment),
        }
    }
}

fn read_tagged_name<T: HostType>(
    name: &String,
    tag: &Expression,
    environment: &mut Environment<T>,
) -> Result<WanderValue<T>, WanderError> {
    if let Some(value) = environment.read(name) {
        Ok(value)
    } else {
        match environment.read_host_function(name) {
            Some(_) => todo!(), //Ok(WanderValue::HostedFunction(name.to_owned())),
            None => read_field(name, environment),
        }
    }
}

fn read_field<T: HostType>(
    name: &str,
    environment: &mut Environment<T>,
) -> Result<WanderValue<T>, WanderError> {
    let t = name
        .split('.')
        .map(|e| e.to_string())
        .collect::<Vec<String>>();
    let mut result = None;
    let (name, fields) = t.split_first().unwrap();
    if let Some(WanderValue::Record(value)) = environment.read(&name.to_string()) {
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

fn call_function<T: HostType + Display>(
    name: &String,
    arguments: &Vec<Expression>,
    environment: &mut Environment<T>,
) -> Result<WanderValue<T>, WanderError> {
    let mut argument_values = vec![];
    for argument in arguments {
        match eval(argument, environment) {
            Ok(value) => argument_values.push(value),
            Err(err) => return Err(err),
        }
    }
    match environment.read(name) {
        //found other value (err), will evntually handle lambdas here
        Some(_) => Err(WanderError(format!("Function {} is not defined.", &name))),
        None => match environment.read_host_function(name) {
            None => Err(WanderError(format!("Function {} is not defined.", name))),
            Some(function) => {
                if argument_values.len() == function.binding().parameters.len() {
                    function.run(&argument_values, environment)
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
