// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::{lexer::Token, WanderError};
use gaze::Gaze;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[doc(hidden)]
#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
pub enum Element {
    Boolean(bool),
    Int(i64),
    String(String),
    Name(String),
    TaggedName(String, Box<Element>),
    HostFunction(String),
    Let(Vec<(String, Option<String>, Element)>, Box<Element>),
    Grouping(Vec<Element>),
    Conditional(Box<Element>, Box<Element>, Box<Element>),
    Lambda(String, Option<String>, Option<String>, Box<Element>),
    Tuple(Vec<Element>),
    List(Vec<Element>),
    Set(HashSet<Element>),
    Record(HashMap<String, Element>),
    Nothing,
    Pipe,
}

impl core::hash::Hash for Element {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);
    }
}

fn boolean(gaze: &mut Gaze<Token>) -> Option<Element> {
    match gaze.next() {
        Some(Token::Boolean(value)) => Some(Element::Boolean(value)),
        _ => None,
    }
}

fn int(gaze: &mut Gaze<Token>) -> Option<Element> {
    match gaze.next() {
        Some(Token::Int(value)) => Some(Element::Int(value)),
        _ => None,
    }
}

fn string(gaze: &mut Gaze<Token>) -> Option<Element> {
    match gaze.next() {
        Some(Token::String(value)) => Some(Element::String(value)),
        _ => None,
    }
}

fn nothing(gaze: &mut Gaze<Token>) -> Option<Element> {
    match gaze.next() {
        Some(Token::Nothing) | Some(Token::QuestionMark) => Some(Element::Nothing),
        _ => None,
    }
}

fn pipe(gaze: &mut Gaze<Token>) -> Option<Element> {
    match gaze.next() {
        Some(Token::Pipe) => Some(Element::Pipe),
        _ => None,
    }
}

fn name(gaze: &mut Gaze<Token>) -> Option<Element> {
    match gaze.next() {
        Some(Token::Name(value)) => Some(Element::Name(value)),
        _ => None,
    }
}

fn let_scope(gaze: &mut Gaze<Token>) -> Option<Element> {
    match gaze.next() {
        Some(Token::Let) => (),
        _ => return None,
    }

    let mut decls = vec![];
    while let Some(element) = gaze.attemptf(&mut val_binding) {
        decls.push(element);
    }

    match gaze.next() {
        Some(Token::In) => {
            let body = if let Some(element) = gaze.attemptf(&mut element) {
                element
            } else {
                Element::Nothing
            };

            match gaze.next() {
                Some(Token::End) => Some(Element::Let(decls, Box::new(body))),
                _ => None,
            }
        }
        _ => Some(Element::Let(decls, Box::new(Element::Nothing))),
    }
}

fn grouping(gaze: &mut Gaze<Token>) -> Option<Element> {
    let mut expressions: Vec<Element> = vec![];

    while let Some(e) = gaze.attemptf(&mut element_inner) {
        expressions.push(e);
    }

    match &expressions[..] {
        [] => None,
        _ => Some(Element::Grouping(expressions)),
    }
}

fn grouped_application(gaze: &mut Gaze<Token>) -> Option<Element> {
    let mut expressions: Vec<Element> = vec![];

    match gaze.next() {
        Some(Token::OpenParen) => (),
        _ => return None,
    }

    while let Some(e) = gaze.attemptf(&mut element_inner) {
        expressions.push(e);
    }

    match gaze.next() {
        Some(Token::CloseParen) => Some(Element::Grouping(expressions)),
        _ => None,
    }
}

fn conditional(gaze: &mut Gaze<Token>) -> Option<Element> {
    match gaze.next() {
        Some(Token::If) => (),
        _ => return None,
    }
    let cond = match gaze.attemptf(&mut element) {
        Some(d) => d,
        None => return None,
    };

    match gaze.next() {
        Some(Token::Then) => (),
        _ => return None,
    }

    let ife = match gaze.attemptf(&mut element) {
        Some(d) => d,
        None => return None,
    };
    if let Some(Token::Else) = gaze.next() {
        //do nothing
    } else {
        return None;
    }
    let elsee = match gaze.attemptf(&mut element) {
        Some(d) => d,
        None => return None,
    };
    if let Some(Token::End) = gaze.next() {
        //do nothing
    } else {
        return None;
    }
    Some(Element::Conditional(
        Box::new(cond),
        Box::new(ife),
        Box::new(elsee),
    ))
}

fn lambda(gaze: &mut Gaze<Token>) -> Option<Element> {
    match gaze.next() {
        Some(Token::Lambda) => (),
        _ => return None,
    }

    let mut params: Vec<(String, Option<String>)> = vec![];

    while let Some(Element::Name(name)) = gaze.attemptf(&mut name) {
        let tag = if gaze.peek() == Some(Token::Colon) {
            gaze.next();
            match gaze.next() {
                Some(Token::Name(name)) => Some(name),
                _ => return None, //no match
            }
        } else {
            None
        };
        params.push((name, tag));
    }

    match gaze.next() {
        Some(Token::Arrow) => (),
        _ => return None,
    }

    gaze.attemptf(&mut element).map(|body| {
        let mut final_lambda = None;
        params.reverse();
        for (name, tag) in params {
            match final_lambda {
                Some(prev_lambda) => {
                    final_lambda = Some(Element::Lambda(
                        name.clone(),
                        tag,
                        None,
                        Box::new(prev_lambda),
                    ))
                }
                None => {
                    final_lambda = Some(Element::Lambda(
                        name.clone(),
                        tag,
                        None,
                        Box::new(body.clone()),
                    ))
                }
            }
        }
        final_lambda.unwrap()
    })
}

fn list(gaze: &mut Gaze<Token>) -> Option<Element> {
    match gaze.next() {
        Some(Token::OpenSquare) => (),
        _ => return None,
    }

    let mut contents = vec![];
    while let Some(e) = gaze.attemptf(&mut element_inner) {
        contents.push(e)
    }

    match gaze.next() {
        Some(Token::CloseSquare) => Some(Element::List(contents)),
        _ => None,
    }
}

fn record(gaze: &mut Gaze<Token>) -> Option<Element> {
    match gaze.next() {
        Some(Token::OpenBrace) => (),
        _ => return None,
    }

    let mut contents = HashMap::new();
    while let Some(Element::Name(name)) = gaze.attemptf(&mut name) {
        match gaze.next() {
            Some(Token::EqualSign) => (),
            _ => return None,
        };
        match gaze.attemptf(&mut element_inner) {
            Some(element) => contents.insert(name, element),
            None => None,
        };
    }

    match gaze.next() {
        Some(Token::CloseBrace) => Some(Element::Record(contents)),
        _ => None,
    }
}

fn set(gaze: &mut Gaze<Token>) -> Option<Element> {
    match gaze.next() {
        Some(Token::Hash) => (),
        _ => return None,
    }

    match gaze.next() {
        Some(Token::OpenParen) => (),
        _ => return None,
    }

    let mut contents = HashSet::new();
    while let Some(e) = gaze.attemptf(&mut element_inner) {
        contents.insert(e);
    }

    match gaze.next() {
        Some(Token::CloseParen) => Some(Element::Set(contents)),
        _ => None,
    }
}

fn tuple(gaze: &mut Gaze<Token>) -> Option<Element> {
    match gaze.next() {
        Some(Token::SingleQuote) => (),
        _ => return None,
    }

    match gaze.next() {
        Some(Token::OpenParen) => (),
        _ => return None,
    }

    let mut contents = vec![];
    while let Some(e) = gaze.attemptf(&mut element_inner) {
        contents.push(e)
    }

    match gaze.next() {
        Some(Token::CloseParen) => Some(Element::Tuple(contents)),
        _ => None,
    }
}

fn val_binding(gaze: &mut Gaze<Token>) -> Option<(String, Option<String>, Element)> {
    let name = match (gaze.next(), gaze.next()) {
        (Some(Token::Val), Some(Token::Name(name))) => name,
        _ => return None,
    };
    let tag = match gaze.peek() {
        Some(Token::Colon) => {
            gaze.next();
            if let Some(Token::Name(name)) = gaze.next() {
                Some(name)
            } else {
                return None;
            }
        }
        _ => None,
    };

    match gaze.next() {
        Some(Token::EqualSign) => (),
        _ => return None,
    };

    gaze.attemptf(&mut element).map(|body| (name, tag, body))
}

//this function is basically the same as element inner but it matches name instead of application
fn element_inner(gaze: &mut Gaze<Token>) -> Option<Element> {
    let mut parsers = vec![
        tuple,
        set,
        record,
        name,
        boolean,
        nothing,
        int,
        string,
        let_scope,
        grouped_application,
        conditional,
        lambda,
        list,
    ];
    for &mut mut parser in parsers.iter_mut() {
        if let Some(element) = gaze.attemptf(&mut parser) {
            return Some(element);
        }
    }
    None
}

fn element(gaze: &mut Gaze<Token>) -> Option<Element> {
    let mut parsers = vec![pipe, let_scope, grouping, grouped_application, conditional];
    for &mut mut parser in parsers.iter_mut() {
        if let Some(element) = gaze.attemptf(&mut parser) {
            return Some(element);
        }
    }
    None
}

fn elements(gaze: &mut Gaze<Token>) -> Option<Vec<Element>> {
    let mut results = vec![];
    while !gaze.is_complete() {
        if let Some(element) = gaze.attemptf(&mut element) {
            results.push(element);
        } else {
            return None;
        }
    }
    Some(results)
}

/// Parse a sequence of Tokens into a sequence of ASTs.
pub fn parse(tokens: Vec<Token>) -> Result<Element, WanderError> {
    let mut gaze = Gaze::from_vec(tokens);
    match gaze.attemptf(&mut elements) {
        Some(values) => {
            if values.len() == 1 {
                Ok(values.first().unwrap().clone())
            } else {
                Ok(Element::Grouping(values.to_vec()))
            }
        }
        None => Err(WanderError(format!("Error parsing {:?}", gaze.peek()))),
    }
}
