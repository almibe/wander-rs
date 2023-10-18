// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use wander::parser::Element;
use wander::{run, NoHostType, WanderValue};

use crate::utilities::{introspect_str, parse_str};
use wander::interpreter::Expression;
use wander::preludes::common;
use wander::translation::translate;

#[path = "utilities.rs"]
mod utilities;

#[test]
fn parse_forward_value_to_name() {
    let res = introspect_str("false >> not");
    let expected = Element::Grouping(vec![
        Element::Grouping(vec![Element::Boolean(false)]),
        Element::Forward,
        Element::Grouping(vec![Element::Name("not".to_owned())]),
    ]);
    assert_eq!(res.element, expected);
    let expected = Expression::Application(vec![
        Expression::Name("not".to_owned()),
        Expression::Boolean(false),
    ]);
    assert_eq!(res.expression, expected);
}

#[test]
fn parse_forward_value_to_application() {
    let res = parse_str("false >> Bool.and true");
    let expected = Element::Grouping(vec![
        Element::Grouping(vec![Element::Boolean(false)]),
        Element::Forward,
        Element::Grouping(vec![
            Element::Name("Bool.and".to_owned()),
            Element::Boolean(true),
        ]),
    ]);
    assert_eq!(res, expected);
    let res = translate(res).unwrap();
    let expected = Expression::Application(vec![
        Expression::Name("Bool.and".to_owned()),
        Expression::Boolean(true),
        Expression::Boolean(false),
    ]);
    assert_eq!(res, expected);
}

#[test]
fn parse_forward_application_to_application() {
    let res = parse_str("Bool.not false >> Bool.and true");
    let expected = Element::Grouping(vec![
        Element::Grouping(vec![
            Element::Name("Bool.not".to_owned()),
            Element::Boolean(false),
        ]),
        Element::Forward,
        Element::Grouping(vec![
            Element::Name("Bool.and".to_owned()),
            Element::Boolean(true),
        ]),
    ]);
    assert_eq!(res, expected);
    let res = translate(res).unwrap();
    let expected = Expression::Application(vec![
        Expression::Name("Bool.and".to_owned()),
        Expression::Boolean(true),
        Expression::Application(vec![
            Expression::Name("Bool.not".to_owned()),
            Expression::Boolean(false),
        ]),
    ]);
    assert_eq!(res, expected);
}

#[test]
fn run_forward_value_to_name() {
    let res = run("false >> Bool.not", &mut common::<NoHostType>());
    let expected = Ok(WanderValue::Boolean(true));
    assert_eq!(res, expected);
}

#[test]
fn run_forward_value_to_application() {
    let res = run("false >> Bool.and true", &mut common::<NoHostType>());
    let expected = Ok(WanderValue::Boolean(false));
    assert_eq!(res, expected);
}

#[test]
fn run_forward_application_to_application() {
    let res = run(
        "Bool.not false >> Bool.and true",
        &mut common::<NoHostType>(),
    );
    let expected = Ok(WanderValue::Boolean(true));
    assert_eq!(res, expected);
}

#[test]
fn run_multiple_forwards() {
    let res = run(
        "Bool.not false >> Bool.and true >> Bool.not",
        &mut common::<NoHostType>(),
    );
    let expected = Ok(WanderValue::Boolean(false));
    assert_eq!(res, expected);
}
