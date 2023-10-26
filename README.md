# Wander

Wander is a programming language designed for extending applications.
Its main use case is to be used as a library by another application (or library)
instead of being used to compile to native code or bytecode like Wasm or the JVM or
be ran as a script directly like Ruby or Python usually are.

It does this by focusing on several areas:

 * Embeddability, Wander is intended to be ran from inside of another program, this is the main use case for the language
 * Size, Wander has a small core that is designed to be flexible, this also helps with embedding both in terms of physical size and ease of interfacing with other programming languages
 * Portability, Wander already partially supports being integrated into projects using Rust, JVM, .NET, JS, or Wasm
 * Dynamicity, Wander tries to combine it's type system with runtime dynamicity in a way that is productive and will help catch common errors
 * Usability, Wander tries to lower as many barriers to entry for beginners while allowing experienced users a familar toolset

## Status

This project is very new, so expect a lot of breaking changes during design and experimentation.

## Documentation

See https://wander-lang.dev/docs/

## Crates in this repo

| Name        | Description                                                                    | Output   |
| ----------- | ------------------------------------------------------------------------------ | -------- |
| wander      | The main crate that includes the parser + data model + interpreter for Wander. | lib      |
| wander-repl | A basic REPL for Wander based on rustyline.                                    | bin      |
| wander-pad  | A simple GTK4 based desktop application for working with Wander.               | bin      |
| wander-wasm | wasm-bindgen support for Wander.                                               | wasm/npm |

## Related projects

| Name                                                             | Description                                       |
| ---------------------------------------------------------------- | ------------------------------------------------- |
| [wander-lang.dev](https://github.com/almibe/wander-lang.dev)     | The main website + documentation for Wander.      |
| [wander-components](https://github.com/almibe/wander-components) | A set of web components for working with Wander.  |
| [wander-ide](https://github.com/almibe/wander-ide)               | An Integrated Development Environment for Wander. |
