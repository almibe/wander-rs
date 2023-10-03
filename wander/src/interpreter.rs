// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::fmt::Display;

use crate::bindings::Bindings;
use crate::parser::Element;
use crate::{PartialApplication, WanderError, WanderType, WanderValue};

pub fn eval<T: Clone + Display + PartialEq + Eq>(
    script: &Vec<Element>,
    bindings: &mut Bindings<T>,
) -> Result<WanderValue<T>, WanderError> {
    let mut result = Ok(WanderValue::Nothing);
    for element in script {
        result = Ok(eval_element(element, bindings)?);
    }
    result
}

pub fn eval_element<T: Clone + Display + PartialEq + Eq>(
    element: &Element,
    bindings: &mut Bindings<T>,
) -> Result<WanderValue<T>, WanderError> {
    match element {
        Element::Boolean(value) => Ok(WanderValue::Boolean(*value)),
        Element::Int(value) => Ok(WanderValue::Int(*value)),
        Element::String(value) => Ok(WanderValue::String(unescape_string(value.to_string()))),
        Element::Val(name, value) => handle_let(name, value, bindings),
        Element::Name(name) => read_name(name, bindings),
        Element::FunctionCall(name, arguments) => call_function(name, arguments, bindings),
        Element::Scope(body) => handle_scope(body, bindings),
        Element::Conditional(c, i, e) => handle_conditional(c, i, e, bindings),
        Element::DeprecatedLambda(params, body) => deprecated_handle_lambda(params, body),
        Element::List(values) => handle_list(values, bindings),
        Element::Nothing => Ok(WanderValue::Nothing),
        Element::Forward => panic!("Should never reach."),
        Element::Tuple(values) => handle_tuple(values, bindings),
        Element::Record(values) => handle_record(values, bindings),
        Element::Lambda(name, input, output, body) => handle_lambda(name, input, output, body),
        Element::Set(values) => handle_set(values, bindings),
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

fn deprecated_handle_lambda<T: Clone + PartialEq + Eq>(
    params: &Vec<String>,
    body: &Vec<Element>,
) -> Result<WanderValue<T>, WanderError> {
    Ok(WanderValue::DeprecatedLambda(
        params.to_owned(),
        body.to_owned(),
    ))
}

fn handle_lambda<T: Clone + PartialEq + Eq>(
    name: &String,
    input: &WanderType,
    output: &WanderType,
    body: &Box<Element>,
) -> Result<WanderValue<T>, WanderError> {
    Ok(WanderValue::Lambda(
        name.clone(),
        input.clone(),
        output.clone(),
        body.clone(),
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

fn handle_scope<T: Clone + Display + PartialEq + Eq>(
    body: &Vec<Element>,
    bindings: &mut Bindings<T>,
) -> Result<WanderValue<T>, WanderError> {
    bindings.add_scope();
    let res = eval(body, bindings);
    bindings.remove_scope();
    res
}

fn handle_let<T: Clone + Display + PartialEq + Eq>(
    name: &String,
    element: &Element,
    bindings: &mut Bindings<T>,
) -> Result<WanderValue<T>, WanderError> {
    match eval_element(element, bindings) {
        Ok(value) => {
            bindings.bind(name.to_string(), value);
            Ok(WanderValue::Nothing)
        }
        Err(err) => Err(err),
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
            Some(_) => Ok(WanderValue::HostedFunction(name.to_owned())),
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
        //corner case of this name shadowing with a native function
        Some(WanderValue::HostedFunction(function_name)) => {
            match bindings.read_host_function(&function_name) {
                Some(function) => function.run(&argument_values, bindings),
                None => Err(WanderError(
                    "Could not read function {name} that references HostedFunction {nf_name}"
                        .to_owned(),
                )),
            }
        }
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
                Ordering::Greater => Ok(WanderValue::PartialApplication(Box::new(
                    PartialApplication {
                        arguments: argument_values,
                        callee: WanderValue::DeprecatedLambda(parameters, body),
                    },
                ))),
            }
        }
        Some(WanderValue::PartialApplication(application)) => match application.callee {
            WanderValue::HostedFunction(function_name) => {
                let mut args = application.arguments.clone();
                args.append(&mut argument_values.clone());
                match bindings.read_host_function(&function_name) {
                    None => Err(WanderError(format!("Function {} is not defined.", name))),
                    Some(function) => {
                        if args.len() == function.binding().parameters.len() {
                            function.run(&args, bindings)
                        } else {
                            Ok(WanderValue::PartialApplication(Box::new(
                                PartialApplication {
                                    arguments: args,
                                    callee: WanderValue::HostedFunction(function_name.clone()),
                                },
                            )))
                        }
                    }
                }
            }
            WanderValue::DeprecatedLambda(parameters, body) => {
                let mut args = application.arguments.clone();
                args.append(&mut argument_values.clone());
                if parameters.len() == args.len() {
                    bindings.add_scope();
                    for (i, parameter) in parameters.iter().enumerate() {
                        bindings.bind(parameter.to_owned(), args.get(i).unwrap().clone());
                    }
                    let res = eval(&body, bindings);
                    bindings.remove_scope();
                    res
                } else {
                    Ok(WanderValue::PartialApplication(Box::new(
                        PartialApplication {
                            arguments: args,
                            callee: WanderValue::DeprecatedLambda(parameters, body),
                        },
                    )))
                }
            }
            _ => panic!(
                "Should never reach. Callee in an Application was not a HostFunction or Lambda."
            ),
        },
        //found other value (err), will evntually handle lambdas here
        Some(_) => Err(WanderError(format!("Function {} is not defined.", &name))),
        None => match bindings.read_host_function(name) {
            None => Err(WanderError(format!("Function {} is not defined.", name))),
            Some(function) => {
                if argument_values.len() == function.binding().parameters.len() {
                    function.run(&argument_values, bindings)
                } else {
                    Ok(WanderValue::PartialApplication(Box::new(
                        PartialApplication {
                            arguments: argument_values,
                            callee: WanderValue::HostedFunction(name.clone()),
                        },
                    )))
                }
            }
        },
    }
}
