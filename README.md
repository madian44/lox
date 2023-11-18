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

### Current

Chapter 4 Scanning
