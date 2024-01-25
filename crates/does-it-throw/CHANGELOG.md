# Changelog

## [0.3.0](https://github.com/michaelangeloio/does-it-throw/compare/does-it-throw-v0.2.0...does-it-throw-v0.3.0) (2024-01-25)


### Features

* **deps:** bump swc_ecma_ast from 0.110.17 to 0.111.1 ([2ae9f64](https://github.com/michaelangeloio/does-it-throw/commit/2ae9f64e149a74502547ed60e1b4737518844b4b))
* **deps:** bump swc_ecma_parser from 0.141.34 to 0.142.1 ([2ae9f64](https://github.com/michaelangeloio/does-it-throw/commit/2ae9f64e149a74502547ed60e1b4737518844b4b))
* **deps:** bump swc_ecma_visit from 0.96 to 0.97.1 ([2ae9f64](https://github.com/michaelangeloio/does-it-throw/commit/2ae9f64e149a74502547ed60e1b4737518844b4b))


### Bug Fixes

* enhance call to throw logic and handle return statements  ([#140](https://github.com/michaelangeloio/does-it-throw/issues/140)) ([a1bfaf1](https://github.com/michaelangeloio/does-it-throw/commit/a1bfaf16c768aeb49ecaecb991ca6a2b57e71072))

## [0.2.0](https://github.com/michaelangeloio/does-it-throw/compare/does-it-throw-v0.1.15...does-it-throw-v0.2.0) (2024-01-06)


### Features

* user can now discard warnings with ignore statements ([#118](https://github.com/michaelangeloio/does-it-throw/issues/118)) ([3f8957c](https://github.com/michaelangeloio/does-it-throw/commit/3f8957c60fd90f9ab7b6646c04ec22dcecb21556))

## [0.1.15](https://github.com/michaelangeloio/does-it-throw/compare/does-it-throw-v0.1.14...does-it-throw-v0.1.15) (2023-12-24)


### Bug Fixes

* catch with throw statement not included ([#95](https://github.com/michaelangeloio/does-it-throw/issues/95)) ([fd223db](https://github.com/michaelangeloio/does-it-throw/commit/fd223db4f56e87439999b9b33a393769bd2b7c5b))

## [0.1.14](https://github.com/michaelangeloio/does-it-throw/compare/does-it-throw-v0.1.13...does-it-throw-v0.1.14) (2023-12-17)


### Bug Fixes

* add missing unit test for try statement ([#88](https://github.com/michaelangeloio/does-it-throw/issues/88)) ([290a323](https://github.com/michaelangeloio/does-it-throw/commit/290a323bae194d293ff8d0c826738f72dfef6212))

## [0.1.13](https://github.com/michaelangeloio/does-it-throw/compare/does-it-throw-v0.1.12...does-it-throw-v0.1.13) (2023-12-17)


### Bug Fixes

* functions and throw statements are underlined even if caught ([#81](https://github.com/michaelangeloio/does-it-throw/issues/81)) ([16adf85](https://github.com/michaelangeloio/does-it-throw/commit/16adf85b05b92542fa6c09ac1611dd56c7603c99))

## [0.1.12](https://github.com/michaelangeloio/does-it-throw/compare/does-it-throw-v0.1.11...does-it-throw-v0.1.12) (2023-12-16)


### Bug Fixes

* add biome for standardization, ensure the builder reports errors correctly ([#72](https://github.com/michaelangeloio/does-it-throw/issues/72)) ([0d18392](https://github.com/michaelangeloio/does-it-throw/commit/0d18392268516abb79d015f90495dd331e7ef998))
* results should still show even if file cannot resolve (calls to throws) ([#76](https://github.com/michaelangeloio/does-it-throw/issues/76)) ([f908556](https://github.com/michaelangeloio/does-it-throw/commit/f908556dfda8eca9195c87269fac71bc6d3e8bf9))

## [0.1.11](https://github.com/michaelangeloio/does-it-throw/compare/does-it-throw-v0.1.10...does-it-throw-v0.1.11) (2023-11-10)


### Bug Fixes

* add coverage for switch statements ([#43](https://github.com/michaelangeloio/does-it-throw/issues/43)) ([99fda18](https://github.com/michaelangeloio/does-it-throw/commit/99fda183a7ca813cbb5f5434f429cd79b594f139))
* re-organize primary crate into modules ([#42](https://github.com/michaelangeloio/does-it-throw/issues/42)) ([badb106](https://github.com/michaelangeloio/does-it-throw/commit/badb1061d0dfc679458d55609e43cccfdca01794))
* update details, fix logic in some call expressions, including spread operators ([#40](https://github.com/michaelangeloio/does-it-throw/issues/40)) ([cdfdf47](https://github.com/michaelangeloio/does-it-throw/commit/cdfdf47a2d657364abc1b3b3ce97e89405b842b3))

## [0.1.10](https://github.com/michaelangeloio/does-it-throw/compare/does-it-throw-v0.1.9...does-it-throw-v0.1.10) (2023-11-09)


### Bug Fixes

* release test 22 ([#35](https://github.com/michaelangeloio/does-it-throw/issues/35)) ([73becad](https://github.com/michaelangeloio/does-it-throw/commit/73becad3667a11ce65898843c050771d6a2a0d94))

## [0.1.9](https://github.com/michaelangeloio/does-it-throw/compare/does-it-throw-v0.1.8...does-it-throw-v0.1.9) (2023-11-09)


### Bug Fixes

* release test 21 ([#33](https://github.com/michaelangeloio/does-it-throw/issues/33)) ([3c04f87](https://github.com/michaelangeloio/does-it-throw/commit/3c04f87ffdebf63e4f274d107610507fc45edd04))

## [0.1.8](https://github.com/michaelangeloio/does-it-throw/compare/does-it-throw-v0.1.7...does-it-throw-v0.1.8) (2023-11-09)


### Bug Fixes

* move release to tag event ([#31](https://github.com/michaelangeloio/does-it-throw/issues/31)) ([082713a](https://github.com/michaelangeloio/does-it-throw/commit/082713afecc40c0d2bc230ffab22e1527298a54c))

## [0.1.7](https://github.com/michaelangeloio/does-it-throw/compare/does-it-throw-v0.1.6...does-it-throw-v0.1.7) (2023-11-09)


### Bug Fixes

* try updated config again ([995dc18](https://github.com/michaelangeloio/does-it-throw/commit/995dc18dd10a0c816d6b34d621e765655a8e4ed7))
