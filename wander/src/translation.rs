// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::collections::HashMap;

use crate::{interpreter::Expression, parser::Element, WanderError};

/// Handle any tranlations needed before creating an expression.
pub fn translate(element: &Element) -> Result<Expression, WanderError> {
    let res = process_forwards(element)?;
    express(&res)
}

/// Handle any tranlations needed before creating expressions.
pub fn translate_all(elements: Vec<Element>) -> Result<Vec<Expression>, WanderError> {
    let elements = process_all_forwards(elements)?;
    express_all(elements)
}

fn process_all_forwards(elements: Vec<Element>) -> Result<Vec<Element>, WanderError> {
    let mut res = vec![];
    for element in elements {
        match element {
            Element::Grouping(_) => res.push(process_forwards(&element)?),
            e => res.push(e),
        }
    }
    Ok(res)
}

fn process_forwards(element: &Element) -> Result<Element, WanderError> {
    let elements = match element {
        Element::Grouping(elements) => elements,
        e => return Ok(e.clone()),
    };
    let mut index = 0;
    let mut results: Vec<Element> = vec![];
    let mut to_move: Vec<Element> = vec![];
    while let Some(element) = elements.get(index) {
        if element == &Element::Forward {
            to_move.append(&mut results);
        } else {
            results.push(element.clone());
        }
        index += 1;
    }
    if !to_move.is_empty() {
        results.append(&mut to_move);
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
    Ok(Expression::Application(
        elements
            .clone()
            .iter()
            .map(|e| express(e).unwrap())
            .collect(),
    ))
}

fn express_all(elements: Vec<Element>) -> Result<Vec<Expression>, WanderError> {
    let elements = elements.iter().map(|e| express(e).unwrap()).collect();
    Ok(elements)
}
