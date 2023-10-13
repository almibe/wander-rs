// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::collections::HashMap;

use crate::{parser::Element, WanderError, interpreter::Expression, bindings};

/// Handle any tranlations needed before creating an expression.
pub fn translate(elements: Vec<Element>) -> Result<Expression, WanderError> {
    let elements = process_forwards(elements)?;
    if elements.len() != 1 {
        Err(WanderError("Invalid script. Only a single top level expression allowed.".to_owned()))
    } else {
        express(elements.first().unwrap())
    }
}

/// Handle any tranlations needed before creating expressions.
pub fn translate_all(elements: Vec<Element>) -> Result<Vec<Expression>, WanderError> {
    let elements = process_forwards(elements)?;
    express_all(elements)
}

fn process_forwards(elements: Vec<Element>) -> Result<Vec<Element>, WanderError> {
    Ok(elements)
    // let mut index = 0;
    // let mut results: Vec<Element> = vec![];
    // while let Some(element) = elements.get(index) {
    //     if element == &Element::Forward {
    //         let prev = results.pop().unwrap(); //elements.get(index - 1).unwrap();
    //         index += 1;
    //         if let Some(Element::FunctionCall(name, arguments)) = elements.get(index) {
    //             let mut arguments = arguments.clone();
    //             arguments.push(prev.clone());
    //             results.push(Element::FunctionCall(name.to_owned(), arguments));
    //         } else {
    //             return Err(WanderError("Error handling forward operator.".to_owned()));
    //         }
    //     } else {
    //         results.push(element.clone());
    //     }
    //     index += 1;
    // }
    // Ok(results)
}

pub fn express(element: &Element) -> Result<Expression, WanderError> {
    let expression = match element {
        Element::Boolean(val) => Expression::Boolean(*val),
        Element::Int(val) => Expression::Int(*val),
        Element::String(val) => Expression::String(val.clone()),
        Element::Name(name) => Expression::Name(name.clone()),
        Element::Let(decls, body) => Expression::Let(decls.clone().iter().map(|e| (e.0.clone(), express(&e.1).unwrap())).collect(), Box::new(express(body).unwrap())),
        Element::Application(call) => Expression::Application(call.clone().iter().map(|e| express(e).unwrap()).collect()),
        Element::Conditional(i, ie, ee) => Expression::Conditional(Box::new(express(i).unwrap()), Box::new(express(ie).unwrap()), Box::new(express(ee).unwrap())),
        Element::Lambda(p, i, o, b) => Expression::Lambda(p.clone(), i.clone(), o.clone(), b.clone()),
        Element::Tuple(values) => Expression::Tuple(values.clone().iter().map(|e| express(e).unwrap()).collect()),
        Element::List(values) => Expression::List(values.clone().iter().map(|e| express(e).unwrap()).collect()),
        Element::Set(values) => Expression::Set(values.clone().iter().map(|e| express(e).unwrap()).collect()),
        Element::Record(values) => {
            let mut result: HashMap<String, Expression> = HashMap::new();
            values.iter().map(|e| (e.0, express(e.1).unwrap())).for_each(|e| {
                result.insert(e.0.clone(), e.1);
            }
            );
            Expression::Record(result)
        },
        Element::Nothing => Expression::Nothing,
        Element::Forward => panic!("Should never reach."),
        Element::HostFunction(name) => Expression::HostFunction(name.clone()),
    };
    Ok(expression)
}

fn express_all(elements: Vec<Element>) -> Result<Vec<Expression>, WanderError> {
    let elements = elements.iter().map(|e| express(e).unwrap()).collect();
    Ok(elements)
}
