// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::collections::HashMap;

use crate::{interpreter::Expression, parser::Element, WanderError, Location};

// Handle any tranlations needed before creating an expression.
pub fn translate(elements: Vec<Location<Element>>) -> Result<Vec<Location<Expression>>, WanderError> {
    let mut results = vec![];
    for Location(element, source) in elements {
        let element = process_pipes(&element)?;
        let expression = express(&element)?;
        results.push(Location(expression, source));
    }
    Ok(results)
}

fn process_pipes(element: &Element) -> Result<Element, WanderError> {
    let elements = match element {
        Element::Grouping(elements) => elements,
        e => return Ok(e.clone()),
    };
    let mut index = 0;
    let mut results: Vec<Element> = vec![];
    while let Some(element) = elements.get(index) {
        if element == &Element::Pipe {
            index += 1;
            match elements.get(index) {
                Some(Element::Grouping(next_elements)) => {
                    let mut next_elements = next_elements.clone();
                    let mut new_results = vec![];
                    next_elements.append(&mut results);
                    new_results.push(Element::Grouping(next_elements.clone()));
                    results = new_results;
                }
                _ => return Err(WanderError("Invalid pipe.".to_owned())),
            }
        } else {
            results.push(element.clone());
        }
        index += 1;
    }
    Ok(Element::Grouping(results))
}

fn express_optional_name(name: &Option<String>) -> Result<Option<Expression>, WanderError> {
    match name {
        Some(element) => Ok(Some(express(&Element::Name(element.to_string()))?)),
        None => Ok(None),
    }
}

pub fn express(element: &Element) -> Result<Expression, WanderError> {
    let expression = match element {
        Element::Boolean(val) => Expression::Boolean(*val),
        Element::Int(val) => Expression::Int(*val),
        Element::String(val) => Expression::String(val.clone()),
        Element::Identifier(value) => Expression::Identifier(value.clone()),
        Element::Name(name) => Expression::Name(name.clone()),
        Element::Let(decls, body) => Expression::Let(
            decls
                .clone()
                .iter()
                .map(|e| {
                    (
                        e.0.clone(),
                        express_optional_name(&e.1).unwrap(),
                        express(&e.2).unwrap(),
                    )
                })
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
        Element::Pipe => {
            return Err(WanderError(
                "Cannot process pipe, Should never reach.".to_owned(),
            ))
        }
        Element::HostFunction(name) => Expression::HostFunction(name.clone()),
        Element::TaggedName(name, tag) => {
            Expression::TaggedName(name.clone(), Box::new(express(tag).unwrap()))
        }
    };
    Ok(expression)
}

fn handle_grouping(elements: &[Element]) -> Result<Expression, WanderError> {
    let expressions: Vec<Expression> = elements.iter().map(|e| express(e).unwrap()).collect();
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
        Ok(expressions.first().unwrap().clone())
    } else {
        let res = Expression::Application(expressions);
        Ok(res)
    }
}
