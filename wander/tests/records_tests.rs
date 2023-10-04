// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::collections::HashMap;
use wander::{preludes::common, run, NoHostType, WanderValue};

#[test]
fn basic_record() {
    let input = "(a: 24)";
    let res = run(input, &mut common::<NoHostType>()).unwrap();
    let res = format!("{res}");
    let res = run(&res, &mut common::<NoHostType>()).unwrap();
    let mut record = HashMap::new();
    record.insert("a".to_owned(), WanderValue::Int(24));
    let expected = WanderValue::Record(record);
    assert_eq!(res, expected);
}

//#[test]
fn nested_record() {
    let input = "(a: 24 b: \"c\" c: (d: (\"e\")))";
    let res = run(input, &mut common::<NoHostType>()).unwrap();
    let res = format!("{res}");
    let res = run(&res, &mut common::<NoHostType>()).unwrap();
    let mut record = HashMap::new();
    record.insert("a".to_owned(), WanderValue::Int(24));
    record.insert("b".to_owned(), WanderValue::String("c".to_owned()));

    let mut inner_record = HashMap::new();
    inner_record.insert(
        "d".to_owned(),
        WanderValue::Tuple(vec![WanderValue::String("e".to_owned())]),
    );

    record.insert("c".to_owned(), WanderValue::Record(inner_record));

    let expected = WanderValue::Record(record);
    assert_eq!(res, expected);
}

//#[test]
fn record_field_access() {
    let input = "val x = (a: 24 b: true) x.b";
    let res = run(input, &mut common::<NoHostType>()).unwrap();
    let expected = WanderValue::Boolean(true);
    assert_eq!(res, expected);
}

//#[test]
fn nested_record_field_access() {
    let input = "val x = (a: 45 b: (a: 45)) x.b.a";
    let res = run(input, &mut common::<NoHostType>()).unwrap();
    let expected = WanderValue::Int(45);
    assert_eq!(res, expected);
}

//#[test]
fn nested_record_field_access2() {
    let input = "val x = (a: 24 b: (a: [] b: (c: 45))) x.b.b.c";
    let res = run(input, &mut common::<NoHostType>()).unwrap();
    let expected = WanderValue::Int(45);
    assert_eq!(res, expected);
}

#[test]
fn missing_record_field_access() {
    let input = "val x = (a: 24 b: true) x.c";
    let res = run(input, &mut common::<NoHostType>());
    assert!(res.is_err());
}

#[test]
fn nested_missing_record_field_access() {
    let input = "val x = (a: 24 b: (a: [], b: (c: 45))) x.b.b.d";
    let res = run(input, &mut common::<NoHostType>());
    assert!(res.is_err());
}

#[test]
fn nested_missing_record_field_access2() {
    let input = "val x = (a: 24 b: (a: [], b: (c: 45))) x.c.b.d";
    let res = run(input, &mut common::<NoHostType>());
    assert!(res.is_err());
}
