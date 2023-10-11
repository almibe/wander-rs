// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use wander::{preludes::common, run, NoHostType, WanderValue};

#[test]
fn basic_let() {
    let input = "let val x = 5 in x end";
    let res = run(input, &mut common::<NoHostType>()).unwrap();
    let expected = WanderValue::Int(5);
    assert_eq!(res, expected);
}

#[test]
fn basic_let_multiple_vals() {
    let input = "let val x = true val y = Bool.and x in y x end";
    let res = run(input, &mut common::<NoHostType>()).unwrap();
    let expected = WanderValue::Boolean(false);
    assert_eq!(res, expected);
}

#[test]
fn nested_lets() {
    let input = r#"
    let 
        val x = true 
        val y = let 
            val y = Bool.and(x false) 
            in Bool.and(x y) 
        end 
        in y 
    end"#;
    let res = run(input, &mut common::<NoHostType>()).unwrap();
    let expected = WanderValue::Boolean(false);
    assert_eq!(res, expected);
}
