# Lox

From Crafting Interpreters

An implementation of lox from [Crafting Interpreters](https://craftinginterpreters.com/) as a visual studio code extension implemented in rust (wasm).

## Pre-requisites

- rust toolchain (https://rustwasm.github.io/docs/book/game-of-life/setup.html) (rust and wasm-pack at least)
- node (https://github.com/nvm-sh/nvm)
- vcse (https://code.visualstudio.com/api/working-with-extensions/publishing-extension#vsce)
- make

## Building

Install vsce dependencies

    $ cd lox-vsce
    $ npm install

Pretty basic at the moment:

    $ cd lox-vsce
    $ make build

For coverage `grcov` and `llvm-tools-preview` are required:

    $ cargo install grcov
    $ rustup component add llvm-tools-preview 

## Installing

    $ code --install-extension lox-vsce-<version>.vsix

## Uninstalling

    $ code --code --uninstall-extension undefined_publisher.lox-vsce

## Projects

### lox

`rust` implementation of Lox.

### lox-wasm

`wasm` build of `lox` plus mapping to `vsce`.

### lox-vsce

Typescript `vsce` extension.

## Progress through the book...

### Implemented

 * Chapter 4 Scanning
 * Chapter 5 Representing Code
 * Chapter 6 Parsing Expressions
 * Chapter 7 Evaluating Expressions
 * Chapter 8 Statements and State
 * Chapter 9 Control Flow
 * Chapter 10 Functions
 * Chapter 11 Resolving and Binding
 * Chapter 12 Classes
