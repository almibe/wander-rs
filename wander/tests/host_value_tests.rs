// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::rc::Rc;

use wander::interpreter::eval;
use wander::parser::Element;
use wander::preludes::common;
use wander::{HostFunction, HostFunctionBinding, HostValue, WanderError, WanderType, WanderValue};

struct SayHello {}
impl HostFunction<String> for SayHello {
    fn run(
        &self,
        _arguments: &[WanderValue<String>],
        _bindings: &wander::bindings::Bindings<String>,
    ) -> Result<WanderValue<String>, WanderError> {
        Ok(WanderValue::HostValue(HostValue {
            value: "hello!".to_owned(),
        }))
    }

    fn binding(&self) -> HostFunctionBinding {
        HostFunctionBinding {
            name: "hello".to_owned(),
            parameters: vec![],
            result: WanderType::String,
            doc_string: "Say hello!".to_owned(),
        }
    }
}

//#[test]
// fn eval_host_value() {
//     let mut bindings = common::<String>();
//     bindings.bind_host_function(Rc::new(SayHello {}));
//     let input = vec![Element::Application(vec![])];
//     let res = eval(&input, &mut bindings);
//     let expected = Ok(WanderValue::HostValue(HostValue {
//         value: "hello!".to_owned(),
//     }));
//     assert_eq!(res, expected);
// }
