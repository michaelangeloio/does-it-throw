# Changelog

## [0.5.0](https://github.com/michaelangeloio/does-it-throw/compare/does-it-throw-vscode-v0.4.0...does-it-throw-vscode-v0.5.0) (2024-01-25)


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

## [0.4.0](https://github.com/michaelangeloio/does-it-throw/compare/does-it-throw-vscode-v0.3.3...does-it-throw-vscode-v0.4.0) (2024-01-06)


### Features

* **deps-dev:** bump esbuild from 0.19.10 to 0.19.11 ([#112](https://github.com/michaelangeloio/does-it-throw/issues/112)) ([15a95e2](https://github.com/michaelangeloio/does-it-throw/commit/15a95e2f6862b46fe1b6a80270b7e7a694427e5c))
* **deps-dev:** bump mockall from 0.12.0 to 0.12.1 ([#101](https://github.com/michaelangeloio/does-it-throw/issues/101)) ([211b21f](https://github.com/michaelangeloio/does-it-throw/commit/211b21f402d94b8bbf26e92b0185aeb5b1b0d196))
* **deps-dev:** bump typescript from 5.2.2 to 5.3.3 ([#102](https://github.com/michaelangeloio/does-it-throw/issues/102)) ([5f45eff](https://github.com/michaelangeloio/does-it-throw/commit/5f45eff8493f674470331c252bdfc2f558d96c3f))
* **deps:** bump serde_json from 1.0.108 to 1.0.109 ([#109](https://github.com/michaelangeloio/does-it-throw/issues/109)) ([5a28b3f](https://github.com/michaelangeloio/does-it-throw/commit/5a28b3f26992c4bca9d7bb276efdd27fa5b9a53a))
* **deps:** bump swc_ecma_parser from 0.141.33 to 0.141.34 ([#110](https://github.com/michaelangeloio/does-it-throw/issues/110)) ([482be78](https://github.com/michaelangeloio/does-it-throw/commit/482be78a20732f350377d4e534afae1053080e58))
* initial jetbrains/intellij support ([#104](https://github.com/michaelangeloio/does-it-throw/issues/104)) ([455d763](https://github.com/michaelangeloio/does-it-throw/commit/455d7635128646c57bbbc5811b75a526cb8adc64))
* user can now discard warnings with ignore statements ([#118](https://github.com/michaelangeloio/does-it-throw/issues/118)) ([3f8957c](https://github.com/michaelangeloio/does-it-throw/commit/3f8957c60fd90f9ab7b6646c04ec22dcecb21556))


### Bug Fixes

* adjust readme to adhere to jetbrains marketplace guidelines ([#115](https://github.com/michaelangeloio/does-it-throw/issues/115)) ([6d68139](https://github.com/michaelangeloio/does-it-throw/commit/6d68139151f43f06033fd4517baee5c3d53e287c))
* jetbrains build.gradle readme parsing logic ([#116](https://github.com/michaelangeloio/does-it-throw/issues/116)) ([a3cb052](https://github.com/michaelangeloio/does-it-throw/commit/a3cb052b5ac1db2dd8bdbda23eabb37a48de1bfa))
* release please setup for jetbrains ([#107](https://github.com/michaelangeloio/does-it-throw/issues/107)) ([df6b9bb](https://github.com/michaelangeloio/does-it-throw/commit/df6b9bba97d79c1bf0cdda6d306403cd2cd8707e))
* rename jetbrains release-please package name ([#108](https://github.com/michaelangeloio/does-it-throw/issues/108)) ([92791a2](https://github.com/michaelangeloio/does-it-throw/commit/92791a2abc7f29f3087460229f6c5a4ee93c62dc))

## [0.3.3](https://github.com/michaelangeloio/does-it-throw/compare/does-it-throw-vscode-v0.3.2...does-it-throw-vscode-v0.3.3) (2023-12-24)


### Bug Fixes

* catch with throw statement not included ([#95](https://github.com/michaelangeloio/does-it-throw/issues/95)) ([fd223db](https://github.com/michaelangeloio/does-it-throw/commit/fd223db4f56e87439999b9b33a393769bd2b7c5b))
* **deps-dev:** remove remaining eslint dev dependencies ([#97](https://github.com/michaelangeloio/does-it-throw/issues/97)) ([5f173a6](https://github.com/michaelangeloio/does-it-throw/commit/5f173a69cb86570a526a665d453b86ae776538d0))
* remove unused tsx, user-home, semver ([#100](https://github.com/michaelangeloio/does-it-throw/issues/100)) ([de8218c](https://github.com/michaelangeloio/does-it-throw/commit/de8218ce72e01d0092fc03141b26f44d28d5fc1b))

## [0.3.2](https://github.com/michaelangeloio/does-it-throw/compare/does-it-throw-vscode-v0.3.1...does-it-throw-vscode-v0.3.2) (2023-12-17)


### Bug Fixes

* add missing unit test for try statement ([#88](https://github.com/michaelangeloio/does-it-throw/issues/88)) ([290a323](https://github.com/michaelangeloio/does-it-throw/commit/290a323bae194d293ff8d0c826738f72dfef6212))
* gifs not populating in vscode marketplace ([#85](https://github.com/michaelangeloio/does-it-throw/issues/85)) ([15a93d7](https://github.com/michaelangeloio/does-it-throw/commit/15a93d70c94e7de3139e79516fbe43a31701dfa6))
* update server package.json keywords ([#87](https://github.com/michaelangeloio/does-it-throw/issues/87)) ([c19717d](https://github.com/michaelangeloio/does-it-throw/commit/c19717d96a09152d959bfd7d5c3a34ac62f5e26d))

## [0.3.1](https://github.com/michaelangeloio/does-it-throw/compare/does-it-throw-vscode-v0.3.0...does-it-throw-vscode-v0.3.1) (2023-12-17)


### Bug Fixes

* functions and throw statements are underlined even if caught ([#81](https://github.com/michaelangeloio/does-it-throw/issues/81)) ([16adf85](https://github.com/michaelangeloio/does-it-throw/commit/16adf85b05b92542fa6c09ac1611dd56c7603c99))

## [0.3.0](https://github.com/michaelangeloio/does-it-throw/compare/does-it-throw-vscode-v0.2.5...does-it-throw-vscode-v0.3.0) (2023-12-16)


### Features

* **deps:** bump vscode-languageclient from 8.1.0 to 9.0.1 ([#49](https://github.com/michaelangeloio/does-it-throw/issues/49)) ([b36a86b](https://github.com/michaelangeloio/does-it-throw/commit/b36a86b22757568dbfa82817d06956e5a15e0f65))
* neovim support ([#78](https://github.com/michaelangeloio/does-it-throw/issues/78)) ([6152786](https://github.com/michaelangeloio/does-it-throw/commit/61527869e70f54e99616375f7efd53b24e0fa01a))
* neovim/language server docs ([#80](https://github.com/michaelangeloio/does-it-throw/issues/80)) ([a00d92a](https://github.com/michaelangeloio/does-it-throw/commit/a00d92a3b13252025495dc811b21f84df3a38201))


### Bug Fixes

* add biome for standardization, ensure the builder reports errors correctly ([#72](https://github.com/michaelangeloio/does-it-throw/issues/72)) ([0d18392](https://github.com/michaelangeloio/does-it-throw/commit/0d18392268516abb79d015f90495dd331e7ef998))
* bump swc_ecma_ast to 0.110.9 ([#75](https://github.com/michaelangeloio/does-it-throw/issues/75)) ([0aa2e91](https://github.com/michaelangeloio/does-it-throw/commit/0aa2e91f4f1c0b9e352d052382c5a7f436cffeb9))
* remove unused eslint ([#74](https://github.com/michaelangeloio/does-it-throw/issues/74)) ([58ef6ae](https://github.com/michaelangeloio/does-it-throw/commit/58ef6aea9d4334eb0c42901c826ba69157994f77))
* results should still show even if file cannot resolve (calls to throws) ([#76](https://github.com/michaelangeloio/does-it-throw/issues/76)) ([f908556](https://github.com/michaelangeloio/does-it-throw/commit/f908556dfda8eca9195c87269fac71bc6d3e8bf9))

## [0.2.5](https://github.com/michaelangeloio/does-it-throw/compare/does-it-throw-vscode-v0.2.4...does-it-throw-vscode-v0.2.5) (2023-11-10)


### Bug Fixes

* add coverage for switch statements ([#43](https://github.com/michaelangeloio/does-it-throw/issues/43)) ([99fda18](https://github.com/michaelangeloio/does-it-throw/commit/99fda183a7ca813cbb5f5434f429cd79b594f139))
* re-organize primary crate into modules ([#42](https://github.com/michaelangeloio/does-it-throw/issues/42)) ([badb106](https://github.com/michaelangeloio/does-it-throw/commit/badb1061d0dfc679458d55609e43cccfdca01794))
* update details, fix logic in some call expressions, including spread operators ([#40](https://github.com/michaelangeloio/does-it-throw/issues/40)) ([cdfdf47](https://github.com/michaelangeloio/does-it-throw/commit/cdfdf47a2d657364abc1b3b3ce97e89405b842b3))
* update readme and contributing ([#44](https://github.com/michaelangeloio/does-it-throw/issues/44)) ([cf258cd](https://github.com/michaelangeloio/does-it-throw/commit/cf258cd8baffb9277a8039cdb7416378691d6684))

## [0.2.4](https://github.com/michaelangeloio/does-it-throw/compare/does-it-throw-vscode-v0.2.3...does-it-throw-vscode-v0.2.4) (2023-11-09)


### Bug Fixes

* update displayname ([#37](https://github.com/michaelangeloio/does-it-throw/issues/37)) ([cbaa0ad](https://github.com/michaelangeloio/does-it-throw/commit/cbaa0ad7a151559807985f6a4fde0dbd528cdd8a))

## [0.2.3](https://github.com/michaelangeloio/does-it-throw/compare/does-it-throw-vscode-v0.2.2...does-it-throw-vscode-v0.2.3) (2023-11-09)


### Bug Fixes

* release test 22 ([#35](https://github.com/michaelangeloio/does-it-throw/issues/35)) ([73becad](https://github.com/michaelangeloio/does-it-throw/commit/73becad3667a11ce65898843c050771d6a2a0d94))

## [0.2.2](https://github.com/michaelangeloio/does-it-throw/compare/does-it-throw-vscode-v0.2.1...does-it-throw-vscode-v0.2.2) (2023-11-09)


### Bug Fixes

* release test 21 ([#33](https://github.com/michaelangeloio/does-it-throw/issues/33)) ([3c04f87](https://github.com/michaelangeloio/does-it-throw/commit/3c04f87ffdebf63e4f274d107610507fc45edd04))

## [0.2.1](https://github.com/michaelangeloio/does-it-throw/compare/does-it-throw-vscode-v0.2.0...does-it-throw-vscode-v0.2.1) (2023-11-09)


### Bug Fixes

* move release to tag event ([#31](https://github.com/michaelangeloio/does-it-throw/issues/31)) ([082713a](https://github.com/michaelangeloio/does-it-throw/commit/082713afecc40c0d2bc230ffab22e1527298a54c))

## [0.2.0](https://github.com/michaelangeloio/does-it-throw/compare/does-it-throw-vscode-v0.1.6...does-it-throw-vscode-v0.2.0) (2023-11-09)


### Features

* try release please manifest ([a8b6e14](https://github.com/michaelangeloio/does-it-throw/commit/a8b6e14dfbf4cc3c13baa84d9570d0421ca804b1))


### Bug Fixes

* add plugins ([22ed677](https://github.com/michaelangeloio/does-it-throw/commit/22ed6770f4cd4b4805351746768a46c83400e7a3))
* proper release manifest filename ([60cbfae](https://github.com/michaelangeloio/does-it-throw/commit/60cbfaee9f01e4aa12478f12559f9d05890cb232))
* testing release ([#26](https://github.com/michaelangeloio/does-it-throw/issues/26)) ([fe1bea4](https://github.com/michaelangeloio/does-it-throw/commit/fe1bea48ac278d2d4fa23aba775e9ea5fd51c59a))
* try updated config ([aeaa47f](https://github.com/michaelangeloio/does-it-throw/commit/aeaa47f6b9c7ecfed85187523478258ca5900217))
* try updated config again ([995dc18](https://github.com/michaelangeloio/does-it-throw/commit/995dc18dd10a0c816d6b34d621e765655a8e4ed7))
* try updating action ([d71bbae](https://github.com/michaelangeloio/does-it-throw/commit/d71bbaea624f9031d90a4a26f37ff0d2b4888042))
* try updating action again ([83dec44](https://github.com/michaelangeloio/does-it-throw/commit/83dec44c31dfd1f5603587e01b7ed09e87cdbc8e))
* try updating manifest ([ee86305](https://github.com/michaelangeloio/does-it-throw/commit/ee86305f424fa3d300c144305fd1e963f4c2084c))
* update action ([2269fe5](https://github.com/michaelangeloio/does-it-throw/commit/2269fe5c92787c6685f7fe8309afdb876064a888))
* update action ([843e77c](https://github.com/michaelangeloio/does-it-throw/commit/843e77c393db15569d8fa22c016a03f1a0ac78c1))
