// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! This module is the main module for the ligature-repl project.
//! It is an application that .

use rustyline::error::ReadlineError;
use rustyline::{Editor, Result};
use tabled::{
    settings::{object::Rows, Modify, Width},
    Table, Tabled,
};
use wander::bindings::Bindings;
use wander::preludes::common;
use wander::{run, HostFunctionBinding, NoHostType};

struct REPLState<T: Clone + PartialEq> {
    bindings: Bindings<T>,
}

fn main() -> Result<()> {
    let bindings = common::<NoHostType>();
    let mut state = REPLState { bindings };
    println!("Welcome to Wander's REPL!");
    println!("Press Ctrl+C or Ctrl+D or enter `:q` to quit.");
    println!("Enter :help or :h for help.");
    println!("---");
    let mut rl = Editor::<()>::new()?;
    if rl.load_history("history.txt").is_err() {
        println!("No previous history.");
    }
    loop {
        let readline = rl.readline("> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str());
                if line.trim().starts_with(':') {
                    if !handle_command(&line, &mut state) {
                        break;
                    }
                } else {
                    let result = run(line.as_str(), &mut state.bindings);
                    match result {
                        Ok(result) => println!("{result}"),
                        Err(err) => println!("Error: {err:?}"),
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("Good Bye!");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("Good Bye!");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
    rl.save_history("history.txt")
}

fn handle_command<T: Clone + PartialEq>(input: &str, instance: &mut REPLState<T>) -> bool {
    let mut parts = input.split_whitespace();
    match parts.next().unwrap() {
        //":remote" => todo!(),
        //":local" => todo!(),
        ":status" | ":s" => status(),
        ":quit" | ":q" => quit(),
        ":bindings" | ":b" => bindings(&instance.bindings),
        ":environment" | ":e" => environment(&mut instance.bindings),
        ":help" | ":h" => help(),
        ":broadcast" => broadcast(input),
        _ => todo!(),
    }
}

fn broadcast(_input: &str) -> bool {
    true
}

fn bindings<T: Clone + PartialEq>(bindings: &Bindings<T>) -> bool {
    bindings
        .bound_names()
        .iter()
        .for_each(|binding| println!("{binding}"));
    true
}

fn environment<T: Clone + PartialEq>(bindings: &mut Bindings<T>) -> bool {
    let mut display: Vec<EnvironmentDisplay> = bindings
        .environment()
        .into_iter()
        .map(EnvironmentDisplay::from)
        .collect();
    display.sort();
    let mut table = Table::new(display);
    table
        .with(
            Modify::new(Rows::new(1..))
                .with(Width::wrap(30).keep_words())
                .with(Width::increase(20)),
        )
        .with(Width::increase(150));
    println!("{table}");
    true
}

fn help() -> bool {
    true
}

fn status() -> bool {
    println!("...");
    true
}

fn quit() -> bool {
    false
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Tabled)]
pub struct EnvironmentDisplay {
    pub name: String,
    pub parameters: String,
    pub result: String,
    pub doc_string: String,
}

impl From<HostFunctionBinding> for EnvironmentDisplay {
    fn from(value: HostFunctionBinding) -> Self {
        EnvironmentDisplay {
            name: value.name,
            parameters: format!("{:?}", value.parameters),
            result: format!("{:?}", value.result),
            doc_string: value.doc_string,
        }
    }
}
