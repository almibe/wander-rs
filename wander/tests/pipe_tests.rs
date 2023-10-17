// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use wander::parser::Element;

use crate::utilities::introspect_str;
use wander::interpreter::Expression;

#[path = "utilities.rs"]
mod utilities;

#[test]
fn parse_forward_value_to_name() {
    let res = introspect_str("false >> not");
    let expected = Element::Grouping(vec![
        Element::Boolean(false),
        Element::Forward,
        Element::Name("not".to_owned()),
    ]);
    assert_eq!(res.element, expected);
    let expected = Expression::Application(vec![
        Expression::Name("not".to_owned()),
        Expression::Boolean(false),
    ]);
    assert_eq!(res.expression, expected);
}

// #[test]
// fn parse_forward_value_to_application() {
//     let res = parse_str("false >> Bool.and true");
//     let expected = Element::Grouping(vec![
//         Element::Boolean(false),
//         Element::Forward,
//         Element::Grouping(vec![
//             Element::Name("Bool.and".to_owned()),
//             Element::Boolean(true),
//         ]),
//     ]);
//     assert_eq!(res, expected);
//     let res = translate(&res).unwrap();
//     let expected = Expression::Application(vec![
//         Expression::Name("Bool.and".to_owned()),
//         Expression::Boolean(true),
//         Expression::Boolean(false)]);
//     assert_eq!(res, expected);
// }

// #[test]
// fn parse_forward_application_to_application() {
//     let res = parse_str("Bool.not false >> Bool.and true");
//     let expected = Element::Grouping(vec![
//         Element::Name("Bool.not".to_owned()),
//         Element::Boolean(false),
//         Element::Forward,
//         Element::Name("Bool.and".to_owned()),
//         Element::Boolean(true),
//     ]);
//     assert_eq!(res, expected);
//     let res = translate(&res).unwrap();
//     let expected = Expression::Application(vec![
//         Expression::Name("Bool.and".to_owned()),
//         Expression::Boolean(true),
//         Expression::Application(vec![
//             Expression::Name("Bool.not".to_owned()),
//             Expression::Boolean(false),
//         ])]);
//     assert_eq!(res, expected);
// }

// #[test]
// fn run_forward_value_to_name() {
//     let res = run("false >> not");
//     let expected = ;
//     assert_eq!(res, expected);
// }

// #[test]
// fn run_forward_value_to_application() {
//     let res = parse_str("false >> Bool.and true");
//     let expected = vec![
//         Element::Boolean(false),
//         Element::Forward,
//         Element::Name("not".to_owned()),
//     ];
//     assert_eq!(res, expected);
//     let res = translate(res).unwrap();
//     let expected = Expression::Application(vec![
//         Expression::Name("not".to_owned()),
//         Expression::Boolean(false)]);
//     assert_eq!(res, expected);
// }

// #[test]
// fn run_forward_value_to_application() {
//     let res = parse_str("Bool.not false >> Bool.and true");
//     let expected = vec![
//         Element::Boolean(false),
//         Element::Forward,
//         Element::Name("not".to_owned()),
//     ];
//     assert_eq!(res, expected);
//     let res = translate(res).unwrap();
//     let expected = Expression::Application(vec![
//         Expression::Name("not".to_owned()),
//         Expression::Boolean(false)]);
//     assert_eq!(res, expected);
// }

//TODO add tests with multiple pipes in a single expression
