// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::collections::HashMap;

use crate::{interpreter::Expression, parser::Element, WanderError};

// Handle any tranlations needed before creating an expression.
pub fn translate(element: Element) -> Result<Expression, WanderError> {
    let element = process_forwards(&element)?;
    express(&element)
}

// /// Handle any tranlations needed before creating expressions.
// pub fn translate_all(elements: Vec<Element>) -> Result<Vec<Expression>, WanderError> {
//     let element = process_forwards(&Element::Grouping(elements))?;
//     express(element)
// }

// fn process_all_forwards(elements: Vec<Element>) -> Result<Vec<Element>, WanderError> {
//     println!("start of process_all_forwards {elements:?}");
//     let mut res = vec![];
//     for element in elements {
//         match element {
//             Element::Grouping(_) => res.push(process_forwards(&element)?),
//             Element::Forward => res.insert(index, element),
//             e => res.push(e),
//         }
//     }
//     Ok(res)
// }

fn process_forwards(element: &Element) -> Result<Element, WanderError> {
    println!("start of process_forwards, {element:?}");
    let elements = match element {
        Element::Grouping(elements) => elements,
        e => return Ok(e.clone()),
    };
    let mut index = 0;
    let mut results: Vec<Element> = vec![];
    let mut to_move: Vec<Element> = vec![];
    while let Some(element) = elements.get(index) {
        if element == &Element::Forward {
            if to_move.is_empty() {
                to_move.append(&mut results);
            } else {
                if to_move.len() == 1 {
                    results.push(to_move.pop().unwrap());
                } else {
                    let mut g = vec![];
                    g.append(&mut to_move);
                    results.push(Element::Grouping(g));
                }
            }
        } else {
            results.push(element.clone());
        }
        index += 1;
    }
    if !to_move.is_empty() {
        if to_move.len() == 1 {
            results.push(to_move.first().unwrap().clone());
        } else {
            let mut g = vec![];
            g.append(&mut to_move);
            results.push(Element::Grouping(g));
        }
    }
    return Ok(Element::Grouping(results));
}

pub fn express(element: &Element) -> Result<Expression, WanderError> {
    let expression = match element {
        Element::Boolean(val) => Expression::Boolean(*val),
        Element::Int(val) => Expression::Int(*val),
        Element::String(val) => Expression::String(val.clone()),
        Element::Name(name) => Expression::Name(name.clone()),
        Element::Let(decls, body) => Expression::Let(
            decls
                .clone()
                .iter()
                .map(|e| (e.0.clone(), express(&e.1).unwrap()))
                .collect(),
            Box::new(express(body).unwrap()),
        ),
        Element::Grouping(elements) => return handle_grouping(elements),
        Element::Conditional(i, ie, ee) => Expression::Conditional(
            Box::new(express(i).unwrap()),
            Box::new(express(ie).unwrap()),
            Box::new(express(ee).unwrap()),
        ),
        Element::Lambda(p, i, o, b) => {
            Expression::Lambda(p.clone(), i.clone(), o.clone(), b.clone())
        }
        Element::Tuple(values) => {
            Expression::Tuple(values.clone().iter().map(|e| express(e).unwrap()).collect())
        }
        Element::List(values) => {
            Expression::List(values.clone().iter().map(|e| express(e).unwrap()).collect())
        }
        Element::Set(values) => {
            Expression::Set(values.clone().iter().map(|e| express(e).unwrap()).collect())
        }
        Element::Record(values) => {
            let mut result: HashMap<String, Expression> = HashMap::new();
            values
                .iter()
                .map(|e| (e.0, express(e.1).unwrap()))
                .for_each(|e| {
                    result.insert(e.0.clone(), e.1);
                });
            Expression::Record(result)
        }
        Element::Nothing => Expression::Nothing,
        Element::Forward => {
            return Err(WanderError(
                "Cannot process pipe, Should never reach.".to_owned(),
            ))
        }
        Element::HostFunction(name) => Expression::HostFunction(name.clone()),
    };
    Ok(expression)
}

fn handle_grouping(elements: &Vec<Element>) -> Result<Expression, WanderError> {
    println!("in handle_grouping {elements:?}");
    let expressions: Vec<Expression> = elements
        .clone()
        .iter()
        .map(|e| express(e).unwrap())
        .collect();
    let expressions: Vec<Expression> = expressions
        .iter()
        .map(|e| match e {
            Expression::Application(application) => {
                if application.len() == 1 {
                    application.first().unwrap().clone()
                } else {
                    e.clone()
                }
            }
            e => e.clone(),
        })
        .collect();
    if expressions.len() == 1 {
        println!("ret 1 - {:?}", expressions.first().unwrap().clone());
        Ok(expressions.first().unwrap().clone())
    } else {
        let res = Expression::Application(expressions);
        println!("ret 2 - {res:?}");
        Ok(res)
    }
}
