// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::{bindings::Bindings, HostFunction, WanderError, WanderType, WanderValue};
use std::rc::Rc;

struct EqFunction {}
impl<T: Clone + PartialEq> HostFunction<T> for EqFunction {
    fn run(
        &self,
        arguments: &[WanderValue<T>],
        _bindings: &Bindings<T>,
    ) -> Result<WanderValue<T>, WanderError> {
        if let [left, right] = arguments {
            Ok(crate::WanderValue::Boolean(left == right))
        } else {
            Err(WanderError(
                "`eq` function requires two parameters.".to_owned(),
            ))
        }
    }

    fn doc(&self) -> String {
        "Check if two values are equal.".to_owned()
    }

    fn params(&self) -> Vec<crate::WanderType> {
        vec![WanderType::Any, WanderType::Any]
    }

    fn returns(&self) -> crate::WanderType {
        WanderType::Any
    }

    fn name(&self) -> String {
        "Core.eq".to_owned()
    }
}

struct AssertEqFunction {}
impl<T: Clone + PartialEq> HostFunction<T> for AssertEqFunction {
    fn run(
        &self,
        arguments: &[WanderValue<T>],
        _bindings: &Bindings<T>,
    ) -> Result<WanderValue<T>, WanderError> {
        if let [left, right] = arguments {
            if left == right {
                Ok(crate::WanderValue::Nothing)
            } else {
                Err(WanderError("Assertion failed!".to_owned()))
            }
        } else {
            Err(WanderError(
                "`assertEq` function requires two parameters.".to_owned(),
            ))
        }
    }

    fn doc(&self) -> String {
        "Assert that two values are equal.".to_owned()
    }

    fn params(&self) -> Vec<crate::WanderType> {
        vec![WanderType::Any, WanderType::Any]
    }

    fn returns(&self) -> crate::WanderType {
        WanderType::Nothing
    }

    fn name(&self) -> String {
        "Assert.assertEq".to_owned()
    }
}

struct AndFunction {}
impl<T: Clone + PartialEq> HostFunction<T> for AndFunction {
    fn run(
        &self,
        arguments: &[WanderValue<T>],
        _bindings: &Bindings<T>,
    ) -> Result<crate::WanderValue<T>, WanderError> {
        if let [WanderValue::Boolean(left), WanderValue::Boolean(right)] = arguments[..] {
            Ok(crate::WanderValue::Boolean(left && right))
        } else {
            Err(WanderError(
                "`and` function requires two boolean parameters.".to_owned(),
            ))
        }
    }

    fn doc(&self) -> String {
        "Check if two boolean values are both true.".to_owned()
    }

    fn params(&self) -> Vec<crate::WanderType> {
        vec![WanderType::Boolean, WanderType::Boolean]
    }

    fn returns(&self) -> crate::WanderType {
        WanderType::Boolean
    }

    fn name(&self) -> String {
        "Bool.and".to_owned()
    }
}

struct NotFunction {}
impl<T: Clone + PartialEq> HostFunction<T> for NotFunction {
    fn run(
        &self,
        arguments: &[WanderValue<T>],
        _bindings: &Bindings<T>,
    ) -> Result<crate::WanderValue<T>, WanderError> {
        if let [WanderValue::Boolean(value)] = arguments[..] {
            Ok(crate::WanderValue::Boolean(!value))
        } else {
            Err(WanderError(
                "`not` function requires one boolean parameter.".to_owned(),
            ))
        }
    }

    fn doc(&self) -> String {
        "Return the opposite of the boolean value passed.".to_owned()
    }

    fn params(&self) -> Vec<crate::WanderType> {
        vec![WanderType::Boolean]
    }

    fn returns(&self) -> crate::WanderType {
        WanderType::Boolean
    }

    fn name(&self) -> String {
        "Bool.not".to_owned()
    }
}

struct EntityFunction {}
impl<T: Clone + PartialEq> HostFunction<T> for EntityFunction {
    fn run(
        &self,
        arguments: &[WanderValue<T>],
        _bindings: &Bindings<T>,
    ) -> Result<WanderValue<T>, WanderError> {
        if let [WanderValue::Tuple(value)] = arguments {
            if value.len() == 3 {
                Ok(value.get(0).unwrap().clone())
            } else {
                Err(WanderError(
                    "`entity` function requires one Statement parameter.".to_owned(),
                ))
            }
        } else {
            Err(WanderError(
                "`entity` function requires one Statement parameter.".to_owned(),
            ))
        }
    }

    fn doc(&self) -> String {
        "Retrieve the Entity from a Statement.".to_owned()
    }

    fn params(&self) -> Vec<crate::WanderType> {
        vec![WanderType::Tuple]
    }

    fn returns(&self) -> crate::WanderType {
        WanderType::Identifier
    }

    fn name(&self) -> String {
        "Statement.entity".to_owned()
    }
}

struct AttributeFunction {}
impl<T: Clone + PartialEq> HostFunction<T> for AttributeFunction {
    fn run(
        &self,
        arguments: &[WanderValue<T>],
        _bindings: &Bindings<T>,
    ) -> Result<WanderValue<T>, WanderError> {
        if let [WanderValue::List(value)] = arguments {
            if value.len() == 3 {
                Ok(value.get(1).unwrap().clone())
            } else {
                Err(WanderError(
                    "`attribute` function requires one Statement parameter.".to_owned(),
                ))
            }
        } else {
            Err(WanderError(
                "`attribute` function requires one Statement parameter.".to_owned(),
            ))
        }
    }

    fn doc(&self) -> String {
        "Retrieve the Attribute from a Statement.".to_owned()
    }

    fn params(&self) -> Vec<crate::WanderType> {
        vec![WanderType::Tuple]
    }

    fn returns(&self) -> crate::WanderType {
        WanderType::Identifier
    }

    fn name(&self) -> String {
        "Statement.attribute".to_owned()
    }
}

struct ValueFunction {}
impl<T: Clone + PartialEq> HostFunction<T> for ValueFunction {
    fn run(
        &self,
        arguments: &[WanderValue<T>],
        _bindings: &Bindings<T>,
    ) -> Result<WanderValue<T>, WanderError> {
        if let [WanderValue::List(value)] = arguments {
            if value.len() == 3 {
                Ok(value.get(2).unwrap().clone())
            } else {
                Err(WanderError(
                    "`value` function requires one Statement parameter.".to_owned(),
                ))
            }
        } else {
            Err(WanderError(
                "`value` function requires one Statement parameter.".to_owned(),
            ))
        }
    }

    fn doc(&self) -> String {
        "Retrieve the Value from a Statement.".to_owned()
    }

    fn params(&self) -> Vec<crate::WanderType> {
        vec![WanderType::Tuple]
    }

    fn returns(&self) -> crate::WanderType {
        WanderType::Value
    }

    fn name(&self) -> String {
        "Statement.value".to_owned()
    }
}

struct AtFunction {}
impl<T: Clone + PartialEq> HostFunction<T> for AtFunction {
    fn run(
        &self,
        arguments: &[WanderValue<T>],
        _: &Bindings<T>,
    ) -> Result<WanderValue<T>, WanderError> {
        if let [WanderValue::Int(index), WanderValue::List(value)] = arguments {
            let index: usize = index.to_owned().try_into().unwrap();
            if index < value.len() {
                let t: Option<&WanderValue<T>> = value.get(index);
                match t {
                    Some(t) => Ok(t.to_owned()),
                    None => Err(WanderError("`at` function err.".to_owned())),
                }
            } else {
                Err(WanderError("`at` function err.".to_owned()))
            }
        } else {
            Err(WanderError("`at` function err.".to_owned()))
        }
    }

    fn doc(&self) -> String {
        "Get the value at a given location.".to_owned()
    }

    fn params(&self) -> Vec<crate::WanderType> {
        vec![WanderType::Int, WanderType::List]
    }

    fn returns(&self) -> crate::WanderType {
        WanderType::Any
    }

    fn name(&self) -> String {
        "List.at".to_owned()
    }
}

//TODO https://github.com/almibe/ligature-rs/issues/305
// struct EnvironmentFunction {}
// impl HostFunction for EnvironmentFunction {
//     fn run(
//         &self,
//         arguments: &[WanderValue],
//         bindings: &Bindings,
//     ) -> Result<WanderValue, WanderError> {
//         if arguments.is_empty() {
//             let b: BTreeSet<Statement> = bindings
//                 .environment()
//                 .iter()
//                 .flat_map(|e| {
//                     let mut statements = vec![];
//                     let name = Identifier::new(e.name.as_str()).unwrap();
//                     statements.push(Statement {
//                         entity: name.clone(),
//                         attribute: Identifier::new("doc").unwrap(),
//                         value: Value::String(e.doc_string.clone()),
//                     });
//                     statements.push(Statement {
//                         entity: name.clone(),
//                         attribute: Identifier::new("parameters").unwrap(),
//                         value: Value::String(format!("{:?}", e.parameters)),
//                     });
//                     statements.push(Statement {
//                         entity: name.clone(),
//                         attribute: Identifier::new("result").unwrap(),
//                         value: Value::String(format!("{:?}", e.result)),
//                     });
//                     statements
//                 })
//                 .collect();
//             Ok(WanderValue::Graph(Graph::new(b)))
//         } else {
//             panic!("should never reach")
//         }
//     }

//     fn doc(&self) -> String {
//         "All Functions in the current Environment.".to_owned()
//     }

//     fn params(&self) -> Vec<WanderType> {
//         vec![]
//     }

//     fn returns(&self) -> WanderType {
//         WanderType::Graph
//     }

//     fn name(&self) -> String {
//         "Halp.environment".to_owned()
//     }
// }

/// Creates a set of Bindings for Wander that consists of all of the common
/// functionality, but doesn't interact with an instance of Ligature.
pub fn common<T: Clone + PartialEq>() -> Bindings<T> {
    let mut bindings = Bindings::new();
    bindings.bind_host_function(Rc::new(EqFunction {}));

    bindings.bind_host_function(Rc::new(AssertEqFunction {}));

    bindings.bind_host_function(Rc::new(AndFunction {}));
    bindings.bind_host_function(Rc::new(NotFunction {}));

    bindings.bind_host_function(Rc::new(EntityFunction {}));
    bindings.bind_host_function(Rc::new(AttributeFunction {}));
    bindings.bind_host_function(Rc::new(ValueFunction {}));

    bindings.bind_host_function(Rc::new(AtFunction {}));

    // bindings.bind_host_function(Rc::new(EnvironmentFunction {}));

    bindings
}
