// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use wander::lexer::Token;
use wander::parser::{parse, Element};
use wander::WanderType;

#[path = "utilities.rs"]
mod utilities;

#[test]
fn parse_booleans() {
    let input = vec![
        Token::Boolean(true),
        Token::Boolean(false),
        Token::Boolean(true),
    ];
    let res = parse(input);
    let expected = Ok(vec![
        Element::Boolean(true),
        Element::Boolean(false),
        Element::Boolean(true),
    ]);
    assert_eq!(res, expected);
}

#[test]
fn parse_integers() {
    let input = vec![Token::Int(0), Token::Int(-100), Token::Int(4200)];
    let res = parse(input);
    let expected = Ok(vec![
        Element::Int(0),
        Element::Int(-100),
        Element::Int(4200),
    ]);
    assert_eq!(res, expected);
}

#[test]
fn parse_strings() {
    let input = vec![
        Token::String(String::from("Hello")),
        Token::String(String::from("This is a test")),
    ];
    let res = parse(input);
    let expected = Ok(vec![
        Element::String(String::from("Hello")),
        Element::String(String::from("This is a test")),
    ]);
    assert_eq!(res, expected);
}

#[test]
fn parse_name() {
    let input = vec![Token::Name(String::from("test"))];
    let expected = Ok(vec![Element::Name(String::from("test"))]);
    let res = parse(input);
    assert_eq!(res, expected);
}

#[test]
fn parse_conditional() {
    let res = utilities::parse_str("if true then 5 else 6 end");
    let expected = vec![Element::Conditional(
        Box::new(Element::Boolean(true)),
        Box::new(Element::Int(5)),
        Box::new(Element::Int(6)),
    )];
    assert_eq!(res, expected);
}

#[test]
fn parse_lambda() {
    let input = vec![
        Token::Lambda,
        Token::Name("test".to_owned()),
        Token::Arrow,
        Token::Name("test".to_owned()),
    ];
    let res = parse(input);
    let expected = Ok(vec![Element::Lambda(
        "test".to_owned(),
        WanderType::Any,
        WanderType::Any,
        Box::new(Element::Name("test".to_owned())),
    )]);
    assert_eq!(res, expected);
}

#[test]
fn parse_list() {
    let res = utilities::parse_str("[test 24601]");
    let expected = vec![Element::List(vec![
        Element::Name("test".to_owned()),
        Element::Int(24601),
    ])];
    assert_eq!(res, expected);
}

#[test]
fn parse_tuple() {
    let res = utilities::parse_str("'(test 24601)");
    let expected = vec![Element::Tuple(vec![
        Element::Name("test".to_owned()),
        Element::Int(24601),
    ])];
    assert_eq!(res, expected);
}

#[test]
fn parse_applications() {
    let res = utilities::parse_str("Bool.not x true");
    let expected = vec![Element::Application(vec![
        Element::Name("Bool.not".to_owned()),
        Element::Name("x".to_owned()),
        Element::Boolean(true),
    ])];
    assert_eq!(res, expected);
}

#[test]
fn parse_nested_function_calls() {
    let res = utilities::parse_str("Bool.not (Bool.not false)");
    let expected = vec![
        Element::Application(
            vec![
                Element::Name("Bool.not".to_owned()), 
                Element::Application(
                    vec![
                        Element::Name("Bool.not".to_owned()), 
                        Element::Boolean(false)])])
    ];
    assert_eq!(res, expected);
}
