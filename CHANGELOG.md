# Changelog

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
