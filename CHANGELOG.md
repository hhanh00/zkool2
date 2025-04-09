# Changelog

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
