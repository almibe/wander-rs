// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use wander::lexer::Token;
use wander::parser::{parse, Element};
use wander::WanderType;

use crate::utilities::parse_str;

#[path = "utilities.rs"]
mod utilities;

#[test]
fn parse_booleans() {
    let res = parse_str("true");
    let expected = Element::Grouping(vec![Element::Boolean(true)]);
    assert_eq!(res, expected);
}

#[test]
fn parse_integers() {
    let res = parse_str("-100");
    let expected = Element::Grouping(vec![Element::Int(-100)]);
    assert_eq!(res, expected);
}

#[test]
fn parse_strings() {
    let res = parse_str("\"Hello\"");
    let expected = Element::Grouping(vec![Element::String("Hello".to_owned())]);
    assert_eq!(res, expected);
}

#[test]
fn parse_name() {
    let input = vec![Token::Name(String::from("test"))];
    let expected = Ok(Element::Grouping(vec![Element::Name(String::from("test"))]));
    let res = parse(input);
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
    let expected = Ok(Element::Grouping(vec![Element::Lambda(
        "test".to_owned(),
        WanderType::Any,
        WanderType::Any,
        Box::new(Element::Grouping(vec![Element::Name("test".to_owned())])),
    )]));
    assert_eq!(res, expected);
}

#[test]
fn parse_list() {
    let res = utilities::parse_str("[test 24601]");
    let expected = Element::Grouping(vec![Element::List(vec![
        Element::Name("test".to_owned()),
        Element::Int(24601),
    ])]);
    assert_eq!(res, expected);
}

#[test]
fn parse_tuple() {
    let res = utilities::parse_str("'(test 24601)");
    let expected = Element::Grouping(vec![Element::Tuple(vec![
        Element::Name("test".to_owned()),
        Element::Int(24601),
    ])]);
    assert_eq!(res, expected);
}

#[test]
fn parse_applications() {
    let res = utilities::parse_str("Bool.not x true");
    let expected = Element::Grouping(vec![
        Element::Name("Bool.not".to_owned()),
        Element::Name("x".to_owned()),
        Element::Boolean(true),
    ]);
    assert_eq!(res, expected);
}

#[test]
fn parse_nested_function_calls() {
    let res = utilities::parse_str("Bool.not (Bool.not false)");
    let expected = Element::Grouping(vec![
        Element::Name("Bool.not".to_owned()),
        Element::Grouping(vec![
            Element::Name("Bool.not".to_owned()),
            Element::Boolean(false),
        ]),
    ]);
    assert_eq!(res, expected);
}
