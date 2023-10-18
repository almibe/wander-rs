// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! This module is an implementation of the Wander language.

#![deny(missing_docs)]

use std::{
    collections::{HashMap, HashSet},
    fmt::{Debug, Display, Write},
};

use bindings::Bindings;
use interpreter::{eval, Expression};
use lexer::{tokenize, tokenize_and_filter, transform, Token};
use parser::{parse, Element};
use serde::{Deserialize, Serialize};
use translation::translate;

#[doc(hidden)]
pub mod bindings;
#[doc(hidden)]
pub mod interpreter;
#[doc(hidden)]
pub mod lexer;
#[doc(hidden)]
pub mod parser;
#[doc(hidden)]
pub mod preludes;
#[doc(hidden)]
pub mod translation;

/// An error that occurs while running a Wander script.
#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct WanderError(pub String);

/// A combination of all the traits needed to implement a HostType.
pub trait HostType: Debug + PartialEq + Eq + Serialize + Clone + Display + Serialize {}
impl<T> HostType for T where T: Debug + PartialEq + Eq + Serialize + Clone + Display + Serialize {}

trait TypeSystem<T: HostType> {
    fn check(value: WanderValue<T>, type_value: WanderValue<T>) -> Result<bool, WanderError>;
}

/// This is a dummy type you can use when you don't need a HostType.
#[derive(Debug, PartialEq, Eq, Deserialize, Serialize, Clone)]
pub struct NoHostType {}

impl Display for NoHostType {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        panic!("NoHostType should never be displayed.")
    }
}

/// Data for
pub struct HostFunctionBinding {
    /// Name used to bind this HostFunction including Namespaces.
    pub name: String,
    /// The type of the parameters this HostFunction takes.
    pub parameters: Vec<(String, WanderType)>,
    /// The type of the result of this HostFunction.
    pub result: WanderType,
    /// The documentation for this HostFunction.
    /// Can be text or Markdown.
    pub doc_string: String,
}

/// A trait representing a function exported from the hosting application that
/// can be called from Wander.
pub trait HostFunction<T: Clone + PartialEq + Eq> {
    /// The function called when the HostFunction is called from Wander.
    fn run(
        &self,
        arguments: &[WanderValue<T>],
        bindings: &Bindings<T>,
    ) -> Result<WanderValue<T>, WanderError>;
    /// Get the binding information for this HostFunction.
    fn binding(&self) -> HostFunctionBinding;
}

/// Type alias used for TokenTransformers.
pub type TokenTransformer = fn(&[Token]) -> Result<Vec<Token>, WanderError>;

/// Types of values allowed in Wander.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WanderType {
    /// Allow any type.
    Any,
    /// A Boolean value.
    Boolean,
    /// A signed 64-bit Integer.
    Int,
    /// A String value.
    String,
    /// The nothing value.
    Nothing,
    /// A Lambda.
    Lambda,
    /// A List.
    List,
    /// A tuple.
    Tuple,
    /// An Optional Value.
    Optional(Box<WanderType>),
}

/// A value of a type provided by the host application that can be accessed via Wander.
/// Note it cannot be accessed by Wander directly, only through HostFunctions.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct HostValue<T> {
    /// The value passed to Wander.
    /// Note it cannot be accessed by Wander directly, only through HostFunctions.
    pub value: T,
}

/// Values in Wander programs used for Wander's implementation and interfacing between
/// Wander and the host application.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub enum WanderValue<T: Clone + PartialEq + Eq> {
    /// A Boolean value.
    Boolean(bool),
    /// An Integer value.
    Int(i64),
    /// A String value.
    String(String),
    /// The nothing value.
    Nothing,
    /// A Lambda
    Lambda(String, WanderType, WanderType, Box<Element>),
    /// A List.
    List(Vec<WanderValue<T>>),
    /// A Tuple.
    Tuple(Vec<WanderValue<T>>),
    /// A Set.
    Set(HashSet<WanderValue<T>>),
    /// A Record.
    Record(HashMap<String, WanderValue<T>>),
    /// A HostValue.
    HostValue(HostValue<T>),
}

impl<T: Clone + PartialEq + Eq> core::hash::Hash for WanderValue<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);
    }
}

/// A struct represting a partially applied function.
/// The function can be a Lambda or a HostFunction.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct PartialApplication<T: Clone + PartialEq + Eq> {
    arguments: Vec<WanderValue<T>>,
    callee: WanderValue<T>,
}

/// Write integer.
pub fn write_integer(integer: &i64) -> String {
    format!("{}", integer)
}

/// Write float.
pub fn write_float(float: &f64) -> String {
    let res = format!("{}", float);
    if res.contains('.') {
        res
    } else {
        res + ".0"
    }
}

// Encode a
// pub fn write_bytes(bytes: &Bytes) -> String {
//     format!("0x{}", encode(bytes))
// }

/// Escape a String value.
pub fn write_string(string: &str) -> String {
    //TODO this could be done better
    let escaped_string = string
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        //.replace("\f", "\\b") <-- TODO not sure how to handle this or if I really need to
        //.replace("\b", "\\b") <-- TODO not sure how to handle this or if I really need to
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t");
    format!("\"{}\"", escaped_string)
}

fn write_list_or_tuple_wander_value<T: Clone + Display + PartialEq + Eq + Debug>(
    open: &str,
    close: char,
    contents: &Vec<WanderValue<T>>,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    f.write_str(open).unwrap();
    let mut i = 0;
    for value in contents {
        write!(f, "{value}").unwrap();
        i += 1;
        if i < contents.len() {
            write!(f, " ").unwrap();
        }
    }
    write!(f, "{close}")
}

fn write_set<T: Clone + Display + PartialEq + Eq + Debug>(
    contents: &HashSet<WanderValue<T>>,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    f.write_str("#(").unwrap();
    let mut i = 0;
    for value in contents {
        write!(f, "{value}").unwrap();
        i += 1;
        if i < contents.len() {
            write!(f, " ").unwrap();
        }
    }
    f.write_char(')')
}

fn write_host_value<T: Display + PartialEq + Eq>(
    value: &HostValue<T>,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    write!(f, "{}", value.value)
}

fn write_record<T: Clone + Display + PartialEq + Eq + Debug>(
    contents: &HashMap<String, WanderValue<T>>,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    write!(f, "{{").unwrap();
    let mut i = 0;
    for (name, value) in contents {
        write!(f, "{name} = {value}").unwrap();
        i += 1;
        if i < contents.len() {
            write!(f, " ").unwrap();
        }
    }
    write!(f, "}}")
}

impl<T: Clone + Display + PartialEq + Eq + std::fmt::Debug> Display for WanderValue<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WanderValue::Boolean(value) => write!(f, "{}", value),
            WanderValue::Int(value) => write!(f, "{}", value),
            WanderValue::String(value) => f.write_str(&write_string(value)),
            WanderValue::Nothing => write!(f, "nothing"),
            WanderValue::List(contents) => write_list_or_tuple_wander_value("[", ']', contents, f),
            WanderValue::HostValue(value) => write_host_value(value, f),
            WanderValue::Tuple(contents) => {
                write_list_or_tuple_wander_value("'(", ')', contents, f)
            }
            WanderValue::Record(values) => write_record(values, f),
            WanderValue::Lambda(p, i, o, b) => write!(
                f,
                "[lambda {:?}]",
                WanderValue::Lambda::<T>(p.clone(), i.clone(), o.clone(), b.clone())
            ),
            WanderValue::Set(contents) => write_set(contents, f),
        }
    }
}

/// Run a Wander script with the given Bindings.
pub fn run<T: HostType + Display>(
    script: &str,
    bindings: &mut Bindings<T>,
) -> Result<WanderValue<T>, WanderError> {
    let tokens = tokenize_and_filter(script)?;
    let tokens = transform(&tokens, bindings)?;
    let elements = parse(tokens)?;
    let expression = translate(elements)?;
    eval(&expression, bindings)
}

#[derive(Debug, Serialize)]
/// Structure used for debugging or inspecting code.
pub struct Introspection {
    /// A list of all Tokens including whitespace.
    pub tokens_ws: Vec<Token>,
    /// A list of all Tokens without whitespace.
    pub tokens: Vec<Token>,
    /// A list of all Tokens after macro transformations.
    pub tokens_transformed: Vec<Token>,
    /// Element representation.
    pub element: Element,
    /// Expression representation.
    pub expression: Expression,
}

/// Run a Wander script with the given Bindings.
pub fn introspect<T: Clone + PartialEq + Eq>(
    script: &str,
    bindings: &Bindings<T>,
) -> Result<Introspection, WanderError> {
    let tokens_ws = tokenize(script).or(Ok(vec![]))?;
    let tokens = tokenize_and_filter(script).or(Ok(vec![]))?;
    let tokens_transformed = transform(&tokens.clone(), bindings).or(Ok(vec![]))?;
    let element = parse(tokens_transformed.clone())?; //.or(Ok(Element::String("Error".to_owned())))?; //TODO handle errors better
    let expression = translate(element.clone())?; //.or(Ok(Expression::String("Error".to_owned())))?; //TODO handle errors better
    Ok(Introspection {
        tokens_ws,
        tokens,
        tokens_transformed,
        element,
        expression,
    })
}
