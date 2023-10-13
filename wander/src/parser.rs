// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::collections::{HashMap, HashSet};

use gaze::Gaze;
use serde::{Deserialize, Serialize};

use crate::{lexer::Token, WanderError, WanderType};

#[doc(hidden)]
#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
pub enum Element {
    Boolean(bool),
    Int(i64),
    String(String),
    Name(String),
    HostFunction(String),
    Let(Vec<(String, Element)>, Box<Element>),
    Application(Vec<Element>),
    Conditional(Box<Element>, Box<Element>, Box<Element>),
    Lambda(String, WanderType, WanderType, Box<Element>),
    Tuple(Vec<Element>),
    List(Vec<Element>),
    Set(HashSet<Element>),
    Record(HashMap<String, Element>),
    Nothing,
    Forward,
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

fn forward(gaze: &mut Gaze<Token>) -> Option<Element> {
    match gaze.next() {
        Some(Token::Forward) => Some(Element::Forward),
        _ => None,
    }
}

fn name(gaze: &mut Gaze<Token>) -> Option<Element> {
    match gaze.next() {
        Some(Token::Name(value)) => Some(Element::Name(value)),
        _ => return None,
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
        },
        _ => Some(Element::Let(decls, Box::new(Element::Nothing))),
    }
}

fn application(gaze: &mut Gaze<Token>) -> Option<Element> {
    let mut expressions: Vec<Element> = vec![];

    match gaze.attemptf(&mut name) {
        Some(name) => expressions.push(name),
        _ => return None,
    }

    while let Some(e) = gaze.attemptf(&mut element_inner) {
        expressions.push(e);
    }

    match &expressions[..] {
        [] => None,
        [e] => Some(e.clone()),
        _ => Some(Element::Application(expressions)),
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
        Some(Token::CloseParen) => Some(Element::Application(expressions)),
        _ => return None,
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

    let mut params: Vec<String> = vec![];
    while let Some(Element::Name(name)) = gaze.attemptf(&mut name) {
        params.push(name);
    }

    match gaze.next() {
        Some(Token::Arrow) => (),
        _ => return None,
    }

    gaze.attemptf(&mut element).map(|body| {
        let mut final_lambda = None;
        params.reverse();
        for name in params {
            match final_lambda {
                Some(prev_lambda) => {
                    final_lambda = Some(Element::Lambda(
                        name.clone(),
                        WanderType::Any,
                        WanderType::Any,
                        Box::new(prev_lambda),
                    ))
                }
                None => {
                    final_lambda = Some(Element::Lambda(
                        name.clone(),
                        WanderType::Any,
                        WanderType::Any,
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
        match gaze.attemptf(&mut element) {
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

fn val_binding(gaze: &mut Gaze<Token>) -> Option<(String, Element)> {
    let name = match (gaze.next(), gaze.next(), gaze.next()) {
        (Some(Token::Val), Some(Token::Name(name)), Some(Token::EqualSign)) => name,
        _ => return None,
    };

    if let Some(body) = gaze.attemptf(&mut element) {
        Some((name, body))
    } else {
        None
    }
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
        name,
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
    let mut parsers = vec![
        tuple,
        set,
        record,
        boolean,
        nothing,
        forward,
        int,
        string,
        let_scope,
        application,
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

/// Parse a sequence of Tokens into an AST.
pub fn parse(tokens: Vec<Token>) -> Result<Vec<Element>, WanderError> {
    let mut gaze = Gaze::from_vec(tokens);
    match gaze.attemptf(&mut elements) {
        Some(value) => Ok(value),
        None => Err(WanderError(format!("Error parsing {:?}", gaze.peek()))),
    }
}
