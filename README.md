# lox
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

    $ cd lox-wasm
    $ make build
    $ ...
    $ cd ../lox-vsce
    $ vsce package


## Installing

    $ code --install-extension lox-vsce-<version>.vsix

## Uninstalling

    $ code --code --uninstall-extension undefined_publisher.lox-vsce



