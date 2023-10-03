// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use wander::{parser::Element, preludes::common, run, NoHostType, WanderType, WanderValue};

#[test]
fn basic_currying() {
    let input = "val isTrue = Bool.and(true) [isTrue(true) isTrue(false)]";
    let res = run(input, &mut common::<NoHostType>()).unwrap();
    let res = format!("{res}");
    let res = run(&res, &mut common::<NoHostType>()).unwrap();
    let expected = WanderValue::List(vec![
        WanderValue::Boolean(true),
        WanderValue::Boolean(false),
    ]);
    assert_eq!(res, expected);
}

#[test]
fn currying_with_lambda() {
    let input =
        "val and = { x y -> Bool.and(x y) } val isTrue = and(true) [isTrue(true) isTrue(false)]";
    let res = run(input, &mut common::<NoHostType>()).unwrap();
    let res = format!("{res}");
    let res = run(&res, &mut common::<NoHostType>()).unwrap();
    let expected = WanderValue::List(vec![
        WanderValue::Boolean(true),
        WanderValue::Boolean(false),
    ]);
    assert_eq!(res, expected);
}

#[test]
fn partial_application_twice_with_lambda() {
    let input = "val and3 = { x y z -> Bool.and(z Bool.and(x y)) } val and = and3(true) val isTrue = and(true) and(isTrue(true) isTrue(false))";
    let res = run(input, &mut common::<NoHostType>()).unwrap();
    let res = format!("{res}");
    let res = run(&res, &mut common::<NoHostType>()).unwrap();
    let expected = WanderValue::Boolean(false);
    assert_eq!(res, expected);
}

#[test]
fn parse_lambda() {
    let input = "\\x -> x";
    let res = run(input, &mut common::<NoHostType>()).unwrap();
    let expected = WanderValue::Lambda(
        "x".to_owned(),
        WanderType::Any,
        WanderType::Any,
        Box::new(Element::Name("x".to_owned())),
    );
    assert_eq!(res, expected);
}

#[test]
fn parse_multi_line_lambda() {
    let input = "\\x -> { x x x x }";
    let res = run(input, &mut common::<NoHostType>()).unwrap();
    let expected = WanderValue::Lambda(
        "x".to_owned(),
        WanderType::Any,
        WanderType::Any,
        Box::new(Element::Scope(vec![
            Element::Name("x".to_owned()),
            Element::Name("x".to_owned()),
            Element::Name("x".to_owned()),
            Element::Name("x".to_owned()),
        ])),
    );
    assert_eq!(res, expected);
}
