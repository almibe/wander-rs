// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::{HostFunction, HostFunctionBinding, TokenTransformer, WanderValue, WanderType, parser::Element, interpreter::Expression};
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
};

/// A structure used to setup the environment a Wander program is executed in.
#[derive(Default)]
pub struct Bindings<T: Clone + PartialEq + Eq> {
    token_transformers: RefCell<HashMap<String, Rc<TokenTransformer>>>,
    host_functions: RefCell<HashMap<String, Rc<dyn HostFunction<T>>>>,
    scopes: Vec<HashMap<String, WanderValue<T>>>,
}

///
// pub trait BindingsProvider<T: Clone> {
//     fn add_bindings(&self, bindings: &mut Bindings<T>);
// }

impl<T: Clone + PartialEq + Eq> Bindings<T> {
    /// Create a new empty Bindings.
    pub fn new() -> Bindings<T> {
        Bindings {
            token_transformers: RefCell::new(HashMap::new()),
            host_functions: RefCell::new(HashMap::new()),
            scopes: vec![HashMap::new()],
        }
    }

    /// Add a new Scope to these Bindings.
    pub fn add_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    /// Remove the current Scope from these Bindings.
    pub fn remove_scope(&mut self) {
        self.scopes.pop();
    }

    /// Read a bound Value.
    pub fn read(&self, name: &String) -> Option<WanderValue<T>> {
        let mut index = self.scopes.len();
        while index > 0 {
            match self.scopes.get(index - 1) {
                Some(scope) => {
                    if let Some(value) = scope.get(name) {
                        return Some(value.clone());
                    }
                }
                _ => return None,
            }
            index -= 1;
        }
        None
    }

    /// Bind a new Value in this Scope.
    pub fn bind(&mut self, name: String, value: WanderValue<T>) {
        let mut current_scope = self.scopes.pop().unwrap();
        current_scope.insert(name, value);
        self.scopes.push(current_scope);
    }

    /// Add a new HostFunction.
    pub fn bind_host_function(&mut self, function: Rc<dyn HostFunction<T>>) {
        let full_name = function.binding().name.to_string();
        self.host_functions.borrow_mut().insert(full_name.clone(), function.clone());
        let mut p = function.binding().parameters.clone();
        let mut result = None;
        p.reverse();
        p.iter().for_each(|p| {
            match &result {
                Some(value) => {
                    match value {
                        WanderValue::Lambda(innerp, i, o, b) => {
                            let p = p.clone();
                            result = Some(WanderValue::Lambda(p.0, p.1, WanderType::Any, Box::new(Element::Lambda(innerp.clone(), i.clone(), o.clone(), b.clone()))));
                        },
                        _ => panic!("Should never reach."),
                    }
                },
                None => {
                    let p = p.clone();
                    result = Some(WanderValue::Lambda(p.0, p.1, WanderType::Any, Box::new(Element::HostFunction(full_name.clone()))));
                },
            }
        });
        self.bind(full_name, result.unwrap());
    }

    /// Read a HostFunction.
    pub fn read_host_function(&self, name: &String) -> Option<Rc<dyn HostFunction<T>>> {
        match self.host_functions.borrow().get(name) {
            None => None,
            Some(value) => Some(value.clone()),
        }
    }

    /// Add a Token Transformer.
    pub fn bind_token_transformer(
        &mut self,
        module: String,
        name: String,
        transformer: Rc<TokenTransformer>,
    ) {
        let full_name = format!("{module}.{name}");
        self.token_transformers
            .borrow_mut()
            .insert(full_name, transformer);
    }

    /// Read a Token Transformer.
    pub fn read_token_transformer(&self, name: &String) -> Option<Rc<TokenTransformer>> {
        self.token_transformers.borrow().get(name).cloned()
    }

    /// Get a collection of all names.
    pub fn bound_names(&self) -> HashSet<String> {
        let mut names = HashSet::new();
        for native_function in self.host_functions.borrow().keys() {
            names.insert(native_function.clone());
        }
        for scope in self.scopes.iter() {
            for name in scope.keys() {
                names.insert(name.clone());
            }
        }
        names
    }

    pub fn environment(&self) -> Vec<HostFunctionBinding> {
        todo!()
    }
}
