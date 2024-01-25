# Changelog

## Unreleased

## [0.5.0](https://github.com/michaelangeloio/does-it-throw/compare/does-it-throw-jetbrains-v0.4.0...does-it-throw-jetbrains-v0.5.0) (2024-01-25)


### Features

* **deps-dev:** bump @biomejs/biome from 1.4.1 to 1.5.3 ([ea29908](https://github.com/michaelangeloio/does-it-throw/commit/ea29908c94253fc3171738f7fc802a71f9166d75))
* **deps-dev:** bump @types/node from 20.10.5 to 20.11.5 ([ea29908](https://github.com/michaelangeloio/does-it-throw/commit/ea29908c94253fc3171738f7fc802a71f9166d75))
* **deps-dev:** bump @types/node from 20.10.5 to 20.11.6 ([ea29908](https://github.com/michaelangeloio/does-it-throw/commit/ea29908c94253fc3171738f7fc802a71f9166d75))
* **deps-dev:** bump bun-types from 1.0.20 to 1.0.25 ([ea29908](https://github.com/michaelangeloio/does-it-throw/commit/ea29908c94253fc3171738f7fc802a71f9166d75))
* **deps-dev:** bump esbuild from 0.19.11 to 0.19.12 ([ea29908](https://github.com/michaelangeloio/does-it-throw/commit/ea29908c94253fc3171738f7fc802a71f9166d75))
* **deps:** bump serde from 1.0.193 to 1.0.195 ([ea29908](https://github.com/michaelangeloio/does-it-throw/commit/ea29908c94253fc3171738f7fc802a71f9166d75))
* **deps:** bump serde_json from 1.0.109 to 1.0.111 ([ea29908](https://github.com/michaelangeloio/does-it-throw/commit/ea29908c94253fc3171738f7fc802a71f9166d75))
* **deps:** bump swc_ecma_ast from 0.110.15 to 0.110.17 ([ea29908](https://github.com/michaelangeloio/does-it-throw/commit/ea29908c94253fc3171738f7fc802a71f9166d75))
* **deps:** bump swc_ecma_ast from 0.110.17 to 0.111.1 ([2ae9f64](https://github.com/michaelangeloio/does-it-throw/commit/2ae9f64e149a74502547ed60e1b4737518844b4b))
* **deps:** bump swc_ecma_parser from 0.141.34 to 0.142.1 ([2ae9f64](https://github.com/michaelangeloio/does-it-throw/commit/2ae9f64e149a74502547ed60e1b4737518844b4b))
* **deps:** bump swc_ecma_visit from 0.96 to 0.97.1 ([2ae9f64](https://github.com/michaelangeloio/does-it-throw/commit/2ae9f64e149a74502547ed60e1b4737518844b4b))
* **deps:** bump swc_ecma_visit from 0.96.15 to 0.96.17 ([ea29908](https://github.com/michaelangeloio/does-it-throw/commit/ea29908c94253fc3171738f7fc802a71f9166d75))
* **deps:** bump vscode-languageserver-textdocument from 1.0.8 to 1.0.11 ([ea29908](https://github.com/michaelangeloio/does-it-throw/commit/ea29908c94253fc3171738f7fc802a71f9166d75))
* **deps:** bump wasm-bindgen from 0.2.89 to 0.2.90 ([ea29908](https://github.com/michaelangeloio/does-it-throw/commit/ea29908c94253fc3171738f7fc802a71f9166d75))
* **jetbrains:** jetbrains implementation ([#119](https://github.com/michaelangeloio/does-it-throw/issues/119)) ([e4d4153](https://github.com/michaelangeloio/does-it-throw/commit/e4d415336da8eb78ef650f2941185a3fa4dc5dd6))
* LSP now supports workspace folders ([#127](https://github.com/michaelangeloio/does-it-throw/issues/127)) ([960b486](https://github.com/michaelangeloio/does-it-throw/commit/960b486e8cfe4fd5165be4dd200457c7e5b90979))


### Bug Fixes

* enhance call to throw logic and handle return statements  ([#140](https://github.com/michaelangeloio/does-it-throw/issues/140)) ([a1bfaf1](https://github.com/michaelangeloio/does-it-throw/commit/a1bfaf16c768aeb49ecaecb991ca6a2b57e71072))

## [0.4.0](https://github.com/michaelangeloio/does-it-throw/compare/does-it-throw-jetbrains-v0.3.3...does-it-throw-jetbrains-v0.4.0) (2024-01-06)


### Features

* initial jetbrains/intellij support ([#104](https://github.com/michaelangeloio/does-it-throw/issues/104)) ([455d763](https://github.com/michaelangeloio/does-it-throw/commit/455d7635128646c57bbbc5811b75a526cb8adc64))


### Bug Fixes

* adjust readme to adhere to jetbrains marketplace guidelines ([#115](https://github.com/michaelangeloio/does-it-throw/issues/115)) ([6d68139](https://github.com/michaelangeloio/does-it-throw/commit/6d68139151f43f06033fd4517baee5c3d53e287c))
* jetbrains build.gradle readme parsing logic ([#116](https://github.com/michaelangeloio/does-it-throw/issues/116)) ([a3cb052](https://github.com/michaelangeloio/does-it-throw/commit/a3cb052b5ac1db2dd8bdbda23eabb37a48de1bfa))
* release please setup for jetbrains ([#107](https://github.com/michaelangeloio/does-it-throw/issues/107)) ([df6b9bb](https://github.com/michaelangeloio/does-it-throw/commit/df6b9bba97d79c1bf0cdda6d306403cd2cd8707e))

## 0.3.3(https://github.com/michaelangeloio/does-it-throw/compare/does-it-throw-lsp-v0.3.2...does-it-throw-lsp-v0.3.3) (2023-12-24)

### Bug Fixes

- catch with throw statement not included ([#95](https://github.com/michaelangeloio/does-it-throw/issues/95)) ([fd223db](https://github.com/michaelangeloio/does-it-throw/commit/fd223db4f56e87439999b9b33a393769bd2b7c5b))
- **deps-dev:** remove remaining eslint dev dependencies ([#97](https://github.com/michaelangeloio/does-it-throw/issues/97)) ([5f173a6](https://github.com/michaelangeloio/does-it-throw/commit/5f173a69cb86570a526a665d453b86ae776538d0))

## 0.3.2(https://github.com/michaelangeloio/does-it-throw/compare/does-it-throw-lsp-v0.3.1...does-it-throw-lsp-v0.3.2) (2023-12-17)

### Bug Fixes

- update server package.json keywords ([#87](https://github.com/michaelangeloio/does-it-throw/issues/87)) ([c19717d](https://github.com/michaelangeloio/does-it-throw/commit/c19717d96a09152d959bfd7d5c3a34ac62f5e26d))

## 0.3.1(https://github.com/michaelangeloio/does-it-throw/compare/does-it-throw-lsp-v0.3.0...does-it-throw-lsp-v0.3.1) (2023-12-17)

### Bug Fixes

- functions and throw statements are underlined even if caught ([#81](https://github.com/michaelangeloio/does-it-throw/issues/81)) ([16adf85](https://github.com/michaelangeloio/does-it-throw/commit/16adf85b05b92542fa6c09ac1611dd56c7603c99))

## 0.3.0(https://github.com/michaelangeloio/does-it-throw/compare/does-it-throw-lsp-v0.2.5...does-it-throw-lsp-v0.3.0) (2023-12-16)

### Features

- neovim support ([#78](https://github.com/michaelangeloio/does-it-throw/issues/78)) ([6152786](https://github.com/michaelangeloio/does-it-throw/commit/61527869e70f54e99616375f7efd53b24e0fa01a))

### Bug Fixes

- add biome for standardization, ensure the builder reports errors correctly ([#72](https://github.com/michaelangeloio/does-it-throw/issues/72)) ([0d18392](https://github.com/michaelangeloio/does-it-throw/commit/0d18392268516abb79d015f90495dd331e7ef998))
- re-organize primary crate into modules ([#42](https://github.com/michaelangeloio/does-it-throw/issues/42)) ([badb106](https://github.com/michaelangeloio/does-it-throw/commit/badb1061d0dfc679458d55609e43cccfdca01794))
- results should still show even if file cannot resolve (calls to throws) ([#76](https://github.com/michaelangeloio/does-it-throw/issues/76)) ([f908556](https://github.com/michaelangeloio/does-it-throw/commit/f908556dfda8eca9195c87269fac71bc6d3e8bf9))
- update details, fix logic in some call expressions, including spread operators ([#40](https://github.com/michaelangeloio/does-it-throw/issues/40)) ([cdfdf47](https://github.com/michaelangeloio/does-it-throw/commit/cdfdf47a2d657364abc1b3b3ce97e89405b842b3))
