# Changelog

## [6.1.0](https://github.com/hhanh00/zkool2/compare/zkool-v6.0.0...zkool-v6.1.0) (2025-09-25)


### Features

* navigation buttons on tx details ([#556](https://github.com/hhanh00/zkool2/issues/556)) ([530f67a](https://github.com/hhanh00/zkool2/commit/530f67a30d23a6d1d05a66ed13977d59ce5e2744))


### Bug Fixes

* edit category ([#560](https://github.com/hhanh00/zkool2/issues/560)) ([4e68483](https://github.com/hhanh00/zkool2/commit/4e684831db200847a296f192f7067b2b3c5c3c81))
* reset account should remove all tx data ([#559](https://github.com/hhanh00/zkool2/issues/559)) ([4d8c550](https://github.com/hhanh00/zkool2/commit/4d8c5500694775d64df6a238f910dc91587d986b))
* separate list of categories by income/expense ([#561](https://github.com/hhanh00/zkool2/issues/561)) ([3e670bc](https://github.com/hhanh00/zkool2/commit/3e670bc44a6138a8fc86c977ee61481c7cebf8e2))

## [6.0.0](https://github.com/hhanh00/zkool2/compare/zkool-v5.2.6...zkool-v6.0.0) (2025-09-23)


### ⚠ BREAKING CHANGES

* db schema for categories and transaction value in fiat ([#524](https://github.com/hhanh00/zkool2/issues/524))

### Features

* add category to send form ([#537](https://github.com/hhanh00/zkool2/issues/537)) ([cd15e32](https://github.com/hhanh00/zkool2/commit/cd15e321f3099a55f389ae36f5951edc964b66f8))
* add default categories ([#535](https://github.com/hhanh00/zkool2/issues/535)) ([14d9f14](https://github.com/hhanh00/zkool2/commit/14d9f147b9088f3a2473f4984aed9781247e4c0a))
* allow manual input of fx rate ([#526](https://github.com/hhanh00/zkool2/issues/526)) ([48f8a61](https://github.com/hhanh00/zkool2/commit/48f8a617f060c61456b446835c493198aea96704))
* category chart ([#540](https://github.com/hhanh00/zkool2/issues/540)) ([0076cce](https://github.com/hhanh00/zkool2/commit/0076cce652b1d51b1b1fa32370c870e1d8c01456))
* category chart ([#544](https://github.com/hhanh00/zkool2/issues/544)) ([3e0d41a](https://github.com/hhanh00/zkool2/commit/3e0d41afe638163505aeb45b7455231ad17a02e3))
* category editor ([#532](https://github.com/hhanh00/zkool2/issues/532)) ([ed37f94](https://github.com/hhanh00/zkool2/commit/ed37f94c4f7287f1d7613e67a7a8bfea014d446d))
* cumulative spending/income by category chart ([#546](https://github.com/hhanh00/zkool2/issues/546)) ([2112c0a](https://github.com/hhanh00/zkool2/commit/2112c0ab71820ca1c7cd0bce5d2b037dfe143987))
* db schema for categories and transaction value in fiat ([#524](https://github.com/hhanh00/zkool2/issues/524)) ([1478f5d](https://github.com/hhanh00/zkool2/commit/1478f5d6fa3c9d6f991f8f977df72d149ef0fb35))
* display fx rate in tx details ([#533](https://github.com/hhanh00/zkool2/issues/533)) ([d8f0d61](https://github.com/hhanh00/zkool2/commit/d8f0d6193d934cea4973e603324e3cc60ffcc0a2))
* edit category in tx details ([#534](https://github.com/hhanh00/zkool2/issues/534)) ([1ffd3f6](https://github.com/hhanh00/zkool2/commit/1ffd3f60c1ba1d121b0e0627e8c6fd82936e14de))
* edit tx price on details page ([#551](https://github.com/hhanh00/zkool2/issues/551)) ([f8ee170](https://github.com/hhanh00/zkool2/commit/f8ee170351f6a6d9709039e6277ce9583dc4c79c))
* fill missing tx prices by querying Coin Gecko ([#530](https://github.com/hhanh00/zkool2/issues/530)) ([e82acd0](https://github.com/hhanh00/zkool2/commit/e82acd030d09010a8d33b0ceb0eb333e9b491731))
* get historical prices from CoinGecko ([#529](https://github.com/hhanh00/zkool2/issues/529)) ([574ea17](https://github.com/hhanh00/zkool2/commit/574ea17909bd53af5acd96a06efe6393c8804ab7))
* reconcile pending tx price/category with real tx ([#538](https://github.com/hhanh00/zkool2/issues/538)) ([73cba46](https://github.com/hhanh00/zkool2/commit/73cba466be2936463555e057e091632d6d8ff6bc))
* retrieve and display tx category ([#531](https://github.com/hhanh00/zkool2/issues/531)) ([e9b792a](https://github.com/hhanh00/zkool2/commit/e9b792a598590b599c05483947571a9d42d58e67))
* save pending tx category & fx rate ([#527](https://github.com/hhanh00/zkool2/issues/527)) ([d8496f9](https://github.com/hhanh00/zkool2/commit/d8496f983d1a8065c277f53045a913cb7502d943))
* save/load categories & tx price to file ([#539](https://github.com/hhanh00/zkool2/issues/539)) ([379b1fb](https://github.com/hhanh00/zkool2/commit/379b1fba6cea24cfe92328abe2a668a3d9cfc7ba))
* spending/income chart ([#541](https://github.com/hhanh00/zkool2/issues/541)) ([32e8ca7](https://github.com/hhanh00/zkool2/commit/32e8ca700c048f56813dfaed59e933f29dd3b1d6))
* tx amount by date scatter chart ([#545](https://github.com/hhanh00/zkool2/issues/545)) ([0759869](https://github.com/hhanh00/zkool2/commit/0759869e5e8d78ea9c5833f73fa54296d358bfc6))


### Bug Fixes

* birth height before sapling activation ([#553](https://github.com/hhanh00/zkool2/issues/553)) ([06479fd](https://github.com/hhanh00/zkool2/commit/06479fdc8138428697aa0c22b88527230e56f3ad))
* chart refresh ([#547](https://github.com/hhanh00/zkool2/issues/547)) ([10b9270](https://github.com/hhanh00/zkool2/commit/10b9270d0bf0a7bc2d13f562992e86a8e9a38672))
* db version check ([#543](https://github.com/hhanh00/zkool2/issues/543)) ([16664f2](https://github.com/hhanh00/zkool2/commit/16664f2079eb44cd25d78132d06b09cd80f4d97c))
* height off by 1 after reset ([#521](https://github.com/hhanh00/zkool2/issues/521)) ([02802fb](https://github.com/hhanh00/zkool2/commit/02802fba3a93df3c5e8ac16f8a20c18633bf4d19))
* reorganize menu items for charts ([#554](https://github.com/hhanh00/zkool2/issues/554)) ([91fe8c8](https://github.com/hhanh00/zkool2/commit/91fe8c85bad50c76ad670c05bd7c88a78a8f8795))
* store block header time ([#523](https://github.com/hhanh00/zkool2/issues/523)) ([fe09e1f](https://github.com/hhanh00/zkool2/commit/fe09e1fbe9d6c33e39d5353e010616ac1ccebb7e))
* typo in db version key name ([#525](https://github.com/hhanh00/zkool2/issues/525)) ([9083a6e](https://github.com/hhanh00/zkool2/commit/9083a6e0a534c55a427c86b88d0b830bd4b28094))
* ui adjustments to chart ([#542](https://github.com/hhanh00/zkool2/issues/542)) ([5fc005e](https://github.com/hhanh00/zkool2/commit/5fc005ee21a8a1ff1f7a612f8ec4ce9f07d7aee6))

## [5.2.6](https://github.com/hhanh00/zkool2/compare/zkool-v5.2.5...zkool-v5.2.6) (2025-09-18)


### Bug Fixes

* block time at birth height for new account ([#518](https://github.com/hhanh00/zkool2/issues/518)) ([68c33cb](https://github.com/hhanh00/zkool2/commit/68c33cbe32e1df04caada98c753077a6fbcdb230))
* dkg - handle error from server ([#515](https://github.com/hhanh00/zkool2/issues/515)) ([f79d340](https://github.com/hhanh00/zkool2/commit/f79d3404c89e329dea2b61ae369650fb80b79d2a))
* duplicate GlobalKey ([#520](https://github.com/hhanh00/zkool2/issues/520)) ([0bb17f2](https://github.com/hhanh00/zkool2/commit/0bb17f243c8b1f979ebc3d7ceeef9ee46f4dcdda))
* get block times of synced points ([#517](https://github.com/hhanh00/zkool2/issues/517)) ([8aa5a51](https://github.com/hhanh00/zkool2/commit/8aa5a5123dccec3c4ac2bab614a94505e8fbc934))
* import account ([f79d340](https://github.com/hhanh00/zkool2/commit/f79d3404c89e329dea2b61ae369650fb80b79d2a))

## [5.2.5](https://github.com/hhanh00/zkool2/compare/zkool-v5.2.4...zkool-v5.2.5) (2025-09-17)


### Bug Fixes

* testnet ([8f06fcf](https://github.com/hhanh00/zkool2/commit/8f06fcff4696a8c1ada83b07296086b34fedd3cc))
* wrong height chosen for witness data, ([#512](https://github.com/hhanh00/zkool2/issues/512)) ([8f06fcf](https://github.com/hhanh00/zkool2/commit/8f06fcff4696a8c1ada83b07296086b34fedd3cc))

## [5.2.4](https://github.com/hhanh00/zkool2/compare/zkool-v5.2.3...zkool-v5.2.4) (2025-09-17)


### Bug Fixes

* linear progress indicator ([#511](https://github.com/hhanh00/zkool2/issues/511)) ([a7bd615](https://github.com/hhanh00/zkool2/commit/a7bd61562ebf13cac6d8b831cfe89d3b9e2bc179))

## [5.2.3](https://github.com/hhanh00/zkool2/compare/zkool-v5.2.2...zkool-v5.2.3) (2025-09-16)


### Bug Fixes

* resize icon ([#505](https://github.com/hhanh00/zkool2/issues/505)) ([e3fb3de](https://github.com/hhanh00/zkool2/commit/e3fb3def77e925f80a5b911e454b1890ffa58394))

## [5.2.2](https://github.com/hhanh00/zkool2/compare/zkool-v5.2.1...zkool-v5.2.2) (2025-09-16)


### Bug Fixes

* add white background to icon ([#503](https://github.com/hhanh00/zkool2/issues/503)) ([1395fb8](https://github.com/hhanh00/zkool2/commit/1395fb8e03dd45d4e35355d17a8abb6a5383bcfa))

## [5.2.1](https://github.com/hhanh00/zkool2/compare/zkool-v5.2.0...zkool-v5.2.1) (2025-09-15)


### Bug Fixes

* send from transparent private key only account ([#501](https://github.com/hhanh00/zkool2/issues/501)) ([cf42f4a](https://github.com/hhanh00/zkool2/commit/cf42f4a60722758e52ad28066fb60b6e62041b3c))

## [5.2.0](https://github.com/hhanh00/zkool2/compare/zkool-v5.1.1...zkool-v5.2.0) (2025-09-15)


### Features

* show block timestamp of account synced height ([#498](https://github.com/hhanh00/zkool2/issues/498)) ([f768251](https://github.com/hhanh00/zkool2/commit/f7682517adf055dc34f6265662b417e33c12a9f4))

## [5.1.1](https://github.com/hhanh00/zkool2/compare/zkool-v5.1.0...zkool-v5.1.1) (2025-09-15)


### Bug Fixes

* windows build ([#496](https://github.com/hhanh00/zkool2/issues/496)) ([59c3f26](https://github.com/hhanh00/zkool2/commit/59c3f26331162af326ae1273d7693661f427b95d))

## [5.1.0](https://github.com/hhanh00/zkool2/compare/zkool-v5.0.5...zkool-v5.1.0) (2025-09-15)


### Features

* show accounts that were sync more than 30 mins ago in red ([#495](https://github.com/hhanh00/zkool2/issues/495)) ([4b1f927](https://github.com/hhanh00/zkool2/commit/4b1f9278eba143fe2c76d581f9bc6c98785a183b))


### Bug Fixes

* change password form has "repeat password" field ([#492](https://github.com/hhanh00/zkool2/issues/492)) ([38fad71](https://github.com/hhanh00/zkool2/commit/38fad71e986a61abf738c2181e0f5d9d8d0c14cc))

## [5.0.5](https://github.com/hhanh00/zkool2/compare/zkool-v5.0.4...zkool-v5.0.5) (2025-09-13)


### Bug Fixes

* android 16k page alignment for rive & camera ([#488](https://github.com/hhanh00/zkool2/issues/488)) ([#490](https://github.com/hhanh00/zkool2/issues/490)) ([6b1e486](https://github.com/hhanh00/zkool2/commit/6b1e48662e2febf1d098990959e7bfb80ebd6df0))

## [5.0.4](https://github.com/hhanh00/zkool2/compare/zkool-v5.0.3...zkool-v5.0.4) (2025-09-13)


### Bug Fixes

* remove unused file ([#486](https://github.com/hhanh00/zkool2/issues/486)) ([f8a8b39](https://github.com/hhanh00/zkool2/commit/f8a8b393d292f8bcda5c7bac4ecd0c90c02c3e13))

## [5.0.3](https://github.com/hhanh00/zkool2/compare/zkool-v5.0.2...zkool-v5.0.3) (2025-09-13)


### Bug Fixes

* update splash icon ([#483](https://github.com/hhanh00/zkool2/issues/483)) ([5144f15](https://github.com/hhanh00/zkool2/commit/5144f152d743e4e2ddc1dcd93cfe30f5ec9a9a2e))

## [5.0.2](https://github.com/hhanh00/zkool2/compare/zkool-v5.0.1...zkool-v5.0.2) (2025-09-12)


### Bug Fixes

* update launcher icon ([#481](https://github.com/hhanh00/zkool2/issues/481)) ([d7ca47d](https://github.com/hhanh00/zkool2/commit/d7ca47d872238c4cf1e029b7b2d544da6135c97f))

## [5.0.1](https://github.com/hhanh00/zkool2/compare/zkool-v5.0.0...zkool-v5.0.1) (2025-09-10)


### Bug Fixes

* error message when tx was broadcast correctly ([#479](https://github.com/hhanh00/zkool2/issues/479)) ([c0d4c8e](https://github.com/hhanh00/zkool2/commit/c0d4c8e9fea82aa778d44d9a5722eb70ece11980))

## [5.0.0](https://github.com/hhanh00/zkool2/compare/zkool-v4.25.2...zkool-v5.0.0) (2025-09-05)


### ⚠ BREAKING CHANGES

* folder and db versioning ([#468](https://github.com/hhanh00/zkool2/issues/468))

### Features

* account folders ([#474](https://github.com/hhanh00/zkool2/issues/474)) ([d203af3](https://github.com/hhanh00/zkool2/commit/d203af3f3eb1f735b348ba6a1808cf43895d7ec4))
* create new folders ([#472](https://github.com/hhanh00/zkool2/issues/472)) ([521d4b9](https://github.com/hhanh00/zkool2/commit/521d4b9d02c141dedfb9c8f513b141d8881001fc))
* folder and db versioning ([#468](https://github.com/hhanh00/zkool2/issues/468)) ([0031c17](https://github.com/hhanh00/zkool2/commit/0031c17194c80a8f3953b9983e20f854d5c062bb))
* folder pop up menu ([#471](https://github.com/hhanh00/zkool2/issues/471)) ([e38e54c](https://github.com/hhanh00/zkool2/commit/e38e54c44fdcd7afec879f91d9b68912d27eee59))
* rename/delete folders ([#473](https://github.com/hhanh00/zkool2/issues/473)) ([2e36975](https://github.com/hhanh00/zkool2/commit/2e36975a88002e9ae43ddb3c4369575ef8947c7d))


### Bug Fixes

* do not add column if it exists ([#470](https://github.com/hhanh00/zkool2/issues/470)) ([bb6794f](https://github.com/hhanh00/zkool2/commit/bb6794f9d7ac1b59aef46b4fb756df8c56c3b821))
* refresh after folder deletion ([#475](https://github.com/hhanh00/zkool2/issues/475)) ([459e32d](https://github.com/hhanh00/zkool2/commit/459e32de0182efdf28b1247de16f1719e22e171c))

## [4.25.2](https://github.com/hhanh00/zkool2/compare/zkool-v4.25.1...zkool-v4.25.2) (2025-09-03)


### Bug Fixes

* switch to rustls for arti-client on macos ([#457](https://github.com/hhanh00/zkool2/issues/457)) ([e9ccf6e](https://github.com/hhanh00/zkool2/commit/e9ccf6e827f10ca90de4f51db6bcfe18a98db944))

## [4.25.1](https://github.com/hhanh00/zkool2/compare/zkool-v4.25.0...zkool-v4.25.1) (2025-09-03)


### Bug Fixes

* observe unconfirmed amount ([#464](https://github.com/hhanh00/zkool2/issues/464)) ([8358c8f](https://github.com/hhanh00/zkool2/commit/8358c8f4840568038666e47b2043fb9f4d7ad927))

## [4.25.0](https://github.com/hhanh00/zkool2/compare/zkool-v4.24.3...zkool-v4.25.0) (2025-09-03)


### Features

* link to block explorer ([#461](https://github.com/hhanh00/zkool2/issues/461)) ([caca27f](https://github.com/hhanh00/zkool2/commit/caca27fdc8da8bf48186edd9e232a1c0d1a6f261))


### Bug Fixes

* set net too early before db loaded ([#463](https://github.com/hhanh00/zkool2/issues/463)) ([de838e0](https://github.com/hhanh00/zkool2/commit/de838e0d5296884ae2503e54fdcf9f511f0c8f59))

## [4.24.3](https://github.com/hhanh00/zkool2/compare/zkool-v4.24.2...zkool-v4.24.3) (2025-09-03)


### Bug Fixes

* lazily build Tor client ([#454](https://github.com/hhanh00/zkool2/issues/454)) ([aa017b8](https://github.com/hhanh00/zkool2/commit/aa017b8a22924407d448ef62411a44f289467c74))
* respond to focus events on address field ([#460](https://github.com/hhanh00/zkool2/issues/460)) ([97fc0d9](https://github.com/hhanh00/zkool2/commit/97fc0d910eb69d6e2b4d8f6d2c95b0dec398bb83))
* use locale for parsing amounts ([#458](https://github.com/hhanh00/zkool2/issues/458)) ([c366370](https://github.com/hhanh00/zkool2/commit/c3663705a5b2c203e9d008538c83310245314b01))

## [4.24.2](https://github.com/hhanh00/zkool2/compare/zkool-v4.24.1...zkool-v4.24.2) (2025-08-31)


### Bug Fixes

* fix typos ([#451](https://github.com/hhanh00/zkool2/issues/451)) ([3e02a70](https://github.com/hhanh00/zkool2/commit/3e02a70c83192a94c8233ad6a170f5dc2b495407))
* progress bar ([#452](https://github.com/hhanh00/zkool2/issues/452)) ([a3bbfa7](https://github.com/hhanh00/zkool2/commit/a3bbfa7a99318588daf817fdb79134798a15c9d8))
* rewind account ([#448](https://github.com/hhanh00/zkool2/issues/448)) ([4f21f15](https://github.com/hhanh00/zkool2/commit/4f21f15cf1682d58d631d47e3d0ea941b3d88e1b))

## [4.24.1](https://github.com/hhanh00/zkool2/compare/zkool-v4.24.0...zkool-v4.24.1) (2025-08-30)


### Bug Fixes

* typo ([#446](https://github.com/hhanh00/zkool2/issues/446)) ([5f3e501](https://github.com/hhanh00/zkool2/commit/5f3e501b46fa8e9049f5b00f0153bfdb376c2fbb))

## [4.24.0](https://github.com/hhanh00/zkool2/compare/zkool-v4.23.0...zkool-v4.24.0) (2025-08-30)


### Features

* add support for NU6.1 on testnet ([#444](https://github.com/hhanh00/zkool2/issues/444)) ([2390ab5](https://github.com/hhanh00/zkool2/commit/2390ab59f09e155b818f94ecb467cf365e793a4b))

## [4.23.0](https://github.com/hhanh00/zkool2/compare/zkool-v4.22.1...zkool-v4.23.0) (2025-08-27)


### Features

* add warning when some notes are disabled ([#442](https://github.com/hhanh00/zkool2/issues/442)) ([e8d42b3](https://github.com/hhanh00/zkool2/commit/e8d42b3b19a7de2e0c05a9277b5328529b8c3977))

## [4.22.1](https://github.com/hhanh00/zkool2/compare/zkool-v4.22.0...zkool-v4.22.1) (2025-08-27)


### Bug Fixes

* amount input widget ([#440](https://github.com/hhanh00/zkool2/issues/440)) ([b553a58](https://github.com/hhanh00/zkool2/commit/b553a580e2975522602179ecd53804f6583d2558))

## [4.22.0](https://github.com/hhanh00/zkool2/compare/zkool-v4.21.1...zkool-v4.22.0) (2025-08-26)


### Features

* enter amount in USD ([#439](https://github.com/hhanh00/zkool2/issues/439)) ([f3d0136](https://github.com/hhanh00/zkool2/commit/f3d01365f686348d3e2e6a62425cddfdc3a6b7fe))


### Bug Fixes

* pinlock + account icon ([#437](https://github.com/hhanh00/zkool2/issues/437)) ([d7227e6](https://github.com/hhanh00/zkool2/commit/d7227e60a05a1b778a3952bf6e37e103c9dbf7b3))

## [4.21.1](https://github.com/hhanh00/zkool2/compare/zkool-v4.21.0...zkool-v4.21.1) (2025-08-25)


### Bug Fixes

* ios build ([#435](https://github.com/hhanh00/zkool2/issues/435)) ([ae44aee](https://github.com/hhanh00/zkool2/commit/ae44aee6e62dbce42af99c36be423165b4e23a1a))

## [4.21.0](https://github.com/hhanh00/zkool2/compare/zkool-v4.20.11...zkool-v4.21.0) (2025-08-25)


### Features

* show confirm dialog when restoring without birth height ([#431](https://github.com/hhanh00/zkool2/issues/431)) ([bbeddbc](https://github.com/hhanh00/zkool2/commit/bbeddbc3b9b89ee7bb7e97cc8516058bf43e3555))


### Bug Fixes

* allow removal of account icon ([#434](https://github.com/hhanh00/zkool2/issues/434)) ([1046bf2](https://github.com/hhanh00/zkool2/commit/1046bf28e8f0982e4b6b78868592d6be3e75eda1))
* dkg error handling ([#433](https://github.com/hhanh00/zkool2/issues/433)) ([2554163](https://github.com/hhanh00/zkool2/commit/25541634e8782975a0cf6a28f42c63de86281220))

## [4.20.11](https://github.com/hhanh00/zkool2/compare/zkool-v4.20.10...zkool-v4.20.11) (2025-08-17)


### Bug Fixes

* database manager button ([#427](https://github.com/hhanh00/zkool2/issues/427)) ([fcdfb60](https://github.com/hhanh00/zkool2/commit/fcdfb60335e548f726983ad8527983a3ef80a0f9))

## [4.20.10](https://github.com/hhanh00/zkool2/compare/zkool-v4.20.9...zkool-v4.20.10) (2025-08-16)


### Bug Fixes

* handle "partial" payment uri like zcash:&lt;addr&gt;? ([#425](https://github.com/hhanh00/zkool2/issues/425)) ([da4749d](https://github.com/hhanh00/zkool2/commit/da4749dea9a1261a78d017d86148cf24030dc6ad))

## [4.20.9](https://github.com/hhanh00/zkool2/compare/zkool-v4.20.8...zkool-v4.20.9) (2025-08-15)


### Bug Fixes

* remove db creation with password ([#419](https://github.com/hhanh00/zkool2/issues/419)) ([7cb1583](https://github.com/hhanh00/zkool2/commit/7cb15835e37b544a4805dc887f1c9a7ad78f7df0))

## [4.20.8](https://github.com/hhanh00/zkool2/compare/zkool-v4.20.7...zkool-v4.20.8) (2025-08-15)


### Bug Fixes

* add a confirmation prompt ([#417](https://github.com/hhanh00/zkool2/issues/417)) ([a10168e](https://github.com/hhanh00/zkool2/commit/a10168e9ed82043a3232a94961e1e309b2826948))

## [4.20.7](https://github.com/hhanh00/zkool2/compare/zkool-v4.20.6...zkool-v4.20.7) (2025-08-15)


### Bug Fixes

* don't fetch chart on linux because the webview isn't supported ([#415](https://github.com/hhanh00/zkool2/issues/415)) ([a4d4982](https://github.com/hhanh00/zkool2/commit/a4d498244a81b4ded0e96414c132309d660a1079))

## [4.20.6](https://github.com/hhanh00/zkool2/compare/zkool-v4.20.5...zkool-v4.20.6) (2025-08-15)


### Bug Fixes

* reformat payment uri ([#413](https://github.com/hhanh00/zkool2/issues/413)) ([60a04d3](https://github.com/hhanh00/zkool2/commit/60a04d3c7b90c1404a86194338eb8e3946a19901))

## [4.20.5](https://github.com/hhanh00/zkool2/compare/zkool-v4.20.4...zkool-v4.20.5) (2025-08-15)


### Bug Fixes

* move payment uri to extra options page ([#409](https://github.com/hhanh00/zkool2/issues/409)) ([70230ce](https://github.com/hhanh00/zkool2/commit/70230ce8e30773abc2d669f58d36bc9b0383417b))

## [4.20.4](https://github.com/hhanh00/zkool2/compare/zkool-v4.20.3...zkool-v4.20.4) (2025-08-14)


### Bug Fixes

* missing memo field ([#407](https://github.com/hhanh00/zkool2/issues/407)) ([1cfff16](https://github.com/hhanh00/zkool2/commit/1cfff1619b2fdbddd4e100273a6e54282106530c))

## [4.20.3](https://github.com/hhanh00/zkool2/compare/zkool-v4.20.2...zkool-v4.20.3) (2025-08-13)


### Bug Fixes

* app locks even when pin is off ([#405](https://github.com/hhanh00/zkool2/issues/405)) ([3005d07](https://github.com/hhanh00/zkool2/commit/3005d077a5d30ed2ba9931ce8f09489af89e5271))

## [4.20.2](https://github.com/hhanh00/zkool2/compare/zkool-v4.20.1...zkool-v4.20.2) (2025-08-13)


### Bug Fixes

* access database manager from open db dialog ([#403](https://github.com/hhanh00/zkool2/issues/403)) ([2de33e5](https://github.com/hhanh00/zkool2/commit/2de33e569b522ff7fde8cfcd498a7d3cf465e4c9))

## [4.20.1](https://github.com/hhanh00/zkool2/compare/zkool-v4.20.0...zkool-v4.20.1) (2025-08-12)


### Bug Fixes

* move database manager to recovery mode ([#400](https://github.com/hhanh00/zkool2/issues/400)) ([83659ac](https://github.com/hhanh00/zkool2/commit/83659acf4baec38b550f0a96a98c51b0324732e8))
* tooltip & router bug ([#402](https://github.com/hhanh00/zkool2/issues/402)) ([fd84d59](https://github.com/hhanh00/zkool2/commit/fd84d59b373dc588459eade631fbd034a9c680a9))

## [4.20.0](https://github.com/hhanh00/zkool2/compare/zkool-v4.19.0...zkool-v4.20.0) (2025-08-12)


### Features

* editable multipay tx ([#397](https://github.com/hhanh00/zkool2/issues/397)) ([54cb67a](https://github.com/hhanh00/zkool2/commit/54cb67a626349095dc6ad9db5264642766553a68))


### Bug Fixes

* manual pinlock ([#399](https://github.com/hhanh00/zkool2/issues/399)) ([bd7ec6c](https://github.com/hhanh00/zkool2/commit/bd7ec6cc0d8b1027a633a3907a515b8fa7dbdcfc))

## [4.19.0](https://github.com/hhanh00/zkool2/compare/zkool-v4.18.2...zkool-v4.19.0) (2025-08-11)


### Features

* database manager ([#395](https://github.com/hhanh00/zkool2/issues/395)) ([1ae9a14](https://github.com/hhanh00/zkool2/commit/1ae9a1476da14c86f1c56395c4bf0917d4c09f2d))

## [4.18.2](https://github.com/hhanh00/zkool2/compare/zkool-v4.18.1...zkool-v4.18.2) (2025-08-11)


### Bug Fixes

* switching useTOR in settings page ([#393](https://github.com/hhanh00/zkool2/issues/393)) ([6301def](https://github.com/hhanh00/zkool2/commit/6301def364785a7c7176b261b18498228af26dd2))

## [4.18.1](https://github.com/hhanh00/zkool2/compare/zkool-v4.18.0...zkool-v4.18.1) (2025-08-09)


### Bug Fixes

* add authentication to Settings page ([#392](https://github.com/hhanh00/zkool2/issues/392)) ([891bcbc](https://github.com/hhanh00/zkool2/commit/891bcbc1a614c207ff4be588eabfd046dd6f3790))
* put zip file in windows build artifact ([#391](https://github.com/hhanh00/zkool2/issues/391)) ([024dbe7](https://github.com/hhanh00/zkool2/commit/024dbe7106edca257e0848f262608629fdc69041))
* show authentication error message ([#389](https://github.com/hhanh00/zkool2/issues/389)) ([8b7ce52](https://github.com/hhanh00/zkool2/commit/8b7ce522bf257d9105be3993916305399f011a5f))

## [4.18.0](https://github.com/hhanh00/zkool2/compare/zkool-v4.17.0...zkool-v4.18.0) (2025-08-09)


### Features

* add generate seed button in restore account section ([#382](https://github.com/hhanh00/zkool2/issues/382)) ([853a642](https://github.com/hhanh00/zkool2/commit/853a6426a64d1067dda5077245817f47195c4728))
* open/save database file ([#387](https://github.com/hhanh00/zkool2/issues/387)) ([cf5b421](https://github.com/hhanh00/zkool2/commit/cf5b421d5b941294457f7376665e750898d96671))
* show memos as speech bubbles ([#384](https://github.com/hhanh00/zkool2/issues/384)) ([756c854](https://github.com/hhanh00/zkool2/commit/756c854c0dfff9dd6355dbccc2aec4766435baa2))


### Bug Fixes

* add confirmation messages after successful open/save database ([#388](https://github.com/hhanh00/zkool2/issues/388)) ([7bc4976](https://github.com/hhanh00/zkool2/commit/7bc497603ef0f803e22a03c17fdcf67cbe91deb1))
* call disableLock before modal dialogs ([#386](https://github.com/hhanh00/zkool2/issues/386)) ([f31818d](https://github.com/hhanh00/zkool2/commit/f31818d3bbf50d654c397bbd454e2656b09b30b4))
* import/save account when pinlock active ([#385](https://github.com/hhanh00/zkool2/issues/385)) ([975c465](https://github.com/hhanh00/zkool2/commit/975c465799c4b9b6363c5ea1e19ee9b42f82f054))

## [4.17.0](https://github.com/hhanh00/zkool2/compare/zkool-v4.16.1...zkool-v4.17.0) (2025-08-08)


### Features

* add support for TOR connection ([#379](https://github.com/hhanh00/zkool2/issues/379)) ([20c588b](https://github.com/hhanh00/zkool2/commit/20c588b19d75f961911b913d9b68ead2537b077f))


### Bug Fixes

* pass tor cache dirs ([#381](https://github.com/hhanh00/zkool2/issues/381)) ([13623c1](https://github.com/hhanh00/zkool2/commit/13623c1d6595535703ad5b239191fd60e49b8d42))

## [4.16.1](https://github.com/hhanh00/zkool2/compare/zkool-v4.16.0...zkool-v4.16.1) (2025-08-08)


### Bug Fixes

* add zip package to windows release ([#377](https://github.com/hhanh00/zkool2/issues/377)) ([5557585](https://github.com/hhanh00/zkool2/commit/55575857e1bd9edcd87635e9775dfa4940228c30))

## [4.16.0](https://github.com/hhanh00/zkool2/compare/zkool-v4.15.1...zkool-v4.16.0) (2025-08-07)


### Features

* offline mode ([#375](https://github.com/hhanh00/zkool2/issues/375)) ([ecca14d](https://github.com/hhanh00/zkool2/commit/ecca14da2b22a8c009cbe564a328aed2df4df6a1))

## [4.15.1](https://github.com/hhanh00/zkool2/compare/zkool-v4.15.0...zkool-v4.15.1) (2025-08-07)


### Bug Fixes

* restore support for testnet ([#372](https://github.com/hhanh00/zkool2/issues/372)) ([837e689](https://github.com/hhanh00/zkool2/commit/837e68982dd414c6fbc278d7af1d129c8b2a745b))

## [4.15.0](https://github.com/hhanh00/zkool2/compare/zkool-v4.14.7...zkool-v4.15.0) (2025-08-06)


### Features

* support for regtest ([#369](https://github.com/hhanh00/zkool2/issues/369)) ([74b8168](https://github.com/hhanh00/zkool2/commit/74b81682bbb9c91f31fd1054e1ef9fbaa6a6b06f))


### Bug Fixes

* regtest send/receive ([#371](https://github.com/hhanh00/zkool2/issues/371)) ([f8bc3f6](https://github.com/hhanh00/zkool2/commit/f8bc3f66b43a102f337427888905b9ab457db7ce))

## [4.14.7](https://github.com/hhanh00/zkool2/compare/zkool-v4.14.6...zkool-v4.14.7) (2025-08-05)


### Bug Fixes

* issue with authenticated being requested multiple times ([#367](https://github.com/hhanh00/zkool2/issues/367)) ([24a05db](https://github.com/hhanh00/zkool2/commit/24a05dbc7fcc9ca6c2d5921ba82ce52e869ce150))

## [4.14.6](https://github.com/hhanh00/zkool2/compare/zkool-v4.14.5...zkool-v4.14.6) (2025-08-05)


### Bug Fixes

* fee does not take min action cost into consideration ([#365](https://github.com/hhanh00/zkool2/issues/365)) ([9c26e2c](https://github.com/hhanh00/zkool2/commit/9c26e2cfa718686fbc0a49df1490b5e94fb426ca))

## [4.14.5](https://github.com/hhanh00/zkool2/compare/zkool-v4.14.4...zkool-v4.14.5) (2025-08-03)


### Bug Fixes

* database encryption + pinlock ([#363](https://github.com/hhanh00/zkool2/issues/363)) ([1ebeb63](https://github.com/hhanh00/zkool2/commit/1ebeb6345f9c191c7aaf1592f1e14ea21bb87076))

## [4.14.4](https://github.com/hhanh00/zkool2/compare/zkool-v4.14.3...zkool-v4.14.4) (2025-08-03)


### Bug Fixes

* workaround for bug in SelectableText on iOS ([#361](https://github.com/hhanh00/zkool2/issues/361)) ([b8043b5](https://github.com/hhanh00/zkool2/commit/b8043b53a866e9907cd69dd2102567824c63f7e0))

## [4.14.3](https://github.com/hhanh00/zkool2/compare/zkool-v4.14.2...zkool-v4.14.3) (2025-08-02)


### Bug Fixes

* blank screen due to race condition ([#359](https://github.com/hhanh00/zkool2/issues/359)) ([9286eaf](https://github.com/hhanh00/zkool2/commit/9286eaf338c3d9cff10515c4040a0939110578e4))

## [4.14.2](https://github.com/hhanh00/zkool2/compare/zkool-v4.14.1...zkool-v4.14.2) (2025-08-02)


### Bug Fixes

* account refresh UI (again) ([#357](https://github.com/hhanh00/zkool2/issues/357)) ([674948c](https://github.com/hhanh00/zkool2/commit/674948cc1261939df12e87993c9750984c06ee42))

## [4.14.1](https://github.com/hhanh00/zkool2/compare/zkool-v4.14.0...zkool-v4.14.1) (2025-08-02)


### Bug Fixes

* account ui fresh ([#355](https://github.com/hhanh00/zkool2/issues/355)) ([c6127e6](https://github.com/hhanh00/zkool2/commit/c6127e6c50691a1a62d61f8c7e984721e485618b))

## [4.14.0](https://github.com/hhanh00/zkool2/compare/zkool-v4.13.0...zkool-v4.14.0) (2025-08-02)


### Features

* show progress bars when synchronizing ([#353](https://github.com/hhanh00/zkool2/issues/353)) ([96565a1](https://github.com/hhanh00/zkool2/commit/96565a1d67c302790a0944ee85793954be8cff7b))

## [4.13.0](https://github.com/hhanh00/zkool2/compare/zkool-v4.12.0...zkool-v4.13.0) (2025-08-01)


### Features

* lock the app with the device auth ([#350](https://github.com/hhanh00/zkool2/issues/350)) ([6c564f8](https://github.com/hhanh00/zkool2/commit/6c564f86c73a4263dcee274dfbdbb0beef38b3ae))

## [4.12.0](https://github.com/hhanh00/zkool2/compare/zkool-v4.11.2...zkool-v4.12.0) (2025-07-31)


### Features

* save current account and resume from it ([#347](https://github.com/hhanh00/zkool2/issues/347)) ([6aa0719](https://github.com/hhanh00/zkool2/commit/6aa0719eec0591e48e7bcfe2638ae44f08702728))


### Bug Fixes

* disclaimer and account preloading ([#349](https://github.com/hhanh00/zkool2/issues/349)) ([301b575](https://github.com/hhanh00/zkool2/commit/301b575be598fe1c244e05863aad4fe7d5930766))

## [4.11.2](https://github.com/hhanh00/zkool2/compare/zkool-v4.11.1...zkool-v4.11.2) (2025-07-30)


### Bug Fixes

* macos notarization ([#344](https://github.com/hhanh00/zkool2/issues/344)) ([38d8dab](https://github.com/hhanh00/zkool2/commit/38d8dab5ee42718150c34a8c96ec5ca29f8c839e))

## [4.11.1](https://github.com/hhanh00/zkool2/compare/zkool-v4.11.0...zkool-v4.11.1) (2025-07-30)


### Bug Fixes

* add count of txs ([#342](https://github.com/hhanh00/zkool2/issues/342)) ([618b445](https://github.com/hhanh00/zkool2/commit/618b44514c9c9ea71379a4e84fa7d0e4243b8d34))

## [4.11.0](https://github.com/hhanh00/zkool2/compare/zkool-v4.10.0...zkool-v4.11.0) (2025-07-26)


### Features

* select pools of receiving ua ([#338](https://github.com/hhanh00/zkool2/issues/338)) ([a09c540](https://github.com/hhanh00/zkool2/commit/a09c540a72200ff3f6b8fa70af126ca7b9ace316))


### Bug Fixes

* new diversified address without transparent receiver ([#336](https://github.com/hhanh00/zkool2/issues/336)) ([ab94c56](https://github.com/hhanh00/zkool2/commit/ab94c56fca1dbeb65fd5547dfed5a5fd10738797))
* pool selection during restore ([#339](https://github.com/hhanh00/zkool2/issues/339)) ([2e82e14](https://github.com/hhanh00/zkool2/commit/2e82e14174811c978c61bf5d3fd2ad94723176f3))

## [4.10.0](https://github.com/hhanh00/zkool2/compare/zkool-v4.9.6...zkool-v4.10.0) (2025-07-25)


### Features

* select account pools when restoring from key ([#334](https://github.com/hhanh00/zkool2/issues/334)) ([cf5a4c9](https://github.com/hhanh00/zkool2/commit/cf5a4c9acd7292c5c91d5785324a42439b0c558c))

## [4.9.6](https://github.com/hhanh00/zkool2/compare/zkool-v4.9.5...zkool-v4.9.6) (2025-07-24)


### Bug Fixes

* icon for shield/unshield buttons ([#332](https://github.com/hhanh00/zkool2/issues/332)) ([bf5a6ac](https://github.com/hhanh00/zkool2/commit/bf5a6acc3bf85c903d9a610389be51b9280d3222))

## [4.9.5](https://github.com/hhanh00/zkool2/compare/zkool-v4.9.4...zkool-v4.9.5) (2025-07-24)


### Bug Fixes

* lazy loading of tx history to listview ([#330](https://github.com/hhanh00/zkool2/issues/330)) ([99c9b46](https://github.com/hhanh00/zkool2/commit/99c9b460fc9f23685f4c97903e58e9ee8add3261))

## [4.9.4](https://github.com/hhanh00/zkool2/compare/zkool-v4.9.3...zkool-v4.9.4) (2025-07-24)


### Bug Fixes

* missing decimal point on keyboard input for amount ([#328](https://github.com/hhanh00/zkool2/issues/328)) ([bfddfe6](https://github.com/hhanh00/zkool2/commit/bfddfe63d412aa6ae079bd2a3a437c566581d012))

## [4.9.3](https://github.com/hhanh00/zkool2/compare/zkool-v4.9.2...zkool-v4.9.3) (2025-07-23)


### Bug Fixes

* missing tx time for transparent only accounts ([#326](https://github.com/hhanh00/zkool2/issues/326)) ([f942eed](https://github.com/hhanh00/zkool2/commit/f942eed4fb469eab06a619b119f50096143e43df))

## [4.9.2](https://github.com/hhanh00/zkool2/compare/zkool-v4.9.1...zkool-v4.9.2) (2025-07-21)


### Bug Fixes

* add last time used to transparent addresses ([#324](https://github.com/hhanh00/zkool2/issues/324)) ([2c251e8](https://github.com/hhanh00/zkool2/commit/2c251e80906f6753e9d1c222891d78fd927b4ce1))

## [4.9.1](https://github.com/hhanh00/zkool2/compare/zkool-v4.9.0...zkool-v4.9.1) (2025-07-20)


### Bug Fixes

* fetch_transparent_address_tx_count ([#322](https://github.com/hhanh00/zkool2/issues/322)) ([43274b3](https://github.com/hhanh00/zkool2/commit/43274b39e048e7c9b09f17b7556a56d2ad9f5ce5))

## [4.9.0](https://github.com/hhanh00/zkool2/compare/zkool-v4.8.0...zkool-v4.9.0) (2025-07-20)


### Features

* show transparent address pool and usage ([#320](https://github.com/hhanh00/zkool2/issues/320)) ([13dff31](https://github.com/hhanh00/zkool2/commit/13dff3144321e43724dad0d2532544f4da287ce5))

## [4.8.0](https://github.com/hhanh00/zkool2/compare/zkool-v4.7.0...zkool-v4.8.0) (2025-07-18)


### Features

* shield one taddress at a time ([#319](https://github.com/hhanh00/zkool2/issues/319)) ([70d7005](https://github.com/hhanh00/zkool2/commit/70d700568f305b68c2d8cd49c1004999d39a5697))


### Bug Fixes

* move mempool button to appbar ([#318](https://github.com/hhanh00/zkool2/issues/318)) ([90487bf](https://github.com/hhanh00/zkool2/commit/90487bf91c00d2b7ea1010ac3c297275bd0b8b59))
* show error message if app fails to load ([#314](https://github.com/hhanh00/zkool2/issues/314)) ([b05e979](https://github.com/hhanh00/zkool2/commit/b05e979df04752b9d56f92ae0ab8f4a423741192))

## [4.7.0](https://github.com/hhanh00/zkool2/compare/zkool-v4.6.5...zkool-v4.7.0) (2025-07-14)


### Features

* market price chart using tradingview widget ([#312](https://github.com/hhanh00/zkool2/issues/312)) ([5a1286a](https://github.com/hhanh00/zkool2/commit/5a1286a38ed0749e7f442c2bbc4b3be5d1b6d77c))

## [4.6.5](https://github.com/hhanh00/zkool2/compare/zkool-v4.6.4...zkool-v4.6.5) (2025-06-30)


### Bug Fixes

* typos connection/pool ([#309](https://github.com/hhanh00/zkool2/issues/309)) ([d355827](https://github.com/hhanh00/zkool2/commit/d355827124bd57e272e551d75ad08652828f336b))
* usage of SqlitePool -&gt; SqliteConnection ([#307](https://github.com/hhanh00/zkool2/issues/307)) ([976addc](https://github.com/hhanh00/zkool2/commit/976addc57c610cb735dda4ad4f6a4dab330376b7))

## [4.6.4](https://github.com/hhanh00/zkool2/compare/zkool-v4.6.3...zkool-v4.6.4) (2025-06-29)


### Bug Fixes

* race condition at db creation ([#305](https://github.com/hhanh00/zkool2/issues/305)) ([8ec2515](https://github.com/hhanh00/zkool2/commit/8ec251545bcbada135f0d0a9cdadb8cd7bbba874))

## [4.6.3](https://github.com/hhanh00/zkool2/compare/zkool-v4.6.2...zkool-v4.6.3) (2025-06-29)


### Bug Fixes

* taddress_txs for full node ([#303](https://github.com/hhanh00/zkool2/issues/303)) ([b68d687](https://github.com/hhanh00/zkool2/commit/b68d6875b67e411ed70dca01cb0f05b91e490a6b))

## [4.6.2](https://github.com/hhanh00/zkool2/compare/zkool-v4.6.1...zkool-v4.6.2) (2025-06-29)


### Bug Fixes

* name of server field on settings page ([#301](https://github.com/hhanh00/zkool2/issues/301)) ([927bb5a](https://github.com/hhanh00/zkool2/commit/927bb5a3ea8f70bc0539cb86519e86991609429b))

## [4.6.1](https://github.com/hhanh00/zkool2/compare/zkool-v4.6.0...zkool-v4.6.1) (2025-06-28)


### Bug Fixes

* ios build ([#299](https://github.com/hhanh00/zkool2/issues/299)) ([a42810e](https://github.com/hhanh00/zkool2/commit/a42810e232cbb41d18573906cc32a2275eba2544))

## [4.6.0](https://github.com/hhanh00/zkool2/compare/zkool-v4.5.2...zkool-v4.6.0) (2025-06-28)


### Features

* support for full node servers (zcashd, zebrad) ([#297](https://github.com/hhanh00/zkool2/issues/297)) ([4412d40](https://github.com/hhanh00/zkool2/commit/4412d404d1c20a57047236ee993dc6259a5dc93d))

## [4.5.2](https://github.com/hhanh00/zkool2/compare/zkool-v4.5.1...zkool-v4.5.2) (2025-06-25)


### Bug Fixes

* fee adjustment when no change output ([#294](https://github.com/hhanh00/zkool2/issues/294)) ([010502b](https://github.com/hhanh00/zkool2/commit/010502ba7f58e5f8b61bd3f947eb93ab5eb3519b))

## [4.5.1](https://github.com/hhanh00/zkool2/compare/zkool-v4.5.0...zkool-v4.5.1) (2025-06-18)


### Bug Fixes

* issue if the only sync point we have is reorged ([#291](https://github.com/hhanh00/zkool2/issues/291)) ([93d95e3](https://github.com/hhanh00/zkool2/commit/93d95e36a6dac549179b1d6502e44ec0611e961b))

## [4.5.0](https://github.com/hhanh00/zkool2/compare/zkool-v4.4.1...zkool-v4.5.0) (2025-06-17)


### Features

* add support for testnet ([#289](https://github.com/hhanh00/zkool2/issues/289)) ([5535601](https://github.com/hhanh00/zkool2/commit/5535601127ccc16fae36aef9e62a307234e944ab))

## [4.4.1](https://github.com/hhanh00/zkool2/compare/zkool-v4.4.0...zkool-v4.4.1) (2025-06-13)


### Bug Fixes

* invalid dindex when importing from sapling key ([#286](https://github.com/hhanh00/zkool2/issues/286)) ([8852f82](https://github.com/hhanh00/zkool2/commit/8852f823813b93e20f4846a0f45a77bf04c979d2))

## [4.4.0](https://github.com/hhanh00/zkool2/compare/zkool-v4.3.0...zkool-v4.4.0) (2025-06-12)


### Features

* display distance from tip after account height ([#285](https://github.com/hhanh00/zkool2/issues/285)) ([1180c59](https://github.com/hhanh00/zkool2/commit/1180c59d9e55fd175c34adec40c8cacb424778ba))


### Bug Fixes

* change icon for spend/receive tx ([#283](https://github.com/hhanh00/zkool2/issues/283)) ([8844c31](https://github.com/hhanh00/zkool2/commit/8844c31e9b375fad2cb8d74accc5f23a8b117c1c))

## [4.3.0](https://github.com/hhanh00/zkool2/compare/zkool-v4.2.0...zkool-v4.3.0) (2025-06-12)


### Features

* shield and unshield buttons ([8b56806](https://github.com/hhanh00/zkool2/commit/8b56806ef88f6ccbc12a4429eb53648434bd57ea))


### Bug Fixes

* remove unused button ([#281](https://github.com/hhanh00/zkool2/issues/281)) ([8b56806](https://github.com/hhanh00/zkool2/commit/8b56806ef88f6ccbc12a4429eb53648434bd57ea))

## [4.2.0](https://github.com/hhanh00/zkool2/compare/zkool-v4.1.0...zkool-v4.2.0) (2025-06-11)


### Features

* display raw mempool tx ([#277](https://github.com/hhanh00/zkool2/issues/277)) ([45aa645](https://github.com/hhanh00/zkool2/commit/45aa645f2a51a77a1937c6ce41558e1109847d18))
* introduce dust change policy: send or discard ([#279](https://github.com/hhanh00/zkool2/issues/279)) ([14082d6](https://github.com/hhanh00/zkool2/commit/14082d687b5e4c70b84a222afed6909b8a481561))


### Bug Fixes

* pool balance display ([#280](https://github.com/hhanh00/zkool2/issues/280)) ([76f9d50](https://github.com/hhanh00/zkool2/commit/76f9d504a53b07a96c1a52b5b6c5d83f5a20cfb5))

## [4.1.0](https://github.com/hhanh00/zkool2/compare/zkool-v4.0.1...zkool-v4.1.0) (2025-06-10)


### Features

* show digits after millis in smaller size and colorize received funds ([#276](https://github.com/hhanh00/zkool2/issues/276)) ([2f45b2a](https://github.com/hhanh00/zkool2/commit/2f45b2a2586ec50528e86fc93d4fe8ca6d966b00))


### Bug Fixes

* remove fingerprint input ([#274](https://github.com/hhanh00/zkool2/issues/274)) ([c09e560](https://github.com/hhanh00/zkool2/commit/c09e56094050083736391f21c9b9e088bcc1731d))

## [4.0.1](https://github.com/hhanh00/zkool2/compare/zkool-v4.0.0...zkool-v4.0.1) (2025-06-08)


### Bug Fixes

* import/export of fee column ([#272](https://github.com/hhanh00/zkool2/issues/272)) ([0cea993](https://github.com/hhanh00/zkool2/commit/0cea993619ed4896a1c59edf03b801bf61708326))

## [4.0.0](https://github.com/hhanh00/zkool2/compare/zkool-v3.0.3...zkool-v4.0.0) (2025-06-08)


### ⚠ BREAKING CHANGES

* add fee column to transactions table ([#270](https://github.com/hhanh00/zkool2/issues/270))

### Features

* add fee column to transactions table ([#270](https://github.com/hhanh00/zkool2/issues/270)) ([b2aae79](https://github.com/hhanh00/zkool2/commit/b2aae7988750ce1856a0b3c8ca3f8d6592383344))


### Bug Fixes

* incorrect labelling of tx in certain cases ([b2aae79](https://github.com/hhanh00/zkool2/commit/b2aae7988750ce1856a0b3c8ca3f8d6592383344))
* make values of the tx details selectable ([b2aae79](https://github.com/hhanh00/zkool2/commit/b2aae7988750ce1856a0b3c8ca3f8d6592383344))

## [3.0.3](https://github.com/hhanh00/zkool2/compare/zkool-v3.0.2...zkool-v3.0.3) (2025-06-04)


### Bug Fixes

* show date in YYYY-MM-DD too in tx and memo lists ([#268](https://github.com/hhanh00/zkool2/issues/268)) ([390995b](https://github.com/hhanh00/zkool2/commit/390995b4ecc6d53f30bbfde01de5e259375a8f38))

## [3.0.2](https://github.com/hhanh00/zkool2/compare/zkool-v3.0.1...zkool-v3.0.2) (2025-06-04)


### Bug Fixes

* parallel/concurrent donwload & trial decryption ([#266](https://github.com/hhanh00/zkool2/issues/266)) ([2102e16](https://github.com/hhanh00/zkool2/commit/2102e16fa71693337f90c2fc3d748c9802393182))

## [3.0.1](https://github.com/hhanh00/zkool2/compare/zkool-v3.0.0...zkool-v3.0.1) (2025-06-03)


### Bug Fixes

* sync error reporting ([#264](https://github.com/hhanh00/zkool2/issues/264)) ([6e00299](https://github.com/hhanh00/zkool2/commit/6e0029911f31e5d9caadf085bfe264a54fd265b3))

## [3.0.0](https://github.com/hhanh00/zkool2/compare/zkool-v2.9.0...zkool-v3.0.0) (2025-06-02)


### ⚠ BREAKING CHANGES

* decode and store outputs ([#262](https://github.com/hhanh00/zkool2/issues/262))

### Features

* decode and store outputs ([#262](https://github.com/hhanh00/zkool2/issues/262)) ([cf681e7](https://github.com/hhanh00/zkool2/commit/cf681e7ec2f69a161af2752ee184b0823c6985d5))
* show transaction type ([cf681e7](https://github.com/hhanh00/zkool2/commit/cf681e7ec2f69a161af2752ee184b0823c6985d5))

## [2.9.0](https://github.com/hhanh00/zkool2/compare/zkool-v2.8.6...zkool-v2.9.0) (2025-06-02)


### Features

* delete db and change db password ([#260](https://github.com/hhanh00/zkool2/issues/260)) ([b79ef41](https://github.com/hhanh00/zkool2/commit/b79ef4138981449e885f1f0b7879e8a3010b0198))

## [2.8.6](https://github.com/hhanh00/zkool2/compare/zkool-v2.8.5...zkool-v2.8.6) (2025-06-01)


### Bug Fixes

* include encryption declaration ([#257](https://github.com/hhanh00/zkool2/issues/257)) ([37b7e09](https://github.com/hhanh00/zkool2/commit/37b7e0907e50e52c50d04105c75ccf4d65cd1686))

## [2.8.5](https://github.com/hhanh00/zkool2/compare/zkool-v2.8.4...zkool-v2.8.5) (2025-06-01)


### Bug Fixes

* linux arm build ([#256](https://github.com/hhanh00/zkool2/issues/256)) ([e712da0](https://github.com/hhanh00/zkool2/commit/e712da0ed889d6dfc50a15b2098f5669d1c3e3a9))
* show required amount needed ([#253](https://github.com/hhanh00/zkool2/issues/253)) ([8845a9f](https://github.com/hhanh00/zkool2/commit/8845a9fbb92ba297a549127975784a7f605e7e7d))

## [2.8.4](https://github.com/hhanh00/zkool2/compare/zkool-v2.8.3...zkool-v2.8.4) (2025-06-01)


### Bug Fixes

* split per abi android build ([#251](https://github.com/hhanh00/zkool2/issues/251)) ([7c2c98b](https://github.com/hhanh00/zkool2/commit/7c2c98bd11a22fab22629a1be5c5fc464ddd83c7))

## [2.8.3](https://github.com/hhanh00/zkool2/compare/zkool-v2.8.2...zkool-v2.8.3) (2025-06-01)


### Bug Fixes

* sending to tex address should disable pool selection ([#249](https://github.com/hhanh00/zkool2/issues/249)) ([b3b51da](https://github.com/hhanh00/zkool2/commit/b3b51da198c8516100a70aeb248996ea438f661f))

## [2.8.2](https://github.com/hhanh00/zkool2/compare/zkool-v2.8.1...zkool-v2.8.2) (2025-05-31)


### Bug Fixes

* ios ipa build ([#244](https://github.com/hhanh00/zkool2/issues/244)) ([ff9dafe](https://github.com/hhanh00/zkool2/commit/ff9dafee5c0a75c106753c675c9c0b0670e9ee64))
* race condition on new db creation ([#245](https://github.com/hhanh00/zkool2/issues/245)) ([4413dfe](https://github.com/hhanh00/zkool2/commit/4413dfe5ed6244fdfd80fc09c0f0974151fa8280))
* tweak icons and move rewind to edit account page ([#241](https://github.com/hhanh00/zkool2/issues/241)) ([4276735](https://github.com/hhanh00/zkool2/commit/427673544b3a9ed0cb2556bc0546aefe118c8cc9))

## [2.8.1](https://github.com/hhanh00/zkool2/compare/zkool-v2.8.0...zkool-v2.8.1) (2025-05-29)


### Bug Fixes

* trim trailing zeros in memo bytes for display ([#238](https://github.com/hhanh00/zkool2/issues/238)) ([4215d12](https://github.com/hhanh00/zkool2/commit/4215d12149de1d598ef60f005bc5a9a9fcf8086e))

## [2.8.0](https://github.com/hhanh00/zkool2/compare/zkool-v2.7.1...zkool-v2.8.0) (2025-05-29)


### Features

* search memos ([#236](https://github.com/hhanh00/zkool2/issues/236)) ([c67285a](https://github.com/hhanh00/zkool2/commit/c67285a75d0bbb6b7917eb8d7077ea302716a20d))

## [2.7.1](https://github.com/hhanh00/zkool2/compare/zkool-v2.7.0...zkool-v2.7.1) (2025-05-28)


### Bug Fixes

* mempool hang ([#232](https://github.com/hhanh00/zkool2/issues/232)) ([0dab800](https://github.com/hhanh00/zkool2/commit/0dab8004d5e84d0ff200a3646150d009bffeac59))
* mempool listener hang on server error ([#234](https://github.com/hhanh00/zkool2/issues/234)) ([131c691](https://github.com/hhanh00/zkool2/commit/131c6911e62203db6843e781da454e6ec5ed1468))

## [2.7.0](https://github.com/hhanh00/zkool2/compare/zkool-v2.6.2...zkool-v2.7.0) (2025-05-27)


### Features

* monitor app lifecycle and restart mempool monitor on resume ([#230](https://github.com/hhanh00/zkool2/issues/230)) ([e4b0d4c](https://github.com/hhanh00/zkool2/commit/e4b0d4c8f034c8747ef266da418f3675cde8dbca))

## [2.6.2](https://github.com/hhanh00/zkool2/compare/zkool-v2.6.1...zkool-v2.6.2) (2025-05-25)


### Bug Fixes

* android build ([#227](https://github.com/hhanh00/zkool2/issues/227)) ([f2e740a](https://github.com/hhanh00/zkool2/commit/f2e740a974abe43d53c806966ac503db383bb80c))

## [2.6.1](https://github.com/hhanh00/zkool2/compare/zkool-v2.6.0...zkool-v2.6.1) (2025-05-25)


### Bug Fixes

* show unconfirmed amount on account page ([#225](https://github.com/hhanh00/zkool2/issues/225)) ([ca4e82a](https://github.com/hhanh00/zkool2/commit/ca4e82aa595156bdedab57af7834717684d8a068))

## [2.6.0](https://github.com/hhanh00/zkool2/compare/zkool-v2.5.0...zkool-v2.6.0) (2025-05-25)


### Features

* add showcase tooltips for dkg/frost ([#224](https://github.com/hhanh00/zkool2/issues/224)) ([f61c352](https://github.com/hhanh00/zkool2/commit/f61c3520b33da356115f562c66bfe9e1b2ac6770))
* mempool tx amounts ([#220](https://github.com/hhanh00/zkool2/issues/220)) ([c9a9361](https://github.com/hhanh00/zkool2/commit/c9a93611f3de82f5c261e6a6f7dbe0fbdbd0e562))
* run mempool scanner button ([#222](https://github.com/hhanh00/zkool2/issues/222)) ([fec3deb](https://github.com/hhanh00/zkool2/commit/fec3debd3b3a162ecf62272b55666ebbacd92c1c))


### Bug Fixes

* ui tweaks ([#223](https://github.com/hhanh00/zkool2/issues/223)) ([46dd47e](https://github.com/hhanh00/zkool2/commit/46dd47e242ffcf7148b631af5b8b1314cd9f4d74))

## [2.5.0](https://github.com/hhanh00/zkool2/compare/zkool-v2.4.0...zkool-v2.5.0) (2025-05-24)


### Features

* add mempool page ([#218](https://github.com/hhanh00/zkool2/issues/218)) ([b6a6a16](https://github.com/hhanh00/zkool2/commit/b6a6a16babb2f8b762575d8a935c55562d470615))

## [2.4.0](https://github.com/hhanh00/zkool2/compare/zkool-v2.3.1...zkool-v2.4.0) (2025-05-23)


### Features

* show txid toast ([#216](https://github.com/hhanh00/zkool2/issues/216)) ([b841195](https://github.com/hhanh00/zkool2/commit/b841195d289e9027e44d404394737c764eb9091f))

## [2.3.1](https://github.com/hhanh00/zkool2/compare/zkool-v2.3.0...zkool-v2.3.1) (2025-05-23)


### Bug Fixes

* iOS build ([#214](https://github.com/hhanh00/zkool2/issues/214)) ([7ee7ed5](https://github.com/hhanh00/zkool2/commit/7ee7ed55499176104439abe2b1d2a09b28552d3c))

## [2.3.0](https://github.com/hhanh00/zkool2/compare/zkool-v2.2.1...zkool-v2.3.0) (2025-05-22)


### Features

* frost to sapling address ([#212](https://github.com/hhanh00/zkool2/issues/212)) ([c21b0f7](https://github.com/hhanh00/zkool2/commit/c21b0f7cc6a7ccf5747dcd4b6060789c478c8176))

## [2.2.1](https://github.com/hhanh00/zkool2/compare/zkool-v2.2.0...zkool-v2.2.1) (2025-05-22)


### Bug Fixes

* race condition in 3/3 signatures ([#210](https://github.com/hhanh00/zkool2/issues/210)) ([329ed5a](https://github.com/hhanh00/zkool2/commit/329ed5ac47d06037cf009fcb38a8da8165692616))

## [2.2.0](https://github.com/hhanh00/zkool2/compare/zkool-v2.1.1...zkool-v2.2.0) (2025-05-22)


### Features

* add cancel button to frost sign ([#209](https://github.com/hhanh00/zkool2/issues/209)) ([e91b917](https://github.com/hhanh00/zkool2/commit/e91b917446bcc5700e0c9e8c726937abdd961c60))


### Bug Fixes

* macos camera permissions ([#207](https://github.com/hhanh00/zkool2/issues/207)) ([87f5395](https://github.com/hhanh00/zkool2/commit/87f53954104257e3ee8c168428a11cb80c1f16cc))

## [2.1.1](https://github.com/hhanh00/zkool2/compare/zkool-v2.1.0...zkool-v2.1.1) (2025-05-21)


### Bug Fixes

* android build ([#205](https://github.com/hhanh00/zkool2/issues/205)) ([30d8885](https://github.com/hhanh00/zkool2/commit/30d8885f380501a8132d44ced3bbf77c6d67722c))

## [2.1.0](https://github.com/hhanh00/zkool2/compare/zkool-v2.0.0...zkool-v2.1.0) (2025-05-21)


### Features

* end to end frost transaction ([#199](https://github.com/hhanh00/zkool2/issues/199)) ([e278307](https://github.com/hhanh00/zkool2/commit/e278307778b0242aa116a9898551b6cd928e7f96))
* **frost:** calculate nonce & commitments, send to coordinator ([#189](https://github.com/hhanh00/zkool2/issues/189)) ([6344169](https://github.com/hhanh00/zkool2/commit/6344169b7267bf3cc58bf8a1ab4087f561f3453a))
* **frost:** coordinator sends signingpackage ([#191](https://github.com/hhanh00/zkool2/issues/191)) ([cf598ee](https://github.com/hhanh00/zkool2/commit/cf598ee7ae2e8b59fd649d432e318c38a466a75e))
* **frost:** create coordinator mailbox account ([#190](https://github.com/hhanh00/zkool2/issues/190)) ([47f02ef](https://github.com/hhanh00/zkool2/commit/47f02efb42b84aabb533cc4ad6bba7a3a4805b68))
* **frost:** initial frost mpc signature - phase 1 ([#187](https://github.com/hhanh00/zkool2/issues/187)) ([aa0fd7b](https://github.com/hhanh00/zkool2/commit/aa0fd7bbb0ad3ac7864376f66c4ca1e8340112c5))
* **frost:** rerandomized signature ([#196](https://github.com/hhanh00/zkool2/issues/196)) ([e4acb73](https://github.com/hhanh00/zkool2/commit/e4acb730a5f5ac101f12d2034446fbb6c89c6220))
* **frost:** tx/rx signature shares & aggregate ([#192](https://github.com/hhanh00/zkool2/issues/192)) ([89a3087](https://github.com/hhanh00/zkool2/commit/89a3087e361bd0bdcd82bfa48ba42590afa8b194))


### Bug Fixes

* delete dkg & frost tables with account ([#204](https://github.com/hhanh00/zkool2/issues/204)) ([f4e1f00](https://github.com/hhanh00/zkool2/commit/f4e1f00502a6d45380efae6e96bc1969c4511700))
* **dkg:** ui ([#201](https://github.com/hhanh00/zkool2/issues/201)) ([28a9aaa](https://github.com/hhanh00/zkool2/commit/28a9aaa1a448ee29520be4c0a54b1eb1f7f4853f))
* **frost:** misc bugs ([#202](https://github.com/hhanh00/zkool2/issues/202)) ([a7d9852](https://github.com/hhanh00/zkool2/commit/a7d985230fe28d3c6c73d9069ef49be104307662))
* **frost:** ui ([#200](https://github.com/hhanh00/zkool2/issues/200)) ([e0dfbf7](https://github.com/hhanh00/zkool2/commit/e0dfbf795efb4243c983dd8962c65a442770179e))
* icons and close button ([#203](https://github.com/hhanh00/zkool2/issues/203)) ([4942093](https://github.com/hhanh00/zkool2/commit/4942093f14e4fa83a46ee7a5083e51a3d13a45e2))
* use randomizer from pczt ([#198](https://github.com/hhanh00/zkool2/issues/198)) ([eb2cefb](https://github.com/hhanh00/zkool2/commit/eb2cefb5539e481024f3a97b549c0e94bac74cdb))

## [2.0.0](https://github.com/hhanh00/zkool2/compare/zkool-v1.17.0...zkool-v2.0.0) (2025-05-14)


### ⚠ BREAKING CHANGES

* internal flag added to account table

### Features

* **dkg:** auto-run dkg state machine ([#183](https://github.com/hhanh00/zkool2/issues/183)) ([a41c7dc](https://github.com/hhanh00/zkool2/commit/a41c7dcfacec8234cd42127fc87929197769cbb7))
* **dkg:** broadcast public package 1 ([#181](https://github.com/hhanh00/zkool2/issues/181)) ([27e5042](https://github.com/hhanh00/zkool2/commit/27e50429f0b4747c357fb1268a80f12ef23aea06))
* **dkg:** import/export frost dkg data ([#185](https://github.com/hhanh00/zkool2/issues/185)) ([21b4893](https://github.com/hhanh00/zkool2/commit/21b48935f2c92bc4d47b684de99bf4d9c11395ac))
* **dkg:** save packages to dkg_* tables ([#184](https://github.com/hhanh00/zkool2/issues/184)) ([d69ea14](https://github.com/hhanh00/zkool2/commit/d69ea14a41ca49247fee774b95f9f922824022d4))
* **dkg:** shared address generation ([#182](https://github.com/hhanh00/zkool2/issues/182)) ([82796d9](https://github.com/hhanh00/zkool2/commit/82796d9bfef6fe8dab16e1b5abdff5a31bcaa668))
* frost dkg - parameters - ui ([#178](https://github.com/hhanh00/zkool2/issues/178)) ([03915fd](https://github.com/hhanh00/zkool2/commit/03915fd86fba3823ccea4135b6190e31e8ccc08c))
* internal flag added to account table ([7132c16](https://github.com/hhanh00/zkool2/commit/7132c16aae34e7b8fffc5bdc50a0101a9907a2bc))


### Bug Fixes

* sync now ([7132c16](https://github.com/hhanh00/zkool2/commit/7132c16aae34e7b8fffc5bdc50a0101a9907a2bc))

## [1.17.0](https://github.com/hhanh00/zkool2/compare/zkool-v1.16.3...zkool-v1.17.0) (2025-05-10)


### Features

* support for tex addresses ([#176](https://github.com/hhanh00/zkool2/issues/176)) ([e885bbb](https://github.com/hhanh00/zkool2/commit/e885bbbb3496db20234a46ea85373fa50f8c85b8))

## [1.16.3](https://github.com/hhanh00/zkool2/compare/zkool-v1.16.2...zkool-v1.16.3) (2025-05-01)


### Bug Fixes

* edit account ([#173](https://github.com/hhanh00/zkool2/issues/173)) ([f6b3013](https://github.com/hhanh00/zkool2/commit/f6b3013ec921b131b90e6f4edafca900c602494c))

## [1.16.2](https://github.com/hhanh00/zkool2/compare/zkool-v1.16.1...zkool-v1.16.2) (2025-04-28)


### Bug Fixes

* navigation stack after tx cancel/submit ([#171](https://github.com/hhanh00/zkool2/issues/171)) ([94963f4](https://github.com/hhanh00/zkool2/commit/94963f4061bbe64ed1589aa855c0e3c600efb79e))

## [1.16.1](https://github.com/hhanh00/zkool2/compare/zkool-v1.16.0...zkool-v1.16.1) (2025-04-28)


### Bug Fixes

* disclaimer repeat showing ([#169](https://github.com/hhanh00/zkool2/issues/169)) ([7a843ef](https://github.com/hhanh00/zkool2/commit/7a843ef59c1da8199b55da4f76823ac97521943d))

## [1.16.0](https://github.com/hhanh00/zkool2/compare/zkool-v1.15.0...zkool-v1.16.0) (2025-04-28)


### Features

* display version and build number ([#168](https://github.com/hhanh00/zkool2/issues/168)) ([be91c0e](https://github.com/hhanh00/zkool2/commit/be91c0e66b023dfbd01b78d8d9429a1d31e6f783))
* splash screen & disclaimer page ([#166](https://github.com/hhanh00/zkool2/issues/166)) ([d5c07fb](https://github.com/hhanh00/zkool2/commit/d5c07fba9ccfe678154d3b30a14a0a6ad3375627))

## [1.15.0](https://github.com/hhanh00/zkool2/compare/zkool-v1.14.0...zkool-v1.15.0) (2025-04-27)


### Features

* cancel synchronization button ([#164](https://github.com/hhanh00/zkool2/issues/164)) ([862d272](https://github.com/hhanh00/zkool2/commit/862d2724b73a164d85beb0bcf23abd03440f30c6))

## [1.14.0](https://github.com/hhanh00/zkool2/compare/zkool-v1.13.1...zkool-v1.14.0) (2025-04-27)


### Features

* separate signing and broadcasting for cold wallet ([#159](https://github.com/hhanh00/zkool2/issues/159)) ([a2befd0](https://github.com/hhanh00/zkool2/commit/a2befd079ec4683277a3d9dab92cabe240ee19f4))


### Bug Fixes

* show seed should also show passphrase and index ([#162](https://github.com/hhanh00/zkool2/issues/162)) ([738a60b](https://github.com/hhanh00/zkool2/commit/738a60b3c29a484bd810630211916452f86f18bb))
* sync of selected accounts ([#163](https://github.com/hhanh00/zkool2/issues/163)) ([cf9d4c3](https://github.com/hhanh00/zkool2/commit/cf9d4c356438c2d46c36d036a60763cc282ac7ad))

## [1.13.1](https://github.com/hhanh00/zkool2/compare/zkool-v1.13.0...zkool-v1.13.1) (2025-04-26)


### Bug Fixes

* authentication bypass by cancel ([a3c6a45](https://github.com/hhanh00/zkool2/commit/a3c6a45120bcd2f09f8579dab6c7abb0cff7e6ac))
* incorrect tx amount in history when spent notes have the same amount ([#157](https://github.com/hhanh00/zkool2/issues/157)) ([a3c6a45](https://github.com/hhanh00/zkool2/commit/a3c6a45120bcd2f09f8579dab6c7abb0cff7e6ac))

## [1.13.0](https://github.com/hhanh00/zkool2/compare/zkool-v1.12.0...zkool-v1.13.0) (2025-04-26)


### Features

* add actions/sync option ([#155](https://github.com/hhanh00/zkool2/issues/155)) ([0d8983c](https://github.com/hhanh00/zkool2/commit/0d8983c40fdc4a0081b027a0965233c36d93054a))
* add max amount button ([#150](https://github.com/hhanh00/zkool2/issues/150)) ([7530107](https://github.com/hhanh00/zkool2/commit/75301078188384dbd4de34b8572c063bd8f8368b))
* expose seed fingerprint ([#143](https://github.com/hhanh00/zkool2/issues/143)) ([1e759b7](https://github.com/hhanh00/zkool2/commit/1e759b7318da74173581786ac94dcaf337a511fe))
* payment uris ([#147](https://github.com/hhanh00/zkool2/issues/147)) ([a2542f6](https://github.com/hhanh00/zkool2/commit/a2542f644ddb60f4cf34aae3f5a7dcf7c87c4985))
* prune old checkpoints ([#151](https://github.com/hhanh00/zkool2/issues/151)) ([bc5d11d](https://github.com/hhanh00/zkool2/commit/bc5d11d484446ae7fa7b21165f35c8c1e97646e0))
* return number of new transparent addresses found during sweep ([#149](https://github.com/hhanh00/zkool2/issues/149)) ([55a688b](https://github.com/hhanh00/zkool2/commit/55a688b844ff910da80d582c5876759cd6f009ba))
* separate tx building from signing, proving, etc. ([#145](https://github.com/hhanh00/zkool2/issues/145)) ([8edb8a7](https://github.com/hhanh00/zkool2/commit/8edb8a7a6eea9adb41e0035edef1d52694d5c93c))
* show seed & biometrics authentication ([#153](https://github.com/hhanh00/zkool2/issues/153)) ([635f472](https://github.com/hhanh00/zkool2/commit/635f47276dfb7da5817d8780691f5f4bc19d311c))
* show viewing keys ([#140](https://github.com/hhanh00/zkool2/issues/140)) ([9282f66](https://github.com/hhanh00/zkool2/commit/9282f66e18a4c7ae1f7dfa7db7ac25e0a4d1f16c))
* skip shielded scan when only transparent key available ([#142](https://github.com/hhanh00/zkool2/issues/142)) ([8b4f2f3](https://github.com/hhanh00/zkool2/commit/8b4f2f3f6d33ec6a309b19faa0de1eef9b407918))


### Bug Fixes

* center account balance ([#154](https://github.com/hhanh00/zkool2/issues/154)) ([c654ff3](https://github.com/hhanh00/zkool2/commit/c654ff3ac265318ea44ab595d4b5f77f37597b36))
* cold wallet spending ([#146](https://github.com/hhanh00/zkool2/issues/146)) ([75f7488](https://github.com/hhanh00/zkool2/commit/75f7488b5cedd9fd751aa45edef4396cde895bb7))
* log span guards ([#144](https://github.com/hhanh00/zkool2/issues/144)) ([26bb2e9](https://github.com/hhanh00/zkool2/commit/26bb2e9f1aefb68b5e12bb616244da20b46a5edc))
* multi payments ([#148](https://github.com/hhanh00/zkool2/issues/148)) ([fe10298](https://github.com/hhanh00/zkool2/commit/fe1029801f7143d68d83c247a8f71d81cd9fed3f))
* reset_sync should not trim headers ([55a688b](https://github.com/hhanh00/zkool2/commit/55a688b844ff910da80d582c5876759cd6f009ba))
* transparent sync when multiple accounts include the same ([#152](https://github.com/hhanh00/zkool2/issues/152)) ([86f1528](https://github.com/hhanh00/zkool2/commit/86f15288552e876ab1b531d8a26d176a9f140156))

## [1.12.0](https://github.com/hhanh00/zkool2/compare/zkool-v1.11.0...zkool-v1.12.0) (2025-04-23)


### Features

* build windows ([#138](https://github.com/hhanh00/zkool2/issues/138)) ([0c0852d](https://github.com/hhanh00/zkool2/commit/0c0852d56d23596ccc7e927fe15159dd8e95c4c9))


### Bug Fixes

* build windows ([#139](https://github.com/hhanh00/zkool2/issues/139)) ([7323995](https://github.com/hhanh00/zkool2/commit/7323995d2c05c9c1fd33b3c85082d80ba8f9b73f))
* macos sign with entitlements ([#136](https://github.com/hhanh00/zkool2/issues/136)) ([47ec6ab](https://github.com/hhanh00/zkool2/commit/47ec6ab3655b0482fd0a0766adf9d66c850cf7dc))

## [1.11.0](https://github.com/hhanh00/zkool2/compare/zkool-v1.10.0...zkool-v1.11.0) (2025-04-23)


### Features

* build for linux ([#134](https://github.com/hhanh00/zkool2/issues/134)) ([8f98db1](https://github.com/hhanh00/zkool2/commit/8f98db1bb8bc8bdbf5d189ef3dbf5d633e9464d7))
* dark mode ([#131](https://github.com/hhanh00/zkool2/issues/131)) ([10d5097](https://github.com/hhanh00/zkool2/commit/10d509700fa0e8f1e5f5a3d1558deb80926d37a7))

## [1.10.0](https://github.com/hhanh00/zkool2/compare/zkool-v1.9.0...zkool-v1.10.0) (2025-04-22)


### Features

* add confirmation dialog boxes ([#124](https://github.com/hhanh00/zkool2/issues/124)) ([bbed7cf](https://github.com/hhanh00/zkool2/commit/bbed7cfad98365fbdd63197c59b6bf74e1a6e67a))
* add tutorial ([#122](https://github.com/hhanh00/zkool2/issues/122)) ([22da20c](https://github.com/hhanh00/zkool2/commit/22da20cc086fa65582e5811940e4db8dab11e6ca))
* coin control ([#129](https://github.com/hhanh00/zkool2/issues/129)) ([b21e18e](https://github.com/hhanh00/zkool2/commit/b21e18e36216f446af9ffe18cf93393dbacb6d05))
* gzip account files before export ([#126](https://github.com/hhanh00/zkool2/issues/126)) ([387df3b](https://github.com/hhanh00/zkool2/commit/387df3bf9dba2466294943c5792b8153fd88b5f0))
* market price from coingecko ([#120](https://github.com/hhanh00/zkool2/issues/120)) ([a044e2e](https://github.com/hhanh00/zkool2/commit/a044e2e6c921b811fadc00411ee5172133e6e220))
* passphrase to seed ([#115](https://github.com/hhanh00/zkool2/issues/115)) ([6013cd6](https://github.com/hhanh00/zkool2/commit/6013cd67d3ce2c3022715b4da383461cea396a34))
* transaction details page ([#127](https://github.com/hhanh00/zkool2/issues/127)) ([e10ec58](https://github.com/hhanh00/zkool2/commit/e10ec5889d7e83657e11d44022c0ffae2f5a8ece))


### Bug Fixes

* account file encryption ([#130](https://github.com/hhanh00/zkool2/issues/130)) ([01118d0](https://github.com/hhanh00/zkool2/commit/01118d0e5484e876769ae5f6f46296a3734246d4))
* do not include spent notes in note tab ([01118d0](https://github.com/hhanh00/zkool2/commit/01118d0e5484e876769ae5f6f46296a3734246d4))
* do not reset sync height on edit birth ([#128](https://github.com/hhanh00/zkool2/issues/128)) ([9e3063f](https://github.com/hhanh00/zkool2/commit/9e3063fd89cfbbd68845aa2262f71d013d02277e))
* improve autosync reliability ([#117](https://github.com/hhanh00/zkool2/issues/117)) ([252cf20](https://github.com/hhanh00/zkool2/commit/252cf2030a4f63f3db8cfaa9d0523eecbfcf141d))
* missing refresh at end of sync ([#118](https://github.com/hhanh00/zkool2/issues/118)) ([37c1c2e](https://github.com/hhanh00/zkool2/commit/37c1c2ea63a1a2f7ef878d1fa2cbe46da41f4e31))
* reload accounts at the end of a sync ([#119](https://github.com/hhanh00/zkool2/issues/119)) ([a2e6ed4](https://github.com/hhanh00/zkool2/commit/a2e6ed4d4d9085bc0aa27f2fc2459d739e1b5cda))
* sync height update ([#116](https://github.com/hhanh00/zkool2/issues/116)) ([f3673f5](https://github.com/hhanh00/zkool2/commit/f3673f5fa602060b5e8025d3067e7af39fc1c97f))
* tutorial ([#123](https://github.com/hhanh00/zkool2/issues/123)) ([f31df9a](https://github.com/hhanh00/zkool2/commit/f31df9abc4ac90dae1d58df1be18a7185aca5f76))
* tutorial messages ([#125](https://github.com/hhanh00/zkool2/issues/125)) ([4c13886](https://github.com/hhanh00/zkool2/commit/4c138862b3c02ff3b54ffa523b40186fb4ba8e69))
* use separate column for orchard address scope ([#113](https://github.com/hhanh00/zkool2/issues/113)) ([bbaf898](https://github.com/hhanh00/zkool2/commit/bbaf898053bd0b70ece794b99f902d5f3fa5f622))

## [1.9.0](https://github.com/hhanh00/zkool2/compare/zkool-v1.8.0...zkool-v1.9.0) (2025-04-19)


### Features

* internal change account option ([#110](https://github.com/hhanh00/zkool2/issues/110)) ([9142c09](https://github.com/hhanh00/zkool2/commit/9142c0915412d4ab72de6885fb4fbc93dac49410))


### Bug Fixes

* rewind checkpoint ([#112](https://github.com/hhanh00/zkool2/issues/112)) ([b4c239e](https://github.com/hhanh00/zkool2/commit/b4c239e410297504e26155be7e02033f5df54e37))

## [1.8.0](https://github.com/hhanh00/zkool2/compare/zkool-v1.7.0...zkool-v1.8.0) (2025-04-18)


### Features

* fallback to default db on cancel password ([#109](https://github.com/hhanh00/zkool2/issues/109)) ([45f2fa0](https://github.com/hhanh00/zkool2/commit/45f2fa07a554def6e8a4ff0c668ceaa46a12363d))
* lwd url configuration setting ([#101](https://github.com/hhanh00/zkool2/issues/101)) ([7eadd90](https://github.com/hhanh00/zkool2/commit/7eadd906b4220298fbbc271755a690c498a94ea9))
* multi account edit ([#100](https://github.com/hhanh00/zkool2/issues/100)) ([d62ce95](https://github.com/hhanh00/zkool2/commit/d62ce95469068e60a2667e846bd58a793185e73f))
* show balance of all accounts ([#97](https://github.com/hhanh00/zkool2/issues/97)) ([a10d4a5](https://github.com/hhanh00/zkool2/commit/a10d4a57bb0139d881bc7bbad8374bf2e6883682))
* toasts and snackbars for log messages ([#98](https://github.com/hhanh00/zkool2/issues/98)) ([d82301e](https://github.com/hhanh00/zkool2/commit/d82301e6ce7863280a0fd205387fa477049e40be))
* transparent sweep ([#96](https://github.com/hhanh00/zkool2/issues/96)) ([3ace737](https://github.com/hhanh00/zkool2/commit/3ace737ad5f0467b28db8f410716b00c736df3e7))


### Bug Fixes

* account list style ([#99](https://github.com/hhanh00/zkool2/issues/99)) ([7d7cf3f](https://github.com/hhanh00/zkool2/commit/7d7cf3ff704620b1855146725f086e3478f0814c))
* account reorder, src pool selection ([#105](https://github.com/hhanh00/zkool2/issues/105)) ([3eb1b41](https://github.com/hhanh00/zkool2/commit/3eb1b4167ef5aa1ab577e2c953290ce4d191e1f0))
* change app icon ([#108](https://github.com/hhanh00/zkool2/issues/108)) ([1d6c95b](https://github.com/hhanh00/zkool2/commit/1d6c95be6f2feec0bb7ee378829121e206b79673))
* new account by xprv/xpub/bip38 ([#103](https://github.com/hhanh00/zkool2/issues/103)) ([d75953d](https://github.com/hhanh00/zkool2/commit/d75953d1885e75ef2913934e8797fe60a72506c0))
* new accounts don't show up ([#102](https://github.com/hhanh00/zkool2/issues/102)) ([6a07fb0](https://github.com/hhanh00/zkool2/commit/6a07fb0357c3110b39c5ad34dd2dc9f327f47c5e))
* sync and init ([#94](https://github.com/hhanh00/zkool2/issues/94)) ([e8ea72e](https://github.com/hhanh00/zkool2/commit/e8ea72e2c1fbeaf39455e6145e0f7dc1dd689c11))
* ufvk import ([#104](https://github.com/hhanh00/zkool2/issues/104)) ([6c93836](https://github.com/hhanh00/zkool2/commit/6c93836d95e969d538f182384a189a4832175b6a))
* use bundled sqlcipher ([#106](https://github.com/hhanh00/zkool2/issues/106)) ([685e776](https://github.com/hhanh00/zkool2/commit/685e7764c4f50a3f5faa1801f5bd552d1307e3f1))

## [1.7.0](https://github.com/hhanh00/zkool2/compare/zkool-v1.6.0...zkool-v1.7.0) (2025-04-15)


### Features

* database encryption ([#93](https://github.com/hhanh00/zkool2/issues/93)) ([2ddafec](https://github.com/hhanh00/zkool2/commit/2ddafec470f430165f1c8f5fbdb9d9bdac0d656a))
* synchronize checked accounts on account list ([#90](https://github.com/hhanh00/zkool2/issues/90)) ([8da267b](https://github.com/hhanh00/zkool2/commit/8da267b0fbbffc27f17d934403cd85ecd35509dc))


### Bug Fixes

* android build ([8da267b](https://github.com/hhanh00/zkool2/commit/8da267b0fbbffc27f17d934403cd85ecd35509dc))
* sync account list heights with sync state ([#92](https://github.com/hhanh00/zkool2/issues/92)) ([ec4da8b](https://github.com/hhanh00/zkool2/commit/ec4da8b2ba8e00f3e91701434ed8a0eaa61609a1))

## [1.6.0](https://github.com/hhanh00/zkool2/compare/zkool-v1.5.0...zkool-v1.6.0) (2025-04-14)


### Features

* android version ([#71](https://github.com/hhanh00/zkool2/issues/71)) ([b9935b8](https://github.com/hhanh00/zkool2/commit/b9935b8b30f35774de39045aa995bdae0af84072))
* export account data ([#75](https://github.com/hhanh00/zkool2/issues/75)) ([4927d47](https://github.com/hhanh00/zkool2/commit/4927d478952501fab7bfbe2a3e81fa14e133e9c5))
* import account from binary data ([#76](https://github.com/hhanh00/zkool2/issues/76)) ([8e52095](https://github.com/hhanh00/zkool2/commit/8e520951f89533b43d4546ad45d45cbf138a5b82))
* import accounts & encryption ([#77](https://github.com/hhanh00/zkool2/issues/77)) ([8eef09a](https://github.com/hhanh00/zkool2/commit/8eef09ab66b4804cfdd0e9dec2b19b76b6e9290e))
* multisend ([#88](https://github.com/hhanh00/zkool2/issues/88)) ([b47766e](https://github.com/hhanh00/zkool2/commit/b47766ebdf2995988c4e73aae9b6d53536ee7b14))
* QR code scanner and display ([#78](https://github.com/hhanh00/zkool2/issues/78)) ([2474b4e](https://github.com/hhanh00/zkool2/commit/2474b4e373346d21cc8e792ec53f5d8e911392b7))
* reorg detection and rewind ([#79](https://github.com/hhanh00/zkool2/issues/79)) ([93def21](https://github.com/hhanh00/zkool2/commit/93def21c22afe9f6ea53c3bf999ff5fd24c68fa5))
* reset account ([#81](https://github.com/hhanh00/zkool2/issues/81)) ([61feaaf](https://github.com/hhanh00/zkool2/commit/61feaaf0cb9047742cd090e3afe84056ce15ecbb))
* settings page ([#86](https://github.com/hhanh00/zkool2/issues/86)) ([73e237c](https://github.com/hhanh00/zkool2/commit/73e237c84c5f4643ce77036ad142a1038b1f87d0))
* synchronization retry with exponential backoff ([#80](https://github.com/hhanh00/zkool2/issues/80)) ([dd774e9](https://github.com/hhanh00/zkool2/commit/dd774e9caa935645b93621aa2b82cfea2859dddb))
* synchronize checked accounts on account list ([#89](https://github.com/hhanh00/zkool2/issues/89)) ([25289a1](https://github.com/hhanh00/zkool2/commit/25289a18c6c74b5775a38a198e72d4bac522585f))


### Bug Fixes

* error handling try/catch around network calls ([#74](https://github.com/hhanh00/zkool2/issues/74)) ([187e5be](https://github.com/hhanh00/zkool2/commit/187e5bedcdc6d253febc04b03e34b0a9be62bdc1))
* import accounts in first position ([#87](https://github.com/hhanh00/zkool2/issues/87)) ([d055d21](https://github.com/hhanh00/zkool2/commit/d055d210e47ab9b12165767e784fbc4e5155a401))
* load of transaction history and memos ([#83](https://github.com/hhanh00/zkool2/issues/83)) ([16b32a8](https://github.com/hhanh00/zkool2/commit/16b32a898f03ee3eefa5049ad347fd53c9842f86))
* retry sync ([#84](https://github.com/hhanh00/zkool2/issues/84)) ([80037af](https://github.com/hhanh00/zkool2/commit/80037af8cf2c47f13396a6597782dc94fea0bb22))
* sync of outgoing notes ([#85](https://github.com/hhanh00/zkool2/issues/85)) ([a7ebcc0](https://github.com/hhanh00/zkool2/commit/a7ebcc0a7e1faf2fe543fde57d2388b56590c625))
* tx memo parsing ([#73](https://github.com/hhanh00/zkool2/issues/73)) ([1af9c80](https://github.com/hhanh00/zkool2/commit/1af9c80c729d969b143205d8ec95dd7c1b09279c))
* updates to sync height were not going through after navigating away from account page ([#82](https://github.com/hhanh00/zkool2/issues/82)) ([3e610f7](https://github.com/hhanh00/zkool2/commit/3e610f74dda37265509a926a451f98d7704c9172))

## [1.5.0](https://github.com/hhanh00/zkool2/compare/zkool-v1.4.0...zkool-v1.5.0) (2025-04-12)


### Features

* auto renew transparent change address ([#65](https://github.com/hhanh00/zkool2/issues/65)) ([1d42e75](https://github.com/hhanh00/zkool2/commit/1d42e7514ab54609d5ec83109c01a8532feaba79))
* expose src pools and recipient pays fees to UI ([#64](https://github.com/hhanh00/zkool2/issues/64)) ([150e063](https://github.com/hhanh00/zkool2/commit/150e0634bff3ae608654bdc86b5e700c3212eeac))
* generate additional transparent addresses ([#61](https://github.com/hhanh00/zkool2/issues/61)) ([93a85b2](https://github.com/hhanh00/zkool2/commit/93a85b2278e8c5f246cad802a3edfa4911ca5164))
* logging framework ([#67](https://github.com/hhanh00/zkool2/issues/67)) ([aa83e32](https://github.com/hhanh00/zkool2/commit/aa83e3249d355778edaf3f4b3488514f3f937f1c))
* scan last 5 receive and change transparent addresses ([#62](https://github.com/hhanh00/zkool2/issues/62)) ([22403b8](https://github.com/hhanh00/zkool2/commit/22403b804be93affb56249eeb3e0177f3c9c8279))
* send memo ([#59](https://github.com/hhanh00/zkool2/issues/59)) ([3d75a82](https://github.com/hhanh00/zkool2/commit/3d75a8272ed236ea9b39d865b15f81b454ba9863))
* show diversified addresses ([#63](https://github.com/hhanh00/zkool2/issues/63)) ([ed8383f](https://github.com/hhanh00/zkool2/commit/ed8383f12f08efbe81013c41b76d23784e19339b))
* show/hide accounts ([#69](https://github.com/hhanh00/zkool2/issues/69)) ([647d521](https://github.com/hhanh00/zkool2/commit/647d52159096f67d847b9810108be01bfd61e45f))
* split tab views for transactions and memos ([#66](https://github.com/hhanh00/zkool2/issues/66)) ([ecf7395](https://github.com/hhanh00/zkool2/commit/ecf7395ebecb30595b8ee4aef579eba6e0c01bfb))


### Bug Fixes

* add more logging messages ([#68](https://github.com/hhanh00/zkool2/issues/68)) ([c94eed5](https://github.com/hhanh00/zkool2/commit/c94eed5d0b33d932449bfa0007da4b2100e31727))
* tx building when recipient pays fees ([150e063](https://github.com/hhanh00/zkool2/commit/150e0634bff3ae608654bdc86b5e700c3212eeac))

## [1.4.0](https://github.com/hhanh00/zkool2/compare/zkool-v1.3.0...zkool-v1.4.0) (2025-04-11)


### Features

* fetch memos ([#58](https://github.com/hhanh00/zkool2/issues/58)) ([8e56bae](https://github.com/hhanh00/zkool2/commit/8e56bae9091d2a1c009a2ca2796477dd875ae777))
* independent account synchronization ([#54](https://github.com/hhanh00/zkool2/issues/54)) ([8e61b14](https://github.com/hhanh00/zkool2/commit/8e61b14d790874534c44e3cffa786edd38860465))
* show transaction history ([#56](https://github.com/hhanh00/zkool2/issues/56)) ([6ac7957](https://github.com/hhanh00/zkool2/commit/6ac79576406c911a0939a58b07cc4abe76f1dbcc))

## [1.3.0](https://github.com/hhanh00/zkool2/compare/zkool-v1.2.0...zkool-v1.3.0) (2025-04-10)


### Features

* build and send transaction ([#53](https://github.com/hhanh00/zkool2/issues/53)) ([51b31ae](https://github.com/hhanh00/zkool2/commit/51b31ae7686a56a5493be874dc112a5dd8fdeb1a))
* show tx plan ([#52](https://github.com/hhanh00/zkool2/issues/52)) ([f180159](https://github.com/hhanh00/zkool2/commit/f1801595fc5e7387f9ef8d45004285e2f7c2a7be))


### Bug Fixes

* github workflow ([#49](https://github.com/hhanh00/zkool2/issues/49)) ([057a021](https://github.com/hhanh00/zkool2/commit/057a021dbec6a455be8fded396cfff9a7eaba47c))
* pczt tx building order ([#51](https://github.com/hhanh00/zkool2/issues/51)) ([989bc3a](https://github.com/hhanh00/zkool2/commit/989bc3a61531f9fbefe3f20a09e396ad09c8fe1b))
* upload release permission ([057a021](https://github.com/hhanh00/zkool2/commit/057a021dbec6a455be8fded396cfff9a7eaba47c))

## [1.2.0](https://github.com/hhanh00/zkool2/compare/zkool-v1.1.0...zkool-v1.2.0) (2025-04-09)


### Features

* add fees manager ([#42](https://github.com/hhanh00/zkool2/issues/42)) ([f02ddf3](https://github.com/hhanh00/zkool2/commit/f02ddf3c4b5e2dcfbfe1f5d7938c3952a10a3fe1))
* add seed fingerprint ([#40](https://github.com/hhanh00/zkool2/issues/40)) ([e2d2997](https://github.com/hhanh00/zkool2/commit/e2d29971b820f9dd07574fd27ce220a30d37304b))
* determine what pool to use for the change ([#41](https://github.com/hhanh00/zkool2/issues/41)) ([af1a6c7](https://github.com/hhanh00/zkool2/commit/af1a6c755f2108c9a1a6e9d9faee133398d7cf62))
* pczt builder ([#38](https://github.com/hhanh00/zkool2/issues/38)) ([7f18457](https://github.com/hhanh00/zkool2/commit/7f18457a5aa2e51b6f8cb3be2efa5fe7cea361ab))
* pczt builder ([#48](https://github.com/hhanh00/zkool2/issues/48)) ([5bd03d2](https://github.com/hhanh00/zkool2/commit/5bd03d265b29d833df63e5c6d99ee355826795fa))
* transaction planner ([#43](https://github.com/hhanh00/zkool2/issues/43)) ([4e6120d](https://github.com/hhanh00/zkool2/commit/4e6120d88f3ee90a746a47cc6621c72e7655728c))


### Bug Fixes

* (tx prepare) complete note assignment ([#45](https://github.com/hhanh00/zkool2/issues/45)) ([a1e7582](https://github.com/hhanh00/zkool2/commit/a1e7582056057083e4870e2ffb539a0fea7f48b0))
* (tx prepare) read unspent notes from db ([#44](https://github.com/hhanh00/zkool2/issues/44)) ([c30af46](https://github.com/hhanh00/zkool2/commit/c30af46ca4ca88a77dd6eb93045ca082f380c9dc))
* add id_taddress to transparent notes ([#46](https://github.com/hhanh00/zkool2/issues/46)) ([d4b7386](https://github.com/hhanh00/zkool2/commit/d4b7386ecf675e2066372c6dcf1a941738c80502))
* add t/s/o inputs to tx builder ([#47](https://github.com/hhanh00/zkool2/issues/47)) ([cfac62e](https://github.com/hhanh00/zkool2/commit/cfac62eb7638a8c07eda331760d58bd57ca395d2))

## [1.1.0](https://github.com/hhanh00/zkool2/compare/zkool-v1.0.0...zkool-v1.1.0) (2025-04-07)


### Features

* account view page ([#19](https://github.com/hhanh00/zkool2/issues/19)) ([98ae49c](https://github.com/hhanh00/zkool2/commit/98ae49c4f97657f2ccfee03f702a59d5ea0174f8))
* calculate pool balances ([#23](https://github.com/hhanh00/zkool2/issues/23)) ([fbeab61](https://github.com/hhanh00/zkool2/commit/fbeab6108259ec98f01996fc0d1cfe3cad8b61d5))
* report sync progress to UI ([#36](https://github.com/hhanh00/zkool2/issues/36)) ([806f0bf](https://github.com/hhanh00/zkool2/commit/806f0bf5db93c851d667ebb9661a485a2049f99d))
* rewind to height (snap to earlier checkpoint) ([#33](https://github.com/hhanh00/zkool2/issues/33)) ([2957cc3](https://github.com/hhanh00/zkool2/commit/2957cc33dbc4d4695ab70b42415acce853687c0f))
* save checkpoint block headers ([#34](https://github.com/hhanh00/zkool2/issues/34)) ([5e2bd28](https://github.com/hhanh00/zkool2/commit/5e2bd2825e712dd50ff085e370ae8f1d365fcb3d))
* shielded sync ([#25](https://github.com/hhanh00/zkool2/issues/25)) ([a999305](https://github.com/hhanh00/zkool2/commit/a99930583145376f2ef7ed9516a53fbdae6d9b67))
* store shielded sync state in database ([#26](https://github.com/hhanh00/zkool2/issues/26)) ([3c18f64](https://github.com/hhanh00/zkool2/commit/3c18f64b45efbfc006b6e37693f8005db9de0c87))
* transparent sync ([#22](https://github.com/hhanh00/zkool2/issues/22)) ([8aabc4c](https://github.com/hhanh00/zkool2/commit/8aabc4c7dd443a006efe3e0e36ed74e7074a5fa8))


### Bug Fixes

* issues with synchronization ([#28](https://github.com/hhanh00/zkool2/issues/28)) ([cc75da2](https://github.com/hhanh00/zkool2/commit/cc75da296ebf8ad11c7478c6ea98d77d47683e20))
* reactivate transparent sync & fix issue with dups ([#31](https://github.com/hhanh00/zkool2/issues/31)) ([9199b4d](https://github.com/hhanh00/zkool2/commit/9199b4d33de0d5ca7fb11dc9f710e514ddb7917a))
* recover from partial sync ([#32](https://github.com/hhanh00/zkool2/issues/32)) ([5dbba88](https://github.com/hhanh00/zkool2/commit/5dbba88f026f2ab842ea2ba4a81e7386f5044f4b))
* remove account id argument, use set_account ([#24](https://github.com/hhanh00/zkool2/issues/24)) ([02eaced](https://github.com/hhanh00/zkool2/commit/02eaced380d57903bfc2fa5db10c894d1152d774))
* resolve transparent tx timestamp during the shielded scan ([#35](https://github.com/hhanh00/zkool2/issues/35)) ([498be85](https://github.com/hhanh00/zkool2/commit/498be85d057328bed0a1fe6ecadf978ea3067692))
* spend detection ([#29](https://github.com/hhanh00/zkool2/issues/29)) ([2298b5d](https://github.com/hhanh00/zkool2/commit/2298b5d64a0bdc8346a8f946251c15ee21c94c50))
* spend detection for notes with vout != 0 ([#30](https://github.com/hhanh00/zkool2/issues/30)) ([c44bdeb](https://github.com/hhanh00/zkool2/commit/c44bdebaeff9a4cc4861aaee6b5d6172c177b106))
* transparent sync ([#27](https://github.com/hhanh00/zkool2/issues/27)) ([f9582d2](https://github.com/hhanh00/zkool2/commit/f9582d27f74346df9ec26adb1a0f9f9c35d023b7))

## 1.0.0 (2025-04-04)


### Features

* account deletion ([#10](https://github.com/hhanh00/zkool2/issues/10)) ([8a964f0](https://github.com/hhanh00/zkool2/commit/8a964f0641022116b80acb821378a1f038c267f0))
* account edit properties ([#8](https://github.com/hhanh00/zkool2/issues/8)) ([277f633](https://github.com/hhanh00/zkool2/commit/277f633afcdaccb0b82936137020c9d0c4275bd6))
* account list data table ([#7](https://github.com/hhanh00/zkool2/issues/7)) ([57c3f03](https://github.com/hhanh00/zkool2/commit/57c3f03a67bc73c8865604801ebe845601466cb0))
* account reordering by drag and drop ([#11](https://github.com/hhanh00/zkool2/issues/11)) ([f3bcaf2](https://github.com/hhanh00/zkool2/commit/f3bcaf26bec307af7bff5e94036280f5979785b8))
* convert to UFVK, UA and individual receivers ([#4](https://github.com/hhanh00/zkool2/issues/4)) ([54c92a8](https://github.com/hhanh00/zkool2/commit/54c92a8d51ec35d9a64deb8dd2878d0e62a5377f))
* create new account with random seed ([#18](https://github.com/hhanh00/zkool2/issues/18)) ([01e8361](https://github.com/hhanh00/zkool2/commit/01e83613e929712c629b5e6de7538e78b337f38e))
* create sapling & orchard account data from seed ([#2](https://github.com/hhanh00/zkool2/issues/2)) ([2b47317](https://github.com/hhanh00/zkool2/commit/2b47317c1916d474342b39aad670ae094b4758d6))
* import of sapling keys (sk/vk), uvk and transparent sk ([#3](https://github.com/hhanh00/zkool2/issues/3)) ([fd3d99e](https://github.com/hhanh00/zkool2/commit/fd3d99ecd98c55c6890579a358bac511bfd77bfb))
* new account implementation ([#15](https://github.com/hhanh00/zkool2/issues/15)) ([00235ed](https://github.com/hhanh00/zkool2/commit/00235edf255f0f82a25a3c64e88650255272d90c))
* new account page ([#13](https://github.com/hhanh00/zkool2/issues/13)) ([4ab4345](https://github.com/hhanh00/zkool2/commit/4ab4345c25fc63d43eced9426d6386f8470bb383))


### Bug Fixes

* artifact upload ([2b47317](https://github.com/hhanh00/zkool2/commit/2b47317c1916d474342b39aad670ae094b4758d6))
* ci workflows ([#12](https://github.com/hhanh00/zkool2/issues/12)) ([57015e1](https://github.com/hhanh00/zkool2/commit/57015e15c4a4002ca307be966a06ee49e72607d1))
* continue work on ([#7](https://github.com/hhanh00/zkool2/issues/7)) ([#9](https://github.com/hhanh00/zkool2/issues/9)) ([f10430f](https://github.com/hhanh00/zkool2/commit/f10430f8d9c377908d592df56f0b0e8888f707dc))
* do not run ci on release-please ([f9aa589](https://github.com/hhanh00/zkool2/commit/f9aa589fd65920a0553e25888a42ce9cd4e9316b))
* new account from key ([#17](https://github.com/hhanh00/zkool2/issues/17)) ([415e4e2](https://github.com/hhanh00/zkool2/commit/415e4e2a3f00d44c98949f3f1b0c96ccd2948b7e))
* remove coin arg from api ([#14](https://github.com/hhanh00/zkool2/issues/14)) ([034adde](https://github.com/hhanh00/zkool2/commit/034addeea2722d38e8fe4d8f65c1ccd0e5ce526b))
* transparent address table should contain sk ([#16](https://github.com/hhanh00/zkool2/issues/16)) ([584a679](https://github.com/hhanh00/zkool2/commit/584a67950423349993cbf5e8953d03fee9ecfc20))
