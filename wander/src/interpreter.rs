// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::fmt::Display;

use crate::bindings::Bindings;
use crate::parser::Element;
use crate::{WanderError, WanderType, WanderValue};

// #[doc(hidden)]
// #[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
// pub enum Expression {
//     Boolean(bool),
//     Int(i64),
//     String(String),
//     Name(String),
//     HostFunction(String), //refers to HostFunctions by name
//     Let(Vec<(String, Element)>, Box<Element>),
//     FunctionCall(Vec<Element>),
//     Conditional(Box<Element>, Box<Element>, Box<Element>),
//     Lambda(String, WanderType, WanderType, Box<Element>),
//     Tuple(Vec<Element>),
//     List(Vec<Element>),
//     Set(HashSet<Element>),
//     Record(HashMap<String, Element>),
//     Nothing,
//     Forward,
// }

pub fn eval<T: Clone + Display + PartialEq + Eq>(
    script: &Vec<Element>,
    bindings: &mut Bindings<T>,
) -> Result<WanderValue<T>, WanderError> {
    let mut result = WanderValue::Nothing;
    for element in script {
        match result {
            WanderValue::Lambda(ref name, _input, _output, ref body) => {
                let argument = eval_element(element, bindings)?;
                bindings.bind(name.clone(), argument);
                result = eval_element(body, bindings)?;
            }

            _ => result = eval_element(element, bindings)?,
        }
    }
    Ok(result)
}

pub fn eval_element<T: Clone + Display + PartialEq + Eq>(
    element: &Element,
    bindings: &mut Bindings<T>,
) -> Result<WanderValue<T>, WanderError> {
    match element {
        Element::Boolean(value) => Ok(WanderValue::Boolean(*value)),
        Element::Int(value) => Ok(WanderValue::Int(*value)),
        Element::String(value) => Ok(WanderValue::String(unescape_string(value.to_string()))),
        Element::Let(decls, body) => handle_let(decls.clone(), *body.clone(), bindings),
        Element::Name(name) => read_name(name, bindings),
        Element::FunctionCall(expressions) => handle_function_call(expressions, bindings),
        Element::Conditional(c, i, e) => handle_conditional(c, i, e, bindings),
        Element::List(values) => handle_list(values, bindings),
        Element::Nothing => Ok(WanderValue::Nothing),
        Element::Forward => panic!("Should never reach."),
        Element::Tuple(values) => handle_tuple(values, bindings),
        Element::Record(values) => handle_record(values, bindings),
        Element::Lambda(name, input, output, body) => handle_lambda(name, input, output, body),
        Element::Set(values) => handle_set(values, bindings),
        Element::HostFunction(name) => handle_host_function(name, bindings),
        Element::Grouping(element) => eval_element(element, bindings),
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

fn handle_set<T: Clone + Display + PartialEq + Eq>(
    elements: &HashSet<Element>,
    bindings: &mut Bindings<T>,
) -> Result<WanderValue<T>, WanderError> {
    let mut results = HashSet::new();
    for element in elements {
        match eval_element(element, bindings) {
            Ok(value) => results.insert(value),
            Err(err) => return Err(err),
        };
    }
    Ok(WanderValue::Set(results))
}

fn handle_tuple<T: Clone + Display + PartialEq + Eq>(
    elements: &Vec<Element>,
    bindings: &mut Bindings<T>,
) -> Result<WanderValue<T>, WanderError> {
    let mut results = vec![];
    for element in elements {
        match eval_element(element, bindings) {
            Ok(value) => results.push(value),
            Err(err) => return Err(err),
        }
    }
    Ok(WanderValue::Tuple(results))
}

fn handle_record<T: Clone + Display + PartialEq + Eq>(
    elements: &HashMap<String, Element>,
    bindings: &mut Bindings<T>,
) -> Result<WanderValue<T>, WanderError> {
    let mut results = HashMap::new();
    for (name, element) in elements {
        match eval_element(element, bindings) {
            Ok(value) => results.insert(name.to_owned(), value),
            Err(err) => return Err(err),
        };
    }
    Ok(WanderValue::Record(results))
}

fn handle_list<T: Clone + Display + PartialEq + Eq>(
    elements: &Vec<Element>,
    bindings: &mut Bindings<T>,
) -> Result<WanderValue<T>, WanderError> {
    let mut results = vec![];
    for element in elements {
        match eval_element(element, bindings) {
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

fn handle_conditional<T: Clone + Display + PartialEq + Eq>(
    cond: &Element,
    ife: &Element,
    elsee: &Element,
    bindings: &mut Bindings<T>,
) -> Result<WanderValue<T>, WanderError> {
    match eval_element(cond, bindings)? {
        WanderValue::Boolean(true) => eval_element(ife, bindings),
        WanderValue::Boolean(false) => eval_element(elsee, bindings),
        value => Err(WanderError(format!(
            "Conditionals require a bool value found, {value}"
        ))),
    }
}

fn handle_function_call<T: Clone + Display + PartialEq + Eq>(
    expressions: &Vec<Element>,
    bindings: &mut Bindings<T>,
) -> Result<WanderValue<T>, WanderError> {
    let mut expressions = expressions.clone();
    expressions.reverse();
    while !expressions.is_empty() {
        let expression = expressions.pop().unwrap();
        match eval_element(&expression, bindings)? {
            WanderValue::Lambda(name, input, output, body) => {
                if expressions.is_empty() {
                    return Ok(WanderValue::Lambda(name, input, output, body))
                } else {
                    let expression = expressions.pop().unwrap();
                    let res = eval_element(&expression, bindings)?;
                    bindings.bind(name, res);
                    match eval_element(&body, bindings) {
                        Ok(value) => expressions.push(value_to_element(value)),
                        Err(err) => return Err(err),
                    }                    
                }
            },
            value => {
                if expressions.is_empty() {
                    return Ok(value)
                } else {
                    return Err(WanderError("Invalid function call.".to_owned()))
                }
            }
        };
    }
    panic!()
}

fn value_to_element<T: Clone + Display + PartialEq + Eq>(value: WanderValue<T>) -> Element {
    match value {
        WanderValue::Boolean(value) => Element::Boolean(value),
        WanderValue::Int(value) => Element::Int(value),
        WanderValue::String(value) => Element::String(value),
        WanderValue::Nothing => Element::Nothing,
        WanderValue::DeprecatedLambda(_, _) => todo!(),
        WanderValue::Lambda(p, i, o, b) => Element::Lambda(p, i, o, b),
        WanderValue::List(value) => todo!(),
        WanderValue::Tuple(value) => todo!(),
        WanderValue::Set(value) => todo!(),
        WanderValue::Record(value) => todo!(),
        WanderValue::HostValue(value) => todo!(),
    }
}

fn handle_let<T: Clone + Display + PartialEq + Eq>(
    decls: Vec<(String, Element)>,
    body: Element,
    bindings: &mut Bindings<T>,
) -> Result<WanderValue<T>, WanderError> {

    for (name, body) in decls {
        handle_decl(name, body, bindings);
    }
    eval_element(&body, bindings)
}

fn handle_decl<T: Clone + Display + PartialEq + Eq>(
    name: String,
    body: Element,
    bindings: &mut Bindings<T>,
) -> Result<(), WanderError> {
    match eval_element(&body, bindings) {
        Ok(value) => {
            bindings.bind(name.to_string(), value);
            Ok(())        
        },
        Err(err) => return Err(err),
    }
}

fn read_name<T: Clone + PartialEq + Display + Eq>(
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

fn read_field<T: Clone + PartialEq + Display + Eq>(
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

fn call_function<T: Clone + Display + PartialEq + Eq>(
    name: &String,
    arguments: &Vec<Element>,
    bindings: &mut Bindings<T>,
) -> Result<WanderValue<T>, WanderError> {
    let mut argument_values = vec![];
    for argument in arguments {
        match eval_element(argument, bindings) {
            Ok(value) => argument_values.push(value),
            Err(err) => return Err(err),
        }
    }
    match bindings.read(name) {
        Some(WanderValue::DeprecatedLambda(parameters, body)) => {
            match parameters.len().cmp(&arguments.len()) {
                Ordering::Equal => {
                    bindings.add_scope();
                    for (i, parameter) in parameters.iter().enumerate() {
                        bindings.bind(
                            parameter.to_owned(),
                            argument_values.get(i).unwrap().clone(),
                        );
                    }
                    let res = eval(&body, bindings);
                    bindings.remove_scope();
                    res
                }
                Ordering::Less => Err(WanderError(format!(
                    "Incorrect number of arguments, {}, passed to {}, expecting {}.",
                    arguments.len(),
                    name,
                    parameters.len()
                ))),
                Ordering::Greater => todo!(), //Ok(WanderValue::PartialApplication(Box::new(
                    // PartialApplication {
                    //     arguments: argument_values,
                    //     callee: WanderValue::DeprecatedLambda(parameters, body),
                    // },
                //))),
            }
        }
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
