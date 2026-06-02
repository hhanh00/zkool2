# Changelog

## [7.0.0](https://github.com/hhanh00/zkool2/compare/zkool-v6.15.0...zkool-v7.0.0) (2026-06-02)


### ⚠ BREAKING CHANGES

* db schema for categories and transaction value in fiat ([#524](https://github.com/hhanh00/zkool2/issues/524))
* folder and db versioning ([#468](https://github.com/hhanh00/zkool2/issues/468))

### Features

* "synchronize" returns current height ([#731](https://github.com/hhanh00/zkool2/issues/731)) ([a2a3759](https://github.com/hhanh00/zkool2/commit/a2a3759d6479423e15e35f23d6430bf9d5773bcd))
* accept transparent public keys ([#592](https://github.com/hhanh00/zkool2/issues/592)) ([421cb5c](https://github.com/hhanh00/zkool2/commit/421cb5c40124d1867062f609594ccfea0d7bd683))
* account folders ([#474](https://github.com/hhanh00/zkool2/issues/474)) ([4a57f84](https://github.com/hhanh00/zkool2/commit/4a57f843bc6a2a96b3e51c45e8b992facaaf7c87))
* account, pay, issuance, GraphQL, FROST, and warp decrypter updates ([0c3e183](https://github.com/hhanh00/zkool2/commit/0c3e1832b7f89ce1c17d5df5fb725d78d9556dac))
* add auto fx rate update flag ([#816](https://github.com/hhanh00/zkool2/issues/816)) ([398365f](https://github.com/hhanh00/zkool2/commit/398365f36a959cf11fb976e8f0244bdd33f8938c))
* add category to send form ([#537](https://github.com/hhanh00/zkool2/issues/537)) ([8125512](https://github.com/hhanh00/zkool2/commit/81255124fa3b57ea6a3730e54238e5fa91a5e821))
* add default categories ([#535](https://github.com/hhanh00/zkool2/issues/535)) ([388b541](https://github.com/hhanh00/zkool2/commit/388b541d4ebffff86f6206139eb1f481562086b7))
* add ed25519 keypair to dkg as round 0 for future message signing ([#900](https://github.com/hhanh00/zkool2/issues/900)) ([5695086](https://github.com/hhanh00/zkool2/commit/56950867e118d7550553a018f58e81c7b4c60dff))
* add edge from note to tx ([#733](https://github.com/hhanh00/zkool2/issues/733)) ([37222aa](https://github.com/hhanh00/zkool2/commit/37222aa963565e954fd9d2f8ae0ec8e6368d93c4))
* add flag "fast" that skips downloading tx details ([#851](https://github.com/hhanh00/zkool2/issues/851)) ([7015481](https://github.com/hhanh00/zkool2/commit/701548168eb8b9147c5bb2cd9434d14bf1be3c8f))
* add Flatpak support ([#766](https://github.com/hhanh00/zkool2/issues/766)) ([726c10c](https://github.com/hhanh00/zkool2/commit/726c10cf3d45ac446db0f04f7815324c002c71c0))
* add polling interval to coin config ([#727](https://github.com/hhanh00/zkool2/issues/727)) ([aa1936c](https://github.com/hhanh00/zkool2/commit/aa1936cf2f65dcbdb4d3ebf53779ff88d25b430b))
* add progress bar during scanning ([#801](https://github.com/hhanh00/zkool2/issues/801)) ([f410465](https://github.com/hhanh00/zkool2/commit/f4104659f9feb390091c66282d59479c6ca4347b))
* add quit election button ([#872](https://github.com/hhanh00/zkool2/issues/872)) ([67354e2](https://github.com/hhanh00/zkool2/commit/67354e20ebd54bcf9c06b1c9615f5c8804e02017))
* add rewind method to witness that brings it to ([#863](https://github.com/hhanh00/zkool2/issues/863)) ([fc9fc09](https://github.com/hhanh00/zkool2/commit/fc9fc093aa56b79536fa25110a73b47b6f497477))
* add serializers to commitment tree state ([#868](https://github.com/hhanh00/zkool2/issues/868)) ([4539d86](https://github.com/hhanh00/zkool2/commit/4539d86d3e8c2b1899fa0575a6f07431c735c5b3))
* add serializers, size to Edge ([#869](https://github.com/hhanh00/zkool2/issues/869)) ([1936ad1](https://github.com/hhanh00/zkool2/commit/1936ad18d773e956f491fdb8365ff2a6ce11847a))
* add support for computing the auth path of a witness at a prior position ([#860](https://github.com/hhanh00/zkool2/issues/860)) ([8cba0ab](https://github.com/hhanh00/zkool2/commit/8cba0ab2e926242aeb22295c4924e94172f1be5e))
* add support for NU6.1 on testnet ([#444](https://github.com/hhanh00/zkool2/issues/444)) ([19f92f0](https://github.com/hhanh00/zkool2/commit/19f92f04a6a2d2519482a29d12383ee3a7c70089))
* add total balance to get_balance ([#734](https://github.com/hhanh00/zkool2/issues/734)) ([095822b](https://github.com/hhanh00/zkool2/commit/095822b327e5482bcb411d2a5843cfbb1936549b))
* add warning when some notes are disabled ([#442](https://github.com/hhanh00/zkool2/issues/442)) ([5e911b8](https://github.com/hhanh00/zkool2/commit/5e911b81e704b6ff69d49a5320dc20807ea495d5))
* allow ledger account without sapling address ([#677](https://github.com/hhanh00/zkool2/issues/677)) ([2711d50](https://github.com/hhanh00/zkool2/commit/2711d5090286426a4b6e779727968ec5d434ae69))
* allow manual input of fx rate ([#526](https://github.com/hhanh00/zkool2/issues/526)) ([4072d91](https://github.com/hhanh00/zkool2/commit/4072d919532430631126e7d946b659dc597bf72b))
* button for deleting election data ([#806](https://github.com/hhanh00/zkool2/issues/806)) ([bf58224](https://github.com/hhanh00/zkool2/commit/bf582246c9c1fc819b4876f6cad03926188982d2))
* category chart ([#540](https://github.com/hhanh00/zkool2/issues/540)) ([3483aad](https://github.com/hhanh00/zkool2/commit/3483aadcabe000a4eb9b9e7a120894f76d671e1e))
* category chart ([#544](https://github.com/hhanh00/zkool2/issues/544)) ([912d386](https://github.com/hhanh00/zkool2/commit/912d38615e13126604093ca8c45844b6c156f03b))
* category editor ([#532](https://github.com/hhanh00/zkool2/issues/532)) ([bc7ec49](https://github.com/hhanh00/zkool2/commit/bc7ec496ef4e771d601ef490ee72ea76177054e7))
* cli config settings ([#737](https://github.com/hhanh00/zkool2/issues/737)) ([0bb508b](https://github.com/hhanh00/zkool2/commit/0bb508b498222ddd39b4fd0622fc889e79d9d4d5))
* coin voting functionality ([#808](https://github.com/hhanh00/zkool2/issues/808)) ([dc012ec](https://github.com/hhanh00/zkool2/commit/dc012ec6b62183502466c14f13a3576c68d289be))
* create new folders ([#472](https://github.com/hhanh00/zkool2/issues/472)) ([4ef37bd](https://github.com/hhanh00/zkool2/commit/4ef37bd23918a8c4494c744fb5e387a7398a0228))
* cumulative spending/income by category chart ([#546](https://github.com/hhanh00/zkool2/issues/546)) ([0adf0d4](https://github.com/hhanh00/zkool2/commit/0adf0d4bd87e8224da6eeb11f3402dbdb25058fc))
* dart vault impl placeholder ([#908](https://github.com/hhanh00/zkool2/issues/908)) ([6336eaa](https://github.com/hhanh00/zkool2/commit/6336eaabd6d9cbdeac9139e51a671076541bfb35))
* db schema for categories and transaction value in fiat ([#524](https://github.com/hhanh00/zkool2/issues/524)) ([7d5a731](https://github.com/hhanh00/zkool2/commit/7d5a731a96780a3bc33042dd43d3d13e03d650da))
* derive Ledger sapling from seed ([#696](https://github.com/hhanh00/zkool2/issues/696)) ([122b0b0](https://github.com/hhanh00/zkool2/commit/122b0b0cb12dc25dd5e1f50cffe41fac6bf14111))
* display fx rate in tx details ([#533](https://github.com/hhanh00/zkool2/issues/533)) ([3580eaf](https://github.com/hhanh00/zkool2/commit/3580eaf621e885a681a6d05cf94511ef9392631e))
* edit category in tx details ([#534](https://github.com/hhanh00/zkool2/issues/534)) ([259957d](https://github.com/hhanh00/zkool2/commit/259957d9a204656d49f3d58e8740bbd392eefd1f))
* edit tx price on details page ([#551](https://github.com/hhanh00/zkool2/issues/551)) ([a67002f](https://github.com/hhanh00/zkool2/commit/a67002f660ee9249348fe62a8d9a7dc129df1bd7))
* encrypt and save account keys to vault ([#912](https://github.com/hhanh00/zkool2/issues/912)) ([ce4317e](https://github.com/hhanh00/zkool2/commit/ce4317e5beb81e7bd3d0322cda023abf19dd2914))
* encrypt wallet file with age/zstd ([#579](https://github.com/hhanh00/zkool2/issues/579)) ([07756b2](https://github.com/hhanh00/zkool2/commit/07756b28f3336cb76c65da5f3639f7ecf94c5023))
* enter amount in USD ([#439](https://github.com/hhanh00/zkool2/issues/439)) ([6003513](https://github.com/hhanh00/zkool2/commit/6003513b8e29aab917d6c61e349fef4cf93b258e))
* export of tx/memos/notes to csv ([#562](https://github.com/hhanh00/zkool2/issues/562)) ([09a0c37](https://github.com/hhanh00/zkool2/commit/09a0c37fa457bf2fce91e49c0142b07d9a048f3a))
* fetch election from vote server ([#798](https://github.com/hhanh00/zkool2/issues/798)) ([c1b57e7](https://github.com/hhanh00/zkool2/commit/c1b57e7d968e6a67b707a9dbafac52ae31e5aa09))
* fetch election from vote server ([#799](https://github.com/hhanh00/zkool2/issues/799)) ([a9b00f2](https://github.com/hhanh00/zkool2/commit/a9b00f25ce268672dc45a2801b0b2fc2f705304f))
* Fetch tx details in the background ([#853](https://github.com/hhanh00/zkool2/issues/853)) ([df9764a](https://github.com/hhanh00/zkool2/commit/df9764aec79c90b551770109f188bad777546720))
* fill missing tx prices by querying Coin Gecko ([#530](https://github.com/hhanh00/zkool2/issues/530)) ([6dce369](https://github.com/hhanh00/zkool2/commit/6dce3691ee7422a65da883551c15fe7dcee6418b))
* folder and db versioning ([#468](https://github.com/hhanh00/zkool2/issues/468)) ([6a3465c](https://github.com/hhanh00/zkool2/commit/6a3465cc4e0d60693ffe2510a99693f9d4572616))
* folder pop up menu ([#471](https://github.com/hhanh00/zkool2/issues/471)) ([c34e77b](https://github.com/hhanh00/zkool2/commit/c34e77b9de1d111598ac3fc5e3adb14463d89527))
* generate diversified addresses for the Ledger ([8b2759f](https://github.com/hhanh00/zkool2/commit/8b2759f1bf9d49eb60b2daf1720741571bd73a38))
* get historical prices from CoinGecko ([#529](https://github.com/hhanh00/zkool2/issues/529)) ([360f520](https://github.com/hhanh00/zkool2/commit/360f5206b0492d93f508329fdc263ea9b7639a2e))
* google drive integration ([#907](https://github.com/hhanh00/zkool2/issues/907)) ([02d7712](https://github.com/hhanh00/zkool2/commit/02d7712d0f31e7bd9c11e61fec114d832dc46896))
* graphql query account main data ([#711](https://github.com/hhanh00/zkool2/issues/711)) ([979a4cd](https://github.com/hhanh00/zkool2/commit/979a4cdcad2dd0df7ae15ab9c2bd459de87cb2d5))
* **graphql:** account_by_id, transaction_by_id, and connections ([#724](https://github.com/hhanh00/zkool2/issues/724)) ([5fcd1f0](https://github.com/hhanh00/zkool2/commit/5fcd1f03066fdf7c18387197703e1b4cc8e62dde))
* **graphql:** add height & balance to account data ([#749](https://github.com/hhanh00/zkool2/issues/749)) ([05a4587](https://github.com/hhanh00/zkool2/commit/05a45873fd247b2c22348a7cea5d4fba982a128c))
* **graphql:** add outputs, memos, spends to tx details ([#750](https://github.com/hhanh00/zkool2/issues/750)) ([cc4846a](https://github.com/hhanh00/zkool2/commit/cc4846ae1439da9023b3860269b0192a6a020d3b))
* **graphql:** add scope, diversifier and address to notes ([#738](https://github.com/hhanh00/zkool2/issues/738)) ([4bac366](https://github.com/hhanh00/zkool2/commit/4bac3666930f070339ee05d7685a093da4098fd5))
* **graphql:** balance of account ([#715](https://github.com/hhanh00/zkool2/issues/715)) ([4eb2378](https://github.com/hhanh00/zkool2/commit/4eb2378edff12ba1e949785fbe20d27abd4ec1c5))
* **graphql:** CI ([#713](https://github.com/hhanh00/zkool2/issues/713)) ([fae3e41](https://github.com/hhanh00/zkool2/commit/fae3e416d90903a68d2446870e0e761da0a6968b))
* **graphql:** cold wallet ([#746](https://github.com/hhanh00/zkool2/issues/746)) ([ea2ec8d](https://github.com/hhanh00/zkool2/commit/ea2ec8da2eb4ea83992b3f2f56393e01e983eb82))
* **graphql:** create_account ([#716](https://github.com/hhanh00/zkool2/issues/716)) ([8444eef](https://github.com/hhanh00/zkool2/commit/8444eef6cf09c132d4d5161f5af83fc86abe0c72))
* **graphql:** dkg (no automation) ([#740](https://github.com/hhanh00/zkool2/issues/740)) ([7015934](https://github.com/hhanh00/zkool2/commit/701593451dd3c8f7bd5ffd9006a6785764c455d0))
* **graphql:** dkg automation ([#741](https://github.com/hhanh00/zkool2/issues/741)) ([7e8b775](https://github.com/hhanh00/zkool2/commit/7e8b775f2d6bfba59f8990065c8fa38565cf6ed7))
* **graphql:** edit/delete account, current_height ([#717](https://github.com/hhanh00/zkool2/issues/717)) ([17a5fdc](https://github.com/hhanh00/zkool2/commit/17a5fdc410392d4dc1fb5c35fec39c5d8fb13dd0))
* **graphql:** frost signature ([#744](https://github.com/hhanh00/zkool2/issues/744)) ([e955e8f](https://github.com/hhanh00/zkool2/commit/e955e8f5df0ba5e712c9a0f7422ac0f6cbe5f5a9))
* **graphql:** frost signing automation ([#745](https://github.com/hhanh00/zkool2/issues/745)) ([e5f78ca](https://github.com/hhanh00/zkool2/commit/e5f78ca8b829f5bdd76baa25a127aa52f48aef5e))
* **graphql:** get_addresses ([#719](https://github.com/hhanh00/zkool2/issues/719)) ([c15635b](https://github.com/hhanh00/zkool2/commit/c15635b5653808ee1182ef47b48009f40663dbc2))
* **graphql:** jwt authorization ([#794](https://github.com/hhanh00/zkool2/issues/794)) ([905c48d](https://github.com/hhanh00/zkool2/commit/905c48de0ec632f8e64b089c1b823d51b72ed974))
* **graphql:** list notes ([#721](https://github.com/hhanh00/zkool2/issues/721)) ([dd2e895](https://github.com/hhanh00/zkool2/commit/dd2e895a5ed4bf8fd4f7345cc62f7c8c43700c84))
* **graphql:** memos_by_transaction ([#714](https://github.com/hhanh00/zkool2/issues/714)) ([1d1ae1e](https://github.com/hhanh00/zkool2/commit/1d1ae1ea295831930a195eb525cc91b47c6aa56e))
* **graphql:** mempool monitoring, unconfirmed txs ([#723](https://github.com/hhanh00/zkool2/issues/723)) ([f14d7e2](https://github.com/hhanh00/zkool2/commit/f14d7e29ade1adde90ba488036171a4933e1d9cd))
* **graphql:** new addresses, get balance at height ([#722](https://github.com/hhanh00/zkool2/issues/722)) ([2c064b3](https://github.com/hhanh00/zkool2/commit/2c064b357bac1e2c2420121d59b87e181ce3107c))
* **graphql:** pczt decode in human readble form ([#743](https://github.com/hhanh00/zkool2/issues/743)) ([6c9afbf](https://github.com/hhanh00/zkool2/commit/6c9afbf7249ad7ad775aea4e2a319ba84d80fc43))
* **graphql:** prepare unsigned tx ([#742](https://github.com/hhanh00/zkool2/issues/742)) ([abd8258](https://github.com/hhanh00/zkool2/commit/abd82587e05aa0c34a5906388b4b6cbeb263c837))
* **graphql:** read/write scope ([#822](https://github.com/hhanh00/zkool2/issues/822)) ([c85635d](https://github.com/hhanh00/zkool2/commit/c85635d66494ce71da7b891d1da41879fc104977))
* **graphql:** send funds ([#720](https://github.com/hhanh00/zkool2/issues/720)) ([a1eda45](https://github.com/hhanh00/zkool2/commit/a1eda45775398769ae3932c139c5d88847566679))
* **graphql:** synchronize ([#718](https://github.com/hhanh00/zkool2/issues/718)) ([fc40425](https://github.com/hhanh00/zkool2/commit/fc40425adb41dcdf66ebb912f57e089485673908))
* **graphql:** transactions_by_account ([#712](https://github.com/hhanh00/zkool2/issues/712)) ([0680fab](https://github.com/hhanh00/zkool2/commit/0680fab018103422c61a84182a2020c05357faf1))
* import ledger accounts ([#594](https://github.com/hhanh00/zkool2/issues/594)) ([819ce0e](https://github.com/hhanh00/zkool2/commit/819ce0e98f2e89075909fc4e67ba0cc885463607))
* issuance note synthesis from per-note CompactBlock data ([51c4a78](https://github.com/hhanh00/zkool2/commit/51c4a78ea7863cf7196a9f562190fe28e43ee8a2))
* **ledger:** error handling ([#609](https://github.com/hhanh00/zkool2/issues/609)) ([4bc523f](https://github.com/hhanh00/zkool2/commit/4bc523f1a2599f3bd471c9269ffeaea6be7560e9))
* **ledger:** error when tx has too many I/O ([#611](https://github.com/hhanh00/zkool2/issues/611)) ([3d91f9a](https://github.com/hhanh00/zkool2/commit/3d91f9a5eb18f6ed3fa97e65d69713c4eae051f3))
* **ledger:** Ledger integration ([#591](https://github.com/hhanh00/zkool2/issues/591)) ([79bdbb4](https://github.com/hhanh00/zkool2/commit/79bdbb4c3e47d02da2c08bd2ef1a7b52de16ba81))
* **ledger:** move zemu under feature flag ([#610](https://github.com/hhanh00/zkool2/issues/610)) ([789cc4d](https://github.com/hhanh00/zkool2/commit/789cc4d878a33d7c36c944861855ccce1b55e795))
* **ledger:** save/restore hw account ([#616](https://github.com/hhanh00/zkool2/issues/616)) ([303e886](https://github.com/hhanh00/zkool2/commit/303e886507e03ce1b278cc877ba5711b919af4b2))
* **ledger:** scan transparent addresses ([#607](https://github.com/hhanh00/zkool2/issues/607)) ([bdc9032](https://github.com/hhanh00/zkool2/commit/bdc9032ccaced2c545f9d9ae3ec07289db48420b))
* **ledger:** show t/z address on device for verification ([#622](https://github.com/hhanh00/zkool2/issues/622)) ([95291d0](https://github.com/hhanh00/zkool2/commit/95291d00acbaa32910f8dfefcecc0b3d7dc70455))
* **ledger:** support for t2t and t2z ([#608](https://github.com/hhanh00/zkool2/issues/608)) ([9b0bed6](https://github.com/hhanh00/zkool2/commit/9b0bed6a29bad6657ffca2c2500a791627d2e3fa))
* **ledger:** support transparent addresses ([#606](https://github.com/hhanh00/zkool2/issues/606)) ([071de71](https://github.com/hhanh00/zkool2/commit/071de71ac713e738a8f4d6ce39716657993d5d75))
* link to block explorer ([#461](https://github.com/hhanh00/zkool2/issues/461)) ([9aea7fd](https://github.com/hhanh00/zkool2/commit/9aea7fd92b206337ae33659c781014c4c40d1ebd))
* MCP server ([#992](https://github.com/hhanh00/zkool2/issues/992)) ([bab0548](https://github.com/hhanh00/zkool2/commit/bab05482407e8d6c083aeea4e4e4d35ca45c8c75))
* navigation buttons on tx details ([#556](https://github.com/hhanh00/zkool2/issues/556)) ([475c5f3](https://github.com/hhanh00/zkool2/commit/475c5f3b0c0b1a5c473f88d2ae4fe1e3190f6010))
* passkey support for the key vault ([#914](https://github.com/hhanh00/zkool2/issues/914)) ([a59b135](https://github.com/hhanh00/zkool2/commit/a59b1357c1a9687d67354ed703cfd6a36a304157))
* pir integration ([#871](https://github.com/hhanh00/zkool2/issues/871)) ([ad67cef](https://github.com/hhanh00/zkool2/commit/ad67cefe5aa72a99358762923cae9d0263f98bde))
* QR code transmission ([#680](https://github.com/hhanh00/zkool2/issues/680)) ([01fe85d](https://github.com/hhanh00/zkool2/commit/01fe85dadb868f15b2c750e8fe8b3583eb7d83f9))
* reconcile pending tx price/category with real tx ([#538](https://github.com/hhanh00/zkool2/issues/538)) ([87be612](https://github.com/hhanh00/zkool2/commit/87be61295b565a16b9ea08c4bb7de1586458b360))
* remove rocket, add warp ([#726](https://github.com/hhanh00/zkool2/issues/726)) ([da1dbea](https://github.com/hhanh00/zkool2/commit/da1dbeae595091bd815bebac291c292d96d81942))
* remove voting feature ([#993](https://github.com/hhanh00/zkool2/issues/993)) ([6ca1d42](https://github.com/hhanh00/zkool2/commit/6ca1d4214f0f0eee74e2f1089adad368797de26c))
* rename/delete folders ([#473](https://github.com/hhanh00/zkool2/issues/473)) ([40ecb72](https://github.com/hhanh00/zkool2/commit/40ecb72bbbcb3aeb398e0e557ff781674322db79))
* retrieve and display tx category ([#531](https://github.com/hhanh00/zkool2/issues/531)) ([fb4d5e8](https://github.com/hhanh00/zkool2/commit/fb4d5e89bd0731a5fb7224df1c6a088af6f41306))
* save pending tx category & fx rate ([#527](https://github.com/hhanh00/zkool2/issues/527)) ([45a76a2](https://github.com/hhanh00/zkool2/commit/45a76a2cc770f7c70c9fcb2ac6f7fe862927d692))
* save/load categories & tx price to file ([#539](https://github.com/hhanh00/zkool2/issues/539)) ([76ff8db](https://github.com/hhanh00/zkool2/commit/76ff8dbb06c857fef94df90389ace5d681685f36))
* scan existing notes to compute voting power ([#800](https://github.com/hhanh00/zkool2/issues/800)) ([c436450](https://github.com/hhanh00/zkool2/commit/c4364503ba8ae9abc4293e0d865e81c6f03a7e97))
* send tx with Ledger ([#595](https://github.com/hhanh00/zkool2/issues/595)) ([f7d7a29](https://github.com/hhanh00/zkool2/commit/f7d7a29cc8f57788c9b60594f6da7c30d7bd3db4))
* set master vault password api ([#910](https://github.com/hhanh00/zkool2/issues/910)) ([0d6b1e9](https://github.com/hhanh00/zkool2/commit/0d6b1e9f5187f8187df162c57afb22597d592bba))
* show accounts that were sync more than 30 mins ago in red ([#495](https://github.com/hhanh00/zkool2/issues/495)) ([d3b0c99](https://github.com/hhanh00/zkool2/commit/d3b0c999050032da0047d2b7e74783e581f1cab9))
* show block timestamp of account synced height ([#498](https://github.com/hhanh00/zkool2/issues/498)) ([6331ccd](https://github.com/hhanh00/zkool2/commit/6331ccddb4694acded3874909516806e3a6825b1))
* show confirm dialog when restoring without birth height ([#431](https://github.com/hhanh00/zkool2/issues/431)) ([eee423c](https://github.com/hhanh00/zkool2/commit/eee423c1fb96f72757daf639d39894706da0ace5))
* sign and verify Frost Messages ([#901](https://github.com/hhanh00/zkool2/issues/901)) ([249d8e2](https://github.com/hhanh00/zkool2/commit/249d8e24d656c7fadcc2e4a33f2c020d50a65918))
* spending/income chart ([#541](https://github.com/hhanh00/zkool2/issues/541)) ([cc28372](https://github.com/hhanh00/zkool2/commit/cc28372eaaadf1a78348f91c2085303228acf1ac))
* submit ballot ([#804](https://github.com/hhanh00/zkool2/issues/804)) ([bd7324b](https://github.com/hhanh00/zkool2/commit/bd7324bf83ff4f051fa0a1f010ab566e4181bcae))
* subscription channels for block/tx events ([#729](https://github.com/hhanh00/zkool2/issues/729)) ([1596769](https://github.com/hhanh00/zkool2/commit/1596769bb196ae608999215b61e26bf4246bfa09))
* subscription to tx and new blocks ([#730](https://github.com/hhanh00/zkool2/issues/730)) ([4b32486](https://github.com/hhanh00/zkool2/commit/4b32486ce0046511e82ec12a787a041e16fa3798))
* support uncompressed transparent private keys (5XXX) ([#973](https://github.com/hhanh00/zkool2/issues/973)) ([a5493d0](https://github.com/hhanh00/zkool2/commit/a5493d062008bdc6ccee1d5d555608f8c20509cc))
* synchronize with voting chain ([#805](https://github.com/hhanh00/zkool2/issues/805)) ([b1695db](https://github.com/hhanh00/zkool2/commit/b1695db24f0322484151b0fd219e295af25f285b))
* transparent scan for addresses page ([#575](https://github.com/hhanh00/zkool2/issues/575)) ([1e61e6c](https://github.com/hhanh00/zkool2/commit/1e61e6cc6fc5e44e49b711dc2d37fd650fdf7307))
* tx account update ([#812](https://github.com/hhanh00/zkool2/issues/812)) ([4e16fa4](https://github.com/hhanh00/zkool2/commit/4e16fa434633a94d5f2dfdb08bdaeccb6264bbc3))
* tx amount by date scatter chart ([#545](https://github.com/hhanh00/zkool2/issues/545)) ([988b72d](https://github.com/hhanh00/zkool2/commit/988b72d31fbe2d3e45b8721a43065f00fc7d1894))
* unlock all notes & lock based on maturity ([#564](https://github.com/hhanh00/zkool2/issues/564)) ([6f818ec](https://github.com/hhanh00/zkool2/commit/6f818ec96715e3b9864f1b8ef970ab0c5b852f73))
* use the best source pool for the change ([#708](https://github.com/hhanh00/zkool2/issues/708)) ([1ec0589](https://github.com/hhanh00/zkool2/commit/1ec0589444d0d9590cf570fce7c5207bf3e86040))
* use warp as the web server ([#728](https://github.com/hhanh00/zkool2/issues/728)) ([4f60dbe](https://github.com/hhanh00/zkool2/commit/4f60dbe0c04bf4a1862200f556048fd19d603207))
* vault impl in dart ([#909](https://github.com/hhanh00/zkool2/issues/909)) ([c49f28e](https://github.com/hhanh00/zkool2/commit/c49f28ee63b5880fba2e272b803150b3b2cb9477))
* vault master key implementation ([#911](https://github.com/hhanh00/zkool2/issues/911)) ([8c8bb92](https://github.com/hhanh00/zkool2/commit/8c8bb92bf59cf755c94ab46faf21a83d5f22dcff))
* vault recovery from master password ([#913](https://github.com/hhanh00/zkool2/issues/913)) ([82df6a1](https://github.com/hhanh00/zkool2/commit/82df6a11c1d715bb1fab32267dbd6e1479d59da8))
* vote delegation ([#810](https://github.com/hhanh00/zkool2/issues/810)) ([6ae7216](https://github.com/hhanh00/zkool2/commit/6ae7216c2dd65630a49011958447114ec8deaea3))
* voting form ([#803](https://github.com/hhanh00/zkool2/issues/803)) ([6b87718](https://github.com/hhanh00/zkool2/commit/6b87718c964c5f2affbbbdc6313898cca5a5844e))
* ZSA holdings, issuance, send support + fix split-spend signing ([7b095eb](https://github.com/hhanh00/zkool2/commit/7b095eb877657b97b3700bf60906ecaf324a2982))
* ZSA-aware transaction history (list + detail view) ([e757e59](https://github.com/hhanh00/zkool2/commit/e757e59a75f6e51104fe4d26a67940b29fd0a8ba))


### Bug Fixes

* account for locked notes in max amount calculation ([#565](https://github.com/hhanh00/zkool2/issues/565)) ([dbc4de6](https://github.com/hhanh00/zkool2/commit/dbc4de639ed70465fd698d8e21120af202c4fbfb))
* account list ui update ([#815](https://github.com/hhanh00/zkool2/issues/815)) ([81f0b6a](https://github.com/hhanh00/zkool2/commit/81f0b6a23d0d5369b27ff9241e414d0a4dedd525))
* account navigation ([#658](https://github.com/hhanh00/zkool2/issues/658)) ([9c88ef3](https://github.com/hhanh00/zkool2/commit/9c88ef3544604b47492728d1360fd76e2b870bca))
* account tx history not refreshing after sync ([#675](https://github.com/hhanh00/zkool2/issues/675)) ([9864d13](https://github.com/hhanh00/zkool2/commit/9864d131e7aad893ff74997de02050333e2125d1))
* add a confirmation prompt ([#417](https://github.com/hhanh00/zkool2/issues/417)) ([1597bba](https://github.com/hhanh00/zkool2/commit/1597bbaf350f59fb62a0fe00aa0d20740e842c82))
* add anchor corruption detection ([#648](https://github.com/hhanh00/zkool2/issues/648)) ([b3b4501](https://github.com/hhanh00/zkool2/commit/b3b4501a97440746f99df218ed3c3d71f5901774))
* add confirmation/explanation messages ([#915](https://github.com/hhanh00/zkool2/issues/915)) ([c8c1cce](https://github.com/hhanh00/zkool2/commit/c8c1cce3c7941cba38adee773e478208707ef5e8))
* add db check ([#644](https://github.com/hhanh00/zkool2/issues/644)) ([552233d](https://github.com/hhanh00/zkool2/commit/552233d872bef2564422b60569069c8e7a330025))
* add debugging messages ([#647](https://github.com/hhanh00/zkool2/issues/647)) ([a5725ca](https://github.com/hhanh00/zkool2/commit/a5725ca2f56866e4770b94f1e0d0dd94705e5143))
* add diversifier index to new_addresses and return from unconfirmed by account ([#881](https://github.com/hhanh00/zkool2/issues/881)) ([8dfaf02](https://github.com/hhanh00/zkool2/commit/8dfaf029c2176e513f908b66ca867ccd7ae4e94d))
* add error handling ([#817](https://github.com/hhanh00/zkool2/issues/817)) ([a1cdf60](https://github.com/hhanh00/zkool2/commit/a1cdf60ad7044c67c50b834e3a8297f40eea0825))
* add expert mode flag and gate the vault behind it ([#957](https://github.com/hhanh00/zkool2/issues/957)) ([ac2dcd7](https://github.com/hhanh00/zkool2/commit/ac2dcd7969101c063b15d92f49b5c58102a941dc))
* add ledger-recovery tool ([#676](https://github.com/hhanh00/zkool2/issues/676)) ([7836e0a](https://github.com/hhanh00/zkool2/commit/7836e0ad4835cf853102daccf10ef28a54ea3133))
* add logging messages and fix passkey on android ([#916](https://github.com/hhanh00/zkool2/issues/916)) ([47b0a8d](https://github.com/hhanh00/zkool2/commit/47b0a8d02118ed3bef4c8a62806a3738119eefd3))
* add message when wallet is offline ([#763](https://github.com/hhanh00/zkool2/issues/763)) ([606883e](https://github.com/hhanh00/zkool2/commit/606883e7bfee306db5faf4dd8f520499582e031a))
* add mining to dkg, frost loop ([#987](https://github.com/hhanh00/zkool2/issues/987)) ([b58583f](https://github.com/hhanh00/zkool2/commit/b58583fd5ab0a1bda9654d4712e8220ca850a985))
* add more info to mempool txs ([#877](https://github.com/hhanh00/zkool2/issues/877)) ([adcaaf3](https://github.com/hhanh00/zkool2/commit/adcaaf3fbe2ecbc0cb2d776fbbd70cc6ecc72215))
* add repeated password and validation to forms ([#572](https://github.com/hhanh00/zkool2/issues/572)) ([018cba3](https://github.com/hhanh00/zkool2/commit/018cba3eaab8c3b1334f27354dfa724cf4c91809))
* add some extra padding ([#956](https://github.com/hhanh00/zkool2/issues/956)) ([62eded9](https://github.com/hhanh00/zkool2/commit/62eded9b95f8d64e4004870fee3927f0a47d1374))
* add timestamp to vault log entry ([#921](https://github.com/hhanh00/zkool2/issues/921)) ([44c5700](https://github.com/hhanh00/zkool2/commit/44c570048e0de68e417e06d580c398009b94088c))
* add try/catch around rust code ([#963](https://github.com/hhanh00/zkool2/issues/963)) ([f41f9d9](https://github.com/hhanh00/zkool2/commit/f41f9d9c7a9c6ddda72fedcf67b1993982d2104b))
* add warning when server is running without JWT auth ([#977](https://github.com/hhanh00/zkool2/issues/977)) ([8c953a9](https://github.com/hhanh00/zkool2/commit/8c953a9817c36c667937a3e4454e664e5006dd22))
* add white background to icon ([#503](https://github.com/hhanh00/zkool2/issues/503)) ([9332574](https://github.com/hhanh00/zkool2/commit/9332574dcc8f280f30202a572d866530b3cc41ee))
* aindex not saved for ledger accounts ([2711d50](https://github.com/hhanh00/zkool2/commit/2711d5090286426a4b6e779727968ec5d434ae69))
* allow admin user to execute any command ([#796](https://github.com/hhanh00/zkool2/issues/796)) ([117ec14](https://github.com/hhanh00/zkool2/commit/117ec147e8dd85bec583c2892876776372d9fd4d))
* allow platform + cross-platform by removing authenticatorAttachment ([#941](https://github.com/hhanh00/zkool2/issues/941)) ([185accb](https://github.com/hhanh00/zkool2/commit/185accb4292c2531d23710f3f2ffbbd1cdd489ad))
* allow removal of account icon ([#434](https://github.com/hhanh00/zkool2/issues/434)) ([b84d859](https://github.com/hhanh00/zkool2/commit/b84d859ecb06f0efcb9a546b899686c8f64ad862))
* alpine base image for docker ([#789](https://github.com/hhanh00/zkool2/issues/789)) ([4e99ba5](https://github.com/hhanh00/zkool2/commit/4e99ba53d4df396e6d1c5c2fc82afc87762a5809))
* amount input widget ([#440](https://github.com/hhanh00/zkool2/issues/440)) ([2a2d853](https://github.com/hhanh00/zkool2/commit/2a2d853ba9aaa858b46433583a0a83a153cef29a))
* android 16k page alignment for rive & camera ([#488](https://github.com/hhanh00/zkool2/issues/488)) ([#490](https://github.com/hhanh00/zkool2/issues/490)) ([64676c9](https://github.com/hhanh00/zkool2/commit/64676c9e1dd652fffb5d53f1406f2150c758b7c6))
* android build break ([#770](https://github.com/hhanh00/zkool2/issues/770)) ([6d1a6bb](https://github.com/hhanh00/zkool2/commit/6d1a6bbfd1feb94a912b93e80ad4f64fd428f297))
* **android:** propagate CargoKit rustflags for zcash_unstable nu7 ([2df0c03](https://github.com/hhanh00/zkool2/commit/2df0c03dec52c6f21347e1c6b358b2af5776fc20))
* app resize ([#661](https://github.com/hhanh00/zkool2/issues/661)) ([ebd3ddf](https://github.com/hhanh00/zkool2/commit/ebd3ddf4c507ab55f58efa8d3ec9be4b9a56e45e))
* app state notification system mobx -&gt; riverpod ([#657](https://github.com/hhanh00/zkool2/issues/657)) ([808e706](https://github.com/hhanh00/zkool2/commit/808e706212e52284eb6eb5c3a55954c6b4f4d3cb))
* appsettings ([#660](https://github.com/hhanh00/zkool2/issues/660)) ([b99dce4](https://github.com/hhanh00/zkool2/commit/b99dce46e803d847d84acb890272846d79c7b16d))
* assert in witness calculation ([#870](https://github.com/hhanh00/zkool2/issues/870)) ([bf1942e](https://github.com/hhanh00/zkool2/commit/bf1942ec0ebb0e94fb9de4c609b34978ebc0a418))
* authenticate jwt subscriptions ([#980](https://github.com/hhanh00/zkool2/issues/980)) ([34de3b2](https://github.com/hhanh00/zkool2/commit/34de3b237289cc7022594a5af5e34239bfca7190))
* autosync & mempool ([#659](https://github.com/hhanh00/zkool2/issues/659)) ([cf589f1](https://github.com/hhanh00/zkool2/commit/cf589f16186568eba1b4c5f0f25065fd9198d58f))
* bind to anyip ([#767](https://github.com/hhanh00/zkool2/issues/767)) ([b43b8ab](https://github.com/hhanh00/zkool2/commit/b43b8ab22611c93cbebbb0636397da56b8359b61))
* birth height before sapling activation ([#553](https://github.com/hhanh00/zkool2/issues/553)) ([f8ce532](https://github.com/hhanh00/zkool2/commit/f8ce5321c45672a1eab157e2018e8887ae62ba08))
* block time at birth height for new account ([#518](https://github.com/hhanh00/zkool2/issues/518)) ([8983dfb](https://github.com/hhanh00/zkool2/commit/8983dfbb8cb1e3683775a6e2b752746dc9ef280e))
* build break iOS ([#888](https://github.com/hhanh00/zkool2/issues/888)) ([7093fdf](https://github.com/hhanh00/zkool2/commit/7093fdf0a68c66a460a717cbed3cfa52e6dd4115))
* build break on CI macos ([#600](https://github.com/hhanh00/zkool2/issues/600)) ([366f88f](https://github.com/hhanh00/zkool2/commit/366f88f5505190dba05cca512311bd57f6d075ae))
* build break on mobile ([#598](https://github.com/hhanh00/zkool2/issues/598)) ([20512c4](https://github.com/hhanh00/zkool2/commit/20512c45a3c94f2baedf8fbcb07ae1cbcd3538b1))
* build break on mobile ([#624](https://github.com/hhanh00/zkool2/issues/624)) ([5ebc8cd](https://github.com/hhanh00/zkool2/commit/5ebc8cd32e9af61587bafade4c1b1ca2147fa001))
* build script for iso ([#636](https://github.com/hhanh00/zkool2/issues/636)) ([8ad8b7a](https://github.com/hhanh00/zkool2/commit/8ad8b7a5a2cdad5360d89582580b94127ccaa123))
* build warnings ([#736](https://github.com/hhanh00/zkool2/issues/736)) ([b9d253b](https://github.com/hhanh00/zkool2/commit/b9d253bced87f67e1b8ea62056bdb517080d4455))
* change logic of the next button in vote & add tooltips ([#824](https://github.com/hhanh00/zkool2/issues/824)) ([ffbbdf9](https://github.com/hhanh00/zkool2/commit/ffbbdf99aac7ee59255977d1c47930e0dc2df192))
* change of lwd ([#662](https://github.com/hhanh00/zkool2/issues/662)) ([ff7e416](https://github.com/hhanh00/zkool2/commit/ff7e4168f082f2b722932c4305085a090001e793))
* change password form has "repeat password" field ([#492](https://github.com/hhanh00/zkool2/issues/492)) ([6b3415a](https://github.com/hhanh00/zkool2/commit/6b3415aadb86219f54dc54297fbbc2983ce85a02))
* chart refresh ([#547](https://github.com/hhanh00/zkool2/issues/547)) ([31044d7](https://github.com/hhanh00/zkool2/commit/31044d7668e2a0f800b85100a955518335c626d0))
* chart sizes and margins ([#568](https://github.com/hhanh00/zkool2/issues/568)) ([db3e80d](https://github.com/hhanh00/zkool2/commit/db3e80d27ea9e96e4682709f28aee7366dc9c70d))
* check for missing witnesses and offer to resync ([#891](https://github.com/hhanh00/zkool2/issues/891)) ([03e74b1](https://github.com/hhanh00/zkool2/commit/03e74b1982a0b70d3be43878b9d99432a3bf5383))
* check that current account is associated with the vote ([#811](https://github.com/hhanh00/zkool2/issues/811)) ([18a5ab2](https://github.com/hhanh00/zkool2/commit/18a5ab233b589cc9431e91103bef42c35a9744ac))
* check_witness_consistency as debug only ([#968](https://github.com/hhanh00/zkool2/issues/968)) ([8c5d0ef](https://github.com/hhanh00/zkool2/commit/8c5d0ef5e03788d3b45f92244be035c2026634b2))
* coingecko api key required now ([#754](https://github.com/hhanh00/zkool2/issues/754)) ([39ae0d3](https://github.com/hhanh00/zkool2/commit/39ae0d3b67058978f74c90bc178ea8722487cc1a))
* conditional NU7 activation and Orchard proving key selection ([cca1567](https://github.com/hhanh00/zkool2/commit/cca15677739ccf789b5971fcd3fc22aaa3d28457))
* conversion from USD to ZEC does not take locale into consideration ([#597](https://github.com/hhanh00/zkool2/issues/597)) ([eaf8df1](https://github.com/hhanh00/zkool2/commit/eaf8df16a8664dde3ee16250feef7bbf2e982fd5))
* database encryption form ([#577](https://github.com/hhanh00/zkool2/issues/577)) ([546a6da](https://github.com/hhanh00/zkool2/commit/546a6dadc40734253c77ba7832cb38e130b64097))
* database manager button ([#427](https://github.com/hhanh00/zkool2/issues/427)) ([6962f08](https://github.com/hhanh00/zkool2/commit/6962f08d29dc09262d1b4524c74fe8a10b1ed891))
* db creation with no password ([#573](https://github.com/hhanh00/zkool2/issues/573)) ([1e1f8dd](https://github.com/hhanh00/zkool2/commit/1e1f8dde10a0caa1375789a0bf142ba888384be7))
* db escaping in change_db_password ([#883](https://github.com/hhanh00/zkool2/issues/883)) ([fa8b8da](https://github.com/hhanh00/zkool2/commit/fa8b8da284e806c26174f3b10a08ffcd387a282c))
* db schema upgrage ([#614](https://github.com/hhanh00/zkool2/issues/614)) ([b4a7a03](https://github.com/hhanh00/zkool2/commit/b4a7a03fd53f28d3492394a691408945ba5adfaf))
* db version check ([#543](https://github.com/hhanh00/zkool2/issues/543)) ([41d9576](https://github.com/hhanh00/zkool2/commit/41d9576712251e47488fc09467f7440617132cef))
* DEFAULT_TX_EXPIRY_DELTA is added by the pczt builder ([#829](https://github.com/hhanh00/zkool2/issues/829)) ([c161a75](https://github.com/hhanh00/zkool2/commit/c161a75949dc9dc0d4e7b5137cfe427add5178f4))
* disable android auto backup ([#932](https://github.com/hhanh00/zkool2/issues/932)) ([07517d2](https://github.com/hhanh00/zkool2/commit/07517d2bd6b66c3fa1c3e53fc3ad7df3c7313a18))
* disable passkeys on unsupported platforms ([#936](https://github.com/hhanh00/zkool2/issues/936)) ([ac681ea](https://github.com/hhanh00/zkool2/commit/ac681ead4c56572eb5bb6c3e061c83700c9c6b04))
* disclaimer page showing up twice ([#752](https://github.com/hhanh00/zkool2/issues/752)) ([b253ff4](https://github.com/hhanh00/zkool2/commit/b253ff45bbbe0374b47ba33c46c4e47f0ea1b5b4))
* dkg - handle error from server ([#515](https://github.com/hhanh00/zkool2/issues/515)) ([4cc301b](https://github.com/hhanh00/zkool2/commit/4cc301bd06c01554d9fc266689d56bc8e467bb8d))
* dkg error handling ([#433](https://github.com/hhanh00/zkool2/issues/433)) ([d280410](https://github.com/hhanh00/zkool2/commit/d280410c809cbd8309ca6448fcd0768c424e4d7e))
* do not add column if it exists ([#470](https://github.com/hhanh00/zkool2/issues/470)) ([b2ed51d](https://github.com/hhanh00/zkool2/commit/b2ed51db6e81ddd0bc7b38161c2d8aca668baef9))
* do not show saved confirmation if canceled ([#566](https://github.com/hhanh00/zkool2/issues/566)) ([b3aa17f](https://github.com/hhanh00/zkool2/commit/b3aa17f8515e776ce8ae5eab9316c1346236cfa2))
* do not show sync snackbar when app is in background ([#832](https://github.com/hhanh00/zkool2/issues/832)) ([4c95f8a](https://github.com/hhanh00/zkool2/commit/4c95f8a97261f156d5ea0096c162b10289b71205))
* do not update vault when disabled ([#947](https://github.com/hhanh00/zkool2/issues/947)) ([f9cf43d](https://github.com/hhanh00/zkool2/commit/f9cf43d439987c3afc9bbba09b1d5d713c5a8e6e))
* docker build ([#779](https://github.com/hhanh00/zkool2/issues/779)) ([8f8db55](https://github.com/hhanh00/zkool2/commit/8f8db555ff1c64be422e101789c2eee39f4fb8c8))
* don't fetch chart on linux because the webview isn't supported ([#415](https://github.com/hhanh00/zkool2/issues/415)) ([7d70350](https://github.com/hhanh00/zkool2/commit/7d7035098d717a5466086eaa0c62e448ffec5a9d))
* don't require pin if biometrics not available ([#679](https://github.com/hhanh00/zkool2/issues/679)) ([9ac4f26](https://github.com/hhanh00/zkool2/commit/9ac4f26246e678cbb8cd509620835eaf2cc3361d))
* drop support for 32-bit android due to build breaks ([#858](https://github.com/hhanh00/zkool2/issues/858)) ([d51dd97](https://github.com/hhanh00/zkool2/commit/d51dd97889347c10608e8163e894b01880bb26e4))
* duplicate GlobalKey ([#520](https://github.com/hhanh00/zkool2/issues/520)) ([199e9c2](https://github.com/hhanh00/zkool2/commit/199e9c24182f9e97c926e0903c2019febda30650))
* edit category ([#560](https://github.com/hhanh00/zkool2/issues/560)) ([69a3afe](https://github.com/hhanh00/zkool2/commit/69a3afe69d37b2db27b8c5f8e092c7bb56becb91))
* eliminate UI refresh "flash" at end of sync ([#958](https://github.com/hhanh00/zkool2/issues/958)) ([0714b2b](https://github.com/hhanh00/zkool2/commit/0714b2b089ffe7adae086e73aff6358f8bd0b8d9))
* error message when tx was broadcast correctly ([#479](https://github.com/hhanh00/zkool2/issues/479)) ([f19026b](https://github.com/hhanh00/zkool2/commit/f19026b71ea9d20ab7060e936d3cd8ce9845033f))
* escape db password and wrap in single quotes ([#840](https://github.com/hhanh00/zkool2/issues/840)) ([a35e72c](https://github.com/hhanh00/zkool2/commit/a35e72c279d24e7613cb83a232334b2105da008a))
* export category to tx csv as name ([#569](https://github.com/hhanh00/zkool2/issues/569)) ([c823157](https://github.com/hhanh00/zkool2/commit/c823157d144522bc5d814391f27e71fa70b50c4f))
* fetch election and import atomically ([#875](https://github.com/hhanh00/zkool2/issues/875)) ([51fb934](https://github.com/hhanh00/zkool2/commit/51fb93488ad3fbc7673926713812071421d74f57))
* fetch tx details from account manager ([#964](https://github.com/hhanh00/zkool2/issues/964)) ([aa4be1e](https://github.com/hhanh00/zkool2/commit/aa4be1e902411a9e007771b6fe7b0bc653f7c628))
* filter zero-value issuance notes in preprocessor, not try_decrypt ([64f3c57](https://github.com/hhanh00/zkool2/commit/64f3c57955aeaec736c62f3064ec0162b160a446))
* fix address qr scan ([#760](https://github.com/hhanh00/zkool2/issues/760)) ([1eda849](https://github.com/hhanh00/zkool2/commit/1eda849727b8d80710fffd71e79c9f0b1bfde7e8))
* fix issue with refresh of input amount widget ([#874](https://github.com/hhanh00/zkool2/issues/874)) ([ff895d8](https://github.com/hhanh00/zkool2/commit/ff895d8b28dd740efd38f139378c6b33e4459fe2))
* fix typos ([#451](https://github.com/hhanh00/zkool2/issues/451)) ([7b3f747](https://github.com/hhanh00/zkool2/commit/7b3f74792d036c1ef3ffea43326c0d7c4efc7f31))
* **frost:** add orchard_split_spend_indices to PcztPackage and sign them ([f22bd99](https://github.com/hhanh00/zkool2/commit/f22bd9913f79bf53490b78e83120dd0e2230552a))
* get block times of synced points ([#517](https://github.com/hhanh00/zkool2/issues/517)) ([3683c3b](https://github.com/hhanh00/zkool2/commit/3683c3b5976d4cb1ba9a5a44f4069644d72f27d1))
* get_notes by txid ([#747](https://github.com/hhanh00/zkool2/issues/747)) ([7cd1a44](https://github.com/hhanh00/zkool2/commit/7cd1a44bf8bce00227e63a93a83bc76933eb3218))
* handle "partial" payment uri like zcash:&lt;addr&gt;? ([#425](https://github.com/hhanh00/zkool2/issues/425)) ([303633f](https://github.com/hhanh00/zkool2/commit/303633fb2805f0b80fb0c5e89eb38ca62251e264))
* height off by 1 after reset ([#521](https://github.com/hhanh00/zkool2/issues/521)) ([09f92aa](https://github.com/hhanh00/zkool2/commit/09f92aa12d684445d2aefc1b35f1cd813a3b83da))
* http over tor ([#688](https://github.com/hhanh00/zkool2/issues/688)) ([95e4290](https://github.com/hhanh00/zkool2/commit/95e42906180a94338a200a3528549851e085843e))
* I/O of is_income in category table ([#571](https://github.com/hhanh00/zkool2/issues/571)) ([629c069](https://github.com/hhanh00/zkool2/commit/629c069764992ca1497ee11fcaefe9aa530c2234))
* import account ([4cc301b](https://github.com/hhanh00/zkool2/commit/4cc301bd06c01554d9fc266689d56bc8e467bb8d))
* incorrect config parsing. toml is overridden by command line args ([#975](https://github.com/hhanh00/zkool2/issues/975)) ([3a79842](https://github.com/hhanh00/zkool2/commit/3a79842245d2c666e8efce4b7dbc441c26d2e501))
* increase build number ([#788](https://github.com/hhanh00/zkool2/issues/788)) ([12c5925](https://github.com/hhanh00/zkool2/commit/12c59254c9ea5efbeacf07073e3991c71b5289eb))
* increase delay after voting from 2s to 15s before refreshing ([#835](https://github.com/hhanh00/zkool2/issues/835)) ([4235976](https://github.com/hhanh00/zkool2/commit/42359767f36a0188003561f29b17b6251802d9fb))
* io save/load assets + id_asset, skip zero-value issuance notes ([d651f6d](https://github.com/hhanh00/zkool2/commit/d651f6d7cf49eef79ec4484912017f0111804920))
* ios build ([#435](https://github.com/hhanh00/zkool2/issues/435)) ([5b46cc9](https://github.com/hhanh00/zkool2/commit/5b46cc96f076f28ebe0b5df26f863891b24e611d))
* iOS build ([#919](https://github.com/hhanh00/zkool2/issues/919)) ([ec3b1b8](https://github.com/hhanh00/zkool2/commit/ec3b1b8efa137e4cdda578cb54d6c3a659b12fdb))
* iOS Google signin to Drive ([#938](https://github.com/hhanh00/zkool2/issues/938)) ([3cf9433](https://github.com/hhanh00/zkool2/commit/3cf9433d215bcd685eb2d61b986600e677d09251))
* **ios:** propagate Cargo rustflags from .cargo/config.toml into CARGO_ENCODED_RUSTFLAGS ([945ebe6](https://github.com/hhanh00/zkool2/commit/945ebe6129f5cb9b825b0e76c00bd53b2b1cea3a))
* **ios:** search workspace .cargo/config.toml for rustflags when building pod ([cd1501f](https://github.com/hhanh00/zkool2/commit/cd1501f91f40e3c7bad041f06b5058cd22579342))
* lazily build Tor client ([#454](https://github.com/hhanh00/zkool2/issues/454)) ([2b9aaf3](https://github.com/hhanh00/zkool2/commit/2b9aaf3f00d7efeaee2256ac521e5a5d768e63ca))
* lazily sync with the vault ([#937](https://github.com/hhanh00/zkool2/issues/937)) ([e0c2df7](https://github.com/hhanh00/zkool2/commit/e0c2df740f22209bc433665d587e79097ff1c7f9))
* ledger build ([#686](https://github.com/hhanh00/zkool2/issues/686)) ([59768d3](https://github.com/hhanh00/zkool2/commit/59768d347dfd09c42b72b7a7b107c187b3f6a958))
* ledger new account ([#887](https://github.com/hhanh00/zkool2/issues/887)) ([9807b89](https://github.com/hhanh00/zkool2/commit/9807b89bf88aedc06027aea1de3ae602d6746509))
* **ledger:** build break on mobile platforms (no support for ledger) ([#613](https://github.com/hhanh00/zkool2/issues/613)) ([52d4792](https://github.com/hhanh00/zkool2/commit/52d47923ea878874a711c069bc612825ceddd8f2))
* **ledger:** device thread serialization ([#626](https://github.com/hhanh00/zkool2/issues/626)) ([ebf0890](https://github.com/hhanh00/zkool2/commit/ebf0890add73ab041c4fc924ff4d532d88385816))
* linear progress indicator ([#511](https://github.com/hhanh00/zkool2/issues/511)) ([d56b237](https://github.com/hhanh00/zkool2/commit/d56b237db0e64f51516e5df089d979a59ae35f5c))
* linux nix build ([#899](https://github.com/hhanh00/zkool2/issues/899)) ([80fdfb4](https://github.com/hhanh00/zkool2/commit/80fdfb4d8bd4ded83334e322443d62ff5ff66b59))
* lock pin ([#665](https://github.com/hhanh00/zkool2/issues/665)) ([3e73d2c](https://github.com/hhanh00/zkool2/commit/3e73d2cc44ff14c1486423b82cc12fa6fc18e208))
* lots of UI glitches ([#955](https://github.com/hhanh00/zkool2/issues/955)) ([78772cb](https://github.com/hhanh00/zkool2/commit/78772cb7065eb1c2532958d322f870fc9987b103))
* macos usb entitlements ([#620](https://github.com/hhanh00/zkool2/issues/620)) ([8e70296](https://github.com/hhanh00/zkool2/commit/8e70296ee2f0e464c083a00a4644da95f586bdb3))
* **macos:** replace Flutter SPM with CocoaPods resources and configure manual code signing ([ee187c9](https://github.com/hhanh00/zkool2/commit/ee187c947999dcf695001f7e6de043d2522dd027))
* make docker image multiplatform ([#785](https://github.com/hhanh00/zkool2/issues/785)) ([ba08222](https://github.com/hhanh00/zkool2/commit/ba08222031c33db596252a1cd231fcd19536ae66))
* make smoke test wait for blocks instead of sleep ([#855](https://github.com/hhanh00/zkool2/issues/855)) ([336eff0](https://github.com/hhanh00/zkool2/commit/336eff082093d4832e9c7b9d5ec3be499d756c25))
* missing update of lwd url ([#684](https://github.com/hhanh00/zkool2/issues/684)) ([61474cc](https://github.com/hhanh00/zkool2/commit/61474cc7266c1ceeab6af980277738add7b1ff66))
* move payment uri to extra options page ([#409](https://github.com/hhanh00/zkool2/issues/409)) ([2946894](https://github.com/hhanh00/zkool2/commit/2946894e6ca07bb832646962a95c10d2b2c5a8f0))
* new account from ledger ([#704](https://github.com/hhanh00/zkool2/issues/704)) ([facb142](https://github.com/hhanh00/zkool2/commit/facb1423072603b80fb7b8989142b7b04386d122))
* no ledger build break ([#629](https://github.com/hhanh00/zkool2/issues/629)) ([e39fffb](https://github.com/hhanh00/zkool2/commit/e39fffb8a0c80a942d399bbed077739b22ee45ec))
* no_mempool should be overridable ([#978](https://github.com/hhanh00/zkool2/issues/978)) ([c062563](https://github.com/hhanh00/zkool2/commit/c062563078a55fbcf339b1582a88e3db02a02edf))
* observe unconfirmed amount ([#464](https://github.com/hhanh00/zkool2/issues/464)) ([5a7f9da](https://github.com/hhanh00/zkool2/commit/5a7f9dad64b5933a0f06b0c8751f2280e9279a28))
* pass coin as parameter ([#682](https://github.com/hhanh00/zkool2/issues/682)) ([5640156](https://github.com/hhanh00/zkool2/commit/56401564d23f1c93dd5f6484b99b0b2941a78ee7))
* pass id_account to issue_asset, remove lock_note race, fix ZIP-32 account param ([1b739d1](https://github.com/hhanh00/zkool2/commit/1b739d1be8f7e0d13d46ee45147e58ced66d9ce3))
* **pay:** always emit per-asset change output and correct ZSA filter ([535bf7b](https://github.com/hhanh00/zkool2/commit/535bf7ba198f01be78e71e20082a975e553fd7e8))
* **pay:** ZSA fee estimation, input selection, and PCZT split spend support ([145e8f9](https://github.com/hhanh00/zkool2/commit/145e8f9c7b195d7eee22892f4f21d7a244cab2fe))
* pin to given flutter version ([#949](https://github.com/hhanh00/zkool2/issues/949)) ([f27bb8f](https://github.com/hhanh00/zkool2/commit/f27bb8f38e75c9700f9bbf14fb326a7582b7a770))
* pinlock ([#748](https://github.com/hhanh00/zkool2/issues/748)) ([cb26279](https://github.com/hhanh00/zkool2/commit/cb262798a5f06a61c48c754db031522806d08b17))
* pinlock + account icon ([#437](https://github.com/hhanh00/zkool2/issues/437)) ([4dc5d6f](https://github.com/hhanh00/zkool2/commit/4dc5d6f64db6ecefc1cf5515b04bad5ee159862e))
* pinlock always needed even when disabled in settings ([#678](https://github.com/hhanh00/zkool2/issues/678)) ([203e9f0](https://github.com/hhanh00/zkool2/commit/203e9f09389c4d32b1be07e1ba8f1e99a2c51fdc))
* pinlock on rest of the pages ([#666](https://github.com/hhanh00/zkool2/issues/666)) ([bd5bb9a](https://github.com/hhanh00/zkool2/commit/bd5bb9a0c3fcdd2a664b4024635ffaa5565b1c0a))
* progress bar ([#452](https://github.com/hhanh00/zkool2/issues/452)) ([fee6aa6](https://github.com/hhanh00/zkool2/commit/fee6aa6f1fc0b25fba69aba92527a569e1b73379))
* put the progress bar in a modal dialog box ([#802](https://github.com/hhanh00/zkool2/issues/802)) ([cd174a0](https://github.com/hhanh00/zkool2/commit/cd174a03de042dc145154cc7f1e11382d03f0ad5))
* reformat payment uri ([#413](https://github.com/hhanh00/zkool2/issues/413)) ([4d36a32](https://github.com/hhanh00/zkool2/commit/4d36a325a477f96a024b569f16a059760d3481ae))
* refresh after folder deletion ([#475](https://github.com/hhanh00/zkool2/issues/475)) ([0e3d04f](https://github.com/hhanh00/zkool2/commit/0e3d04fb98b1022597a4c77033d2847b3cc96339))
* remove db creation with password ([#419](https://github.com/hhanh00/zkool2/issues/419)) ([142c614](https://github.com/hhanh00/zkool2/commit/142c614d6ed3dacb409b5b0eada008c60a903896))
* remove dependency on connectivity_plus and use config setting ([#654](https://github.com/hhanh00/zkool2/issues/654)) ([20cc7bb](https://github.com/hhanh00/zkool2/commit/20cc7bb497890fc3cbd16cbff7a44b9ebbab0a93))
* remove dialog asking for scanning taddr on new accounts ([#582](https://github.com/hhanh00/zkool2/issues/582)) ([a8da6c8](https://github.com/hhanh00/zkool2/commit/a8da6c805cbeba13946edd14558fe138976ad4b5))
* remove dummy text ([ae9eccc](https://github.com/hhanh00/zkool2/commit/ae9eccc6bdd1b638ceddfed70de0907083355dd1))
* remove dust change policy ([#755](https://github.com/hhanh00/zkool2/issues/755)) ([9201776](https://github.com/hhanh00/zkool2/commit/92017767e2767c73e20f174bc3aa8b65701a462c))
* remove extra column from query ([#618](https://github.com/hhanh00/zkool2/issues/618)) ([b23a9c9](https://github.com/hhanh00/zkool2/commit/b23a9c9030610446eaefc7ff50f2a0affeac5e9f))
* remove Ledger NU6.1 warning since the app was updated ([#707](https://github.com/hhanh00/zkool2/issues/707)) ([8b39952](https://github.com/hhanh00/zkool2/commit/8b3995266771592c283a1128d4b703452bcf6e4b))
* remove out of band abort messages that could mess with the commit ([#646](https://github.com/hhanh00/zkool2/issues/646)) ([0cb6f42](https://github.com/hhanh00/zkool2/commit/0cb6f423f4f1a9ca1d5cb19910783452dc6c59e6))
* remove polling_interval from config ([#732](https://github.com/hhanh00/zkool2/issues/732)) ([73b0e24](https://github.com/hhanh00/zkool2/commit/73b0e24ff32070d45cf5faee6cc32649340977b3))
* remove redundant tooltip reset message ([#830](https://github.com/hhanh00/zkool2/issues/830)) ([81d152b](https://github.com/hhanh00/zkool2/commit/81d152b7f835309800738b485864c7cc3b19dcc3))
* remove some ui glitch ([#668](https://github.com/hhanh00/zkool2/issues/668)) ([c18608b](https://github.com/hhanh00/zkool2/commit/c18608b8dbb46c6199dc536bebb360ed2d5a623a))
* remove transparent receiver from default ua ([#735](https://github.com/hhanh00/zkool2/issues/735)) ([66cf36d](https://github.com/hhanh00/zkool2/commit/66cf36d426b6eb85da51c95b6447de694acf5d2d))
* remove unused file ([#486](https://github.com/hhanh00/zkool2/issues/486)) ([f18799b](https://github.com/hhanh00/zkool2/commit/f18799bf605462479c7de4b1aa8070c47d3adf99))
* removed display, zip212 grace period ([#966](https://github.com/hhanh00/zkool2/issues/966)) ([13fc225](https://github.com/hhanh00/zkool2/commit/13fc22557c825bb86ec72f5f3c1d960247542b68))
* reorganize menu items for charts ([#554](https://github.com/hhanh00/zkool2/issues/554)) ([5890df0](https://github.com/hhanh00/zkool2/commit/5890df0e8ab68e0106e17841ba4af6949f8c6007))
* replace zaino by lightwalletd ([#895](https://github.com/hhanh00/zkool2/issues/895)) ([14a3f90](https://github.com/hhanh00/zkool2/commit/14a3f908bd2d6e1bf39cd1c4f207944b43e18daa))
* reregister the passkey if it is stale ([#923](https://github.com/hhanh00/zkool2/issues/923)) ([ea75bb9](https://github.com/hhanh00/zkool2/commit/ea75bb97bc9a2db9bf694ec515361029fcf442ed))
* reset account should remove all tx data ([#559](https://github.com/hhanh00/zkool2/issues/559)) ([5970194](https://github.com/hhanh00/zkool2/commit/5970194a5c3b90a04319ae79963da768075f84b6))
* resize icon ([#505](https://github.com/hhanh00/zkool2/issues/505)) ([c4a3fcb](https://github.com/hhanh00/zkool2/commit/c4a3fcbbe3ae068667acef5cba8d170e882c43be))
* respond to focus events on address field ([#460](https://github.com/hhanh00/zkool2/issues/460)) ([7ea6d5f](https://github.com/hhanh00/zkool2/commit/7ea6d5f8f67cb4ad6f4f1ce27b495362e5b35f66))
* return error msg when no prf support ([#944](https://github.com/hhanh00/zkool2/issues/944)) ([d75f851](https://github.com/hhanh00/zkool2/commit/d75f851476b1b1a8b9a3f29083375b7cb854d188))
* rewind account ([#448](https://github.com/hhanh00/zkool2/issues/448)) ([926db85](https://github.com/hhanh00/zkool2/commit/926db85b4f8bf08244e0b6ec66e33e3706c906da))
* save current account between restarts ([#814](https://github.com/hhanh00/zkool2/issues/814)) ([dc72967](https://github.com/hhanh00/zkool2/commit/dc729676b95e22d161b4438d67bd2384682af328))
* save send form state between pinlocks ([#756](https://github.com/hhanh00/zkool2/issues/756)) ([4bac33f](https://github.com/hhanh00/zkool2/commit/4bac33fd71b1f75d37ffc9913cac96cc461e254f))
* send from transparent private key only account ([#501](https://github.com/hhanh00/zkool2/issues/501)) ([223fbbf](https://github.com/hhanh00/zkool2/commit/223fbbf46ef4732a2be369337e16dfafa4769e8f))
* sending to tex address ([#765](https://github.com/hhanh00/zkool2/issues/765)) ([6f6803d](https://github.com/hhanh00/zkool2/commit/6f6803daee7c695cb8e305292c273b46f7518bd6))
* separate list of categories by income/expense ([#561](https://github.com/hhanh00/zkool2/issues/561)) ([5628d36](https://github.com/hhanh00/zkool2/commit/5628d366a3d572191be9d01d2e0d7359b0b265f6))
* set net too early before db loaded ([#463](https://github.com/hhanh00/zkool2/issues/463)) ([a84ee5a](https://github.com/hhanh00/zkool2/commit/a84ee5ae7eb8c09d85b2601dcaf326d602d2a6e0))
* show warning when using ledger because of NU6.1 ([#691](https://github.com/hhanh00/zkool2/issues/691)) ([43dc8f7](https://github.com/hhanh00/zkool2/commit/43dc8f70464b714baea74f7e94d6490af2b4bcb8))
* skip zero-value issuance reference notes in wallet, keep cmx in tree ([2c86332](https://github.com/hhanh00/zkool2/commit/2c863322a8bbd5dd4637ec1d0191fc3f90c5429d))
* small ui bug ([#672](https://github.com/hhanh00/zkool2/issues/672)) ([6c72a4f](https://github.com/hhanh00/zkool2/commit/6c72a4ff6fc458208dac67ab5aa6a46c47ff700a))
* spending sapling internal notes ([#706](https://github.com/hhanh00/zkool2/issues/706)) ([c1d99a0](https://github.com/hhanh00/zkool2/commit/c1d99a0ded6a68c31f2ce496cb2c2676c9d45a10))
* store block header time ([#523](https://github.com/hhanh00/zkool2/issues/523)) ([8cb320d](https://github.com/hhanh00/zkool2/commit/8cb320dbfe34c417721f50127062071942a3a347))
* support for NU6.1 ([#683](https://github.com/hhanh00/zkool2/issues/683)) ([e31c46f](https://github.com/hhanh00/zkool2/commit/e31c46f0e52865ae07d0eeb3725ba49bbd44d210))
* support ledger memos ([#632](https://github.com/hhanh00/zkool2/issues/632)) ([fcf185f](https://github.com/hhanh00/zkool2/commit/fcf185f68d7df71f6135d05b6c6b35b27f812d61))
* switch to rustls for arti-client on macos ([#457](https://github.com/hhanh00/zkool2/issues/457)) ([3c9e368](https://github.com/hhanh00/zkool2/commit/3c9e368a212c92eb132343a24bf46f22fd936daf))
* sync missing last chunk of messages ([#670](https://github.com/hhanh00/zkool2/issues/670)) ([8498a34](https://github.com/hhanh00/zkool2/commit/8498a34604b227b87497c56d090474b6a938542b))
* sync sends extra chunk of blocks when reorg/abort ([#951](https://github.com/hhanh00/zkool2/issues/951)) ([4510588](https://github.com/hhanh00/zkool2/commit/45105888d620bb714d26785b7a599edce29582a1))
* synced_height was getting inserted for missing pools ([#664](https://github.com/hhanh00/zkool2/issues/664)) ([06bd0b5](https://github.com/hhanh00/zkool2/commit/06bd0b54e19a849d8807102d89f91fccc2a26c4f))
* taddress at dindex=0 should always be created ([#953](https://github.com/hhanh00/zkool2/issues/953)) ([5936d1a](https://github.com/hhanh00/zkool2/commit/5936d1ac765586a4cc7962d2c16a0250912aa880))
* **test:** mine a few blocks via api instead of waiting ([#854](https://github.com/hhanh00/zkool2/issues/854)) ([5b3785a](https://github.com/hhanh00/zkool2/commit/5b3785a7776c5eff415f3b4fbdfe6b0b2368734b))
* testnet ([fe9f1a4](https://github.com/hhanh00/zkool2/commit/fe9f1a4b5440ba76b072de4c0bd9eb2ec930709b))
* tile overflow ([#819](https://github.com/hhanh00/zkool2/issues/819)) ([4d2fe81](https://github.com/hhanh00/zkool2/commit/4d2fe8140a9382080a555c2fb1faa3b657931361))
* tooltip ([#873](https://github.com/hhanh00/zkool2/issues/873)) ([7130e5e](https://github.com/hhanh00/zkool2/commit/7130e5eaba2578f9fff1327e3f1ec13f2693d281))
* transaction export to csv was missing tx without category ([#656](https://github.com/hhanh00/zkool2/issues/656)) ([e8cac28](https://github.com/hhanh00/zkool2/commit/e8cac28df6468646ced62d858762749d6faec2c4))
* transparent scan on restore account ([#753](https://github.com/hhanh00/zkool2/issues/753)) ([1deb9c7](https://github.com/hhanh00/zkool2/commit/1deb9c7fb9649fb1fed5d4e2487b1dba6ba2fc42))
* transparent sweep ([#663](https://github.com/hhanh00/zkool2/issues/663)) ([865393c](https://github.com/hhanh00/zkool2/commit/865393c23015f44e4ab846012cfe4cd92961443c))
* try to decode as string before bytes ([#567](https://github.com/hhanh00/zkool2/issues/567)) ([89753ff](https://github.com/hhanh00/zkool2/commit/89753ff9a3afb05eb5b78c3cf472bf1de1962711))
* tx details ([#757](https://github.com/hhanh00/zkool2/issues/757)) ([64ff8bc](https://github.com/hhanh00/zkool2/commit/64ff8bc8289f622f5f96e567e01f017377dd235c))
* txid in csv in wrong byte order ([#776](https://github.com/hhanh00/zkool2/issues/776)) ([440949d](https://github.com/hhanh00/zkool2/commit/440949dc685c9c3def56a09ae0331092ecfe6c84))
* typo ([#446](https://github.com/hhanh00/zkool2/issues/446)) ([e39c75c](https://github.com/hhanh00/zkool2/commit/e39c75cc0511d07ac462a6d13c2f17b14d8060a1))
* typo in db version key name ([#525](https://github.com/hhanh00/zkool2/issues/525)) ([230e174](https://github.com/hhanh00/zkool2/commit/230e174d3051adbcdbd12115406ed84e47db576e))
* ua pool selection ([#961](https://github.com/hhanh00/zkool2/issues/961)) ([426e411](https://github.com/hhanh00/zkool2/commit/426e4119eeaa5b7e6a17551ff09a33736eecfff0))
* ui adjustments to chart ([#542](https://github.com/hhanh00/zkool2/issues/542)) ([106b776](https://github.com/hhanh00/zkool2/commit/106b776114a66ca69aba2ef988af1edc9d1994a0))
* UI bugs ([#671](https://github.com/hhanh00/zkool2/issues/671)) ([ff35012](https://github.com/hhanh00/zkool2/commit/ff350129da72ae5f32547763e2007edbce0e21da))
* ui bugs ([#758](https://github.com/hhanh00/zkool2/issues/758)) ([8ac72d2](https://github.com/hhanh00/zkool2/commit/8ac72d213db35dd9cc007cd57f60e621f8ff1d76))
* update dependencies ([#702](https://github.com/hhanh00/zkool2/issues/702)) ([9c64a0d](https://github.com/hhanh00/zkool2/commit/9c64a0d5588e69ad2b8dc139bff9cb21ebac71b9))
* update launcher icon ([#481](https://github.com/hhanh00/zkool2/issues/481)) ([a02eafd](https://github.com/hhanh00/zkool2/commit/a02eafd35b4ff1ccdd1aafe8c81bf2e54380eddb))
* update splash icon ([#483](https://github.com/hhanh00/zkool2/issues/483)) ([5555d2b](https://github.com/hhanh00/zkool2/commit/5555d2b9538f0d4d8d922f58e62d9c18427c0dac))
* update to upstream crates ([#885](https://github.com/hhanh00/zkool2/issues/885)) ([6ff6aff](https://github.com/hhanh00/zkool2/commit/6ff6aff2c4301e5dc2264fb2f49d278e06efd076))
* update zkool for regtest ([f430291](https://github.com/hhanh00/zkool2/commit/f43029189b57b2245e3c3b48dbb848d80e8f9d18))
* upgrade zcvlib ([#948](https://github.com/hhanh00/zkool2/issues/948)) ([a78c22d](https://github.com/hhanh00/zkool2/commit/a78c22d58197bbe84b7de92df315d83c2c20031b))
* use better constant salt ([#946](https://github.com/hhanh00/zkool2/issues/946)) ([ed1c9d2](https://github.com/hhanh00/zkool2/commit/ed1c9d2083170f51df8506f9a42297af962d917a))
* use CARGO_ENCODED_RUSTFLAGS for Android to avoid cargokit override ([3117db6](https://github.com/hhanh00/zkool2/commit/3117db659675d391817986c8b81c750d93f7e25f))
* use CARGO_ENCODED_RUSTFLAGS with \u001f escape for Android ([6dc0d6d](https://github.com/hhanh00/zkool2/commit/6dc0d6df0cb97bfcea515d7713364863b8b6e201))
* use default expiry delta (40) ([#827](https://github.com/hhanh00/zkool2/issues/827)) ([e003579](https://github.com/hhanh00/zkool2/commit/e0035798c91fa25ff1df648bb2bf5cbc3ecee280))
* use get_address_sapling to avoid div_list bug ([#627](https://github.com/hhanh00/zkool2/issues/627)) ([ba664f1](https://github.com/hhanh00/zkool2/commit/ba664f1ce91e22bfb2ad95794755030931fc81db))
* use helper fn to ensure that data is loaded ([#710](https://github.com/hhanh00/zkool2/issues/710)) ([a4e51b5](https://github.com/hhanh00/zkool2/commit/a4e51b57498658862303742f448ed7e3da5b5e56))
* use locale for parsing amounts ([#458](https://github.com/hhanh00/zkool2/issues/458)) ([da950fb](https://github.com/hhanh00/zkool2/commit/da950fba1538eca376bab5a09223de0e3ffb76e4))
* V6/ZSA orchard bundle support across mempool, memo, zebra, and decryptor ([84f0036](https://github.com/hhanh00/zkool2/commit/84f00365535c66fe09307470149873238ad78396))
* v6/ZSA transaction support for transparent sync and shielding ([f287ccc](https://github.com/hhanh00/zkool2/commit/f287cccf091369394fa34a38d6b68c72483c018f))
* **vault:** restore latest logentry ([#922](https://github.com/hhanh00/zkool2/issues/922)) ([f12610b](https://github.com/hhanh00/zkool2/commit/f12610b9fd5166f32529c6c359f6b9e4ced75655))
* **vault:** skip accounts that use a short seed phrase ([#954](https://github.com/hhanh00/zkool2/issues/954)) ([9798e6b](https://github.com/hhanh00/zkool2/commit/9798e6bd72f8bedd6929a47551140bfde7d92a9e))
* voting button "Next" should not be enabled when election associated with another account ([#838](https://github.com/hhanh00/zkool2/issues/838)) ([c5bc3fa](https://github.com/hhanh00/zkool2/commit/c5bc3fa367fae51311c162a06e68e032caf958a7))
* voting ui ([#809](https://github.com/hhanh00/zkool2/issues/809)) ([f41c8ef](https://github.com/hhanh00/zkool2/commit/f41c8efd01a168e779e58a3bfdae3281604feb72))
* windows build ([#496](https://github.com/hhanh00/zkool2/issues/496)) ([1de130f](https://github.com/hhanh00/zkool2/commit/1de130f5c2e9bb5e5226d4ad7f47e6451d88a460))
* witness.rewind and test ([#865](https://github.com/hhanh00/zkool2/issues/865)) ([a47ce91](https://github.com/hhanh00/zkool2/commit/a47ce91e7892d1044ea080c2d9ed7949fa7f0811))
* wrong height chosen for witness data, ([#512](https://github.com/hhanh00/zkool2/issues/512)) ([fe9f1a4](https://github.com/hhanh00/zkool2/commit/fe9f1a4b5440ba76b072de4c0bd9eb2ec930709b))
* zcash-trees panic on empty rho ([7ef50b3](https://github.com/hhanh00/zkool2/commit/7ef50b332d12d7581996dbe3eee9e5af08f351c9))

## [6.15.0](https://github.com/hhanh00/zkool2/compare/zkool-v6.14.6...zkool-v6.15.0) (2026-05-07)


### Features

* support uncompressed transparent private keys (5XXX) ([#973](https://github.com/hhanh00/zkool2/issues/973)) ([f20bb8a](https://github.com/hhanh00/zkool2/commit/f20bb8a0ead97b74b5e2a62b5deeefaba1083706))

## [6.14.6](https://github.com/hhanh00/zkool2/compare/zkool-v6.13.5...zkool-v6.14.6) (2026-05-01)


### Features

* add ed25519 keypair to dkg as round 0 for future message signing ([#900](https://github.com/hhanh00/zkool2/issues/900)) ([80733df](https://github.com/hhanh00/zkool2/commit/80733df1a5f5e88a8db9fdab2db47739765294d3))
* dart vault impl placeholder ([#908](https://github.com/hhanh00/zkool2/issues/908)) ([98a5041](https://github.com/hhanh00/zkool2/commit/98a504188cc6ad48a2873ae777be53a1d406ae75))
* encrypt and save account keys to vault ([#912](https://github.com/hhanh00/zkool2/issues/912)) ([572813e](https://github.com/hhanh00/zkool2/commit/572813e3a462687f46ab4dea1621ce5df90a060e))
* google drive integration ([#907](https://github.com/hhanh00/zkool2/issues/907)) ([d2e3712](https://github.com/hhanh00/zkool2/commit/d2e371202ccd5f8488e14204e45e113eeca75980))
* passkey support for the key vault ([#914](https://github.com/hhanh00/zkool2/issues/914)) ([0bec4cf](https://github.com/hhanh00/zkool2/commit/0bec4cf2a55a4506e862838c51b368a6dce491d4))
* set master vault password api ([#910](https://github.com/hhanh00/zkool2/issues/910)) ([98d6ddf](https://github.com/hhanh00/zkool2/commit/98d6ddf2829afd816b986e8e0f6099117aa8648b))
* sign and verify Frost Messages ([#901](https://github.com/hhanh00/zkool2/issues/901)) ([40dc4e1](https://github.com/hhanh00/zkool2/commit/40dc4e1adaa771ef294a80c6d60e4c713ef9a41a))
* vault impl in dart ([#909](https://github.com/hhanh00/zkool2/issues/909)) ([d36e13e](https://github.com/hhanh00/zkool2/commit/d36e13ebdde74aceb4224baa2391d88219efe9d9))
* vault master key implementation ([#911](https://github.com/hhanh00/zkool2/issues/911)) ([8220e63](https://github.com/hhanh00/zkool2/commit/8220e637f6e7d1c75ba43954ab46746322ab239a))
* vault recovery from master password ([#913](https://github.com/hhanh00/zkool2/issues/913)) ([8f09005](https://github.com/hhanh00/zkool2/commit/8f0900511cac591ac0a7178e0cc6b2d4476cb48e))


### Bug Fixes

* add confirmation/explanation messages ([#915](https://github.com/hhanh00/zkool2/issues/915)) ([def0c69](https://github.com/hhanh00/zkool2/commit/def0c69e20b9100002b20d5b7c9c1fc0a7c56507))
* add expert mode flag and gate the vault behind it ([#957](https://github.com/hhanh00/zkool2/issues/957)) ([2603bcd](https://github.com/hhanh00/zkool2/commit/2603bcdb7d686562918fd856e92fad17fba35fb9))
* add logging messages and fix passkey on android ([#916](https://github.com/hhanh00/zkool2/issues/916)) ([2d28cbc](https://github.com/hhanh00/zkool2/commit/2d28cbc4318f080487eddb0a28e653a7a1049bb7))
* add some extra padding ([#956](https://github.com/hhanh00/zkool2/issues/956)) ([f64014e](https://github.com/hhanh00/zkool2/commit/f64014e36ff18c97489d573a59413629c2ba19b3))
* add timestamp to vault log entry ([#921](https://github.com/hhanh00/zkool2/issues/921)) ([8488cfb](https://github.com/hhanh00/zkool2/commit/8488cfb656888287f1f12343af751c42bc58e6c8))
* add try/catch around rust code ([#963](https://github.com/hhanh00/zkool2/issues/963)) ([ae646b7](https://github.com/hhanh00/zkool2/commit/ae646b73fa3c319aa158cc52fd465f66af245401))
* allow platform + cross-platform by removing authenticatorAttachment ([#941](https://github.com/hhanh00/zkool2/issues/941)) ([c44724c](https://github.com/hhanh00/zkool2/commit/c44724cefb4d3016c41dea9c20524bda89514e68))
* check for missing witnesses and offer to resync ([#891](https://github.com/hhanh00/zkool2/issues/891)) ([d8c7aa5](https://github.com/hhanh00/zkool2/commit/d8c7aa5f9e361091a5bb01d9799a43a172948439))
* check_witness_consistency as debug only ([#968](https://github.com/hhanh00/zkool2/issues/968)) ([5e19e30](https://github.com/hhanh00/zkool2/commit/5e19e30301f107b939e973b841354780b50ec817))
* disable android auto backup ([#932](https://github.com/hhanh00/zkool2/issues/932)) ([4859fd6](https://github.com/hhanh00/zkool2/commit/4859fd6ea0f3bd5eb8f7a7a7c0e07d6020aaebb9))
* disable passkeys on unsupported platforms ([#936](https://github.com/hhanh00/zkool2/issues/936)) ([643dc95](https://github.com/hhanh00/zkool2/commit/643dc95a40ced093f8446ee7a40a3402a087192f))
* do not update vault when disabled ([#947](https://github.com/hhanh00/zkool2/issues/947)) ([d56c7f0](https://github.com/hhanh00/zkool2/commit/d56c7f0c73b6bc2e5ccf424c8ff2b11955814b14))
* eliminate UI refresh "flash" at end of sync ([#958](https://github.com/hhanh00/zkool2/issues/958)) ([20f9bde](https://github.com/hhanh00/zkool2/commit/20f9bded8105e2861ffe4838cebabb0a46371635))
* fetch tx details from account manager ([#964](https://github.com/hhanh00/zkool2/issues/964)) ([89bc7a4](https://github.com/hhanh00/zkool2/commit/89bc7a4769a2b4a9b88e6d76f204c0e3fac3df69))
* iOS build ([#919](https://github.com/hhanh00/zkool2/issues/919)) ([087b755](https://github.com/hhanh00/zkool2/commit/087b755970b3c6e2ea64af5c818af64252703908))
* iOS Google signin to Drive ([#938](https://github.com/hhanh00/zkool2/issues/938)) ([a510c9f](https://github.com/hhanh00/zkool2/commit/a510c9f5b7cb3488c8b1a32782a9f04c82252095))
* lazily sync with the vault ([#937](https://github.com/hhanh00/zkool2/issues/937)) ([293dd53](https://github.com/hhanh00/zkool2/commit/293dd53fb6d13083b2e2a9cec4e959b69e9a2a3a))
* linux nix build ([#899](https://github.com/hhanh00/zkool2/issues/899)) ([ef778f9](https://github.com/hhanh00/zkool2/commit/ef778f95b079b9fa24f00fecaa15a07ed0dfc9eb))
* lots of UI glitches ([#955](https://github.com/hhanh00/zkool2/issues/955)) ([2189fb9](https://github.com/hhanh00/zkool2/commit/2189fb9c5fbfd184d44b95f0b67d3680c6c87d10))
* pin to given flutter version ([#949](https://github.com/hhanh00/zkool2/issues/949)) ([fe63f75](https://github.com/hhanh00/zkool2/commit/fe63f750426e9c84b08d9864a593affd5475adc7))
* removed display, zip212 grace period ([#966](https://github.com/hhanh00/zkool2/issues/966)) ([575aa67](https://github.com/hhanh00/zkool2/commit/575aa67c8ebe6f5012e0b3a8fb82cf3508cab335))
* replace zaino by lightwalletd ([#895](https://github.com/hhanh00/zkool2/issues/895)) ([1b0b67f](https://github.com/hhanh00/zkool2/commit/1b0b67fba7db22f2b7244b50683a70987cbe075a))
* reregister the passkey if it is stale ([#923](https://github.com/hhanh00/zkool2/issues/923)) ([2de92cb](https://github.com/hhanh00/zkool2/commit/2de92cb76cd010a61227c5c4079c6defa2027546))
* return error msg when no prf support ([#944](https://github.com/hhanh00/zkool2/issues/944)) ([5f8216f](https://github.com/hhanh00/zkool2/commit/5f8216f1022fa36de44ce80c08fe9c7ff88364ff))
* sync sends extra chunk of blocks when reorg/abort ([#951](https://github.com/hhanh00/zkool2/issues/951)) ([83a2f1b](https://github.com/hhanh00/zkool2/commit/83a2f1b9470af31a58f09b7090229528bd76fe3a))
* taddress at dindex=0 should always be created ([#953](https://github.com/hhanh00/zkool2/issues/953)) ([5aa35c2](https://github.com/hhanh00/zkool2/commit/5aa35c22f5cbca688f1639fa8b302ff6f66e3a7e))
* ua pool selection ([#961](https://github.com/hhanh00/zkool2/issues/961)) ([e15b98e](https://github.com/hhanh00/zkool2/commit/e15b98ec71b5dc5cfbf5c19af9fb9d919d49a9df))
* upgrade zcvlib ([#948](https://github.com/hhanh00/zkool2/issues/948)) ([0850576](https://github.com/hhanh00/zkool2/commit/0850576d942590fb241551952d7ece17f406c73f))
* use better constant salt ([#946](https://github.com/hhanh00/zkool2/issues/946)) ([008a942](https://github.com/hhanh00/zkool2/commit/008a942066f98e35c23f63100743be45a81a616b))
* **vault:** restore latest logentry ([#922](https://github.com/hhanh00/zkool2/issues/922)) ([59edc06](https://github.com/hhanh00/zkool2/commit/59edc06fa248c16c3606a8c5b06587924b50cd84))
* **vault:** skip accounts that use a short seed phrase ([#954](https://github.com/hhanh00/zkool2/issues/954)) ([b80c4dd](https://github.com/hhanh00/zkool2/commit/b80c4dd712488915b446a992cac5dbf25aa788a3))
* zcash-trees panic on empty rho ([dc3f037](https://github.com/hhanh00/zkool2/commit/dc3f037ff8e5c0c44ca10ca1b99bbf1bd4a7a526))

## [6.13.5](https://github.com/hhanh00/zkool2/compare/zkool-v6.12.0...zkool-v6.13.5) (2026-04-08)


### Features

* add quit election button ([#872](https://github.com/hhanh00/zkool2/issues/872)) ([19db47a](https://github.com/hhanh00/zkool2/commit/19db47a491a75a301512b967923111afe23fb48b))
* add rewind method to witness that brings it to ([#863](https://github.com/hhanh00/zkool2/issues/863)) ([ea50692](https://github.com/hhanh00/zkool2/commit/ea506928036b56d040e0f07ec22707cafe1e6f8b))
* add serializers to commitment tree state ([#868](https://github.com/hhanh00/zkool2/issues/868)) ([054433c](https://github.com/hhanh00/zkool2/commit/054433c8aa0fcd15b55f363a55c03d803b89503a))
* add serializers, size to Edge ([#869](https://github.com/hhanh00/zkool2/issues/869)) ([fb17c08](https://github.com/hhanh00/zkool2/commit/fb17c0833583b74695345b82e60c549c51d3db33))
* add support for computing the auth path of a witness at a prior position ([#860](https://github.com/hhanh00/zkool2/issues/860)) ([b1cb42c](https://github.com/hhanh00/zkool2/commit/b1cb42cc06312af0e9a7e72c108d6d7dc7cdb0e8))
* pir integration ([#871](https://github.com/hhanh00/zkool2/issues/871)) ([4c81679](https://github.com/hhanh00/zkool2/commit/4c81679cda92281aeaa62caaca60b92d581f2d41))


### Bug Fixes

* add diversifier index to new_addresses and return from unconfirmed by account ([#881](https://github.com/hhanh00/zkool2/issues/881)) ([590b7cb](https://github.com/hhanh00/zkool2/commit/590b7cbf5f4db4b003232091b867a01b24df9a91))
* add more info to mempool txs ([#877](https://github.com/hhanh00/zkool2/issues/877)) ([ef03343](https://github.com/hhanh00/zkool2/commit/ef03343c916ac0d1a9ff2c257ceda6d33d0a486a))
* assert in witness calculation ([#870](https://github.com/hhanh00/zkool2/issues/870)) ([97ba893](https://github.com/hhanh00/zkool2/commit/97ba893f61c422c7f8032ab341f6473eab94ae20))
* build break iOS ([#888](https://github.com/hhanh00/zkool2/issues/888)) ([366fc63](https://github.com/hhanh00/zkool2/commit/366fc63095c14bd270888e823522526cf4cc7f0b))
* db escaping in change_db_password ([#883](https://github.com/hhanh00/zkool2/issues/883)) ([ed04c2c](https://github.com/hhanh00/zkool2/commit/ed04c2cc60bc6c8c0deaa68f9f3584c9b7f476ce))
* drop support for 32-bit android due to build breaks ([#858](https://github.com/hhanh00/zkool2/issues/858)) ([3edbd1e](https://github.com/hhanh00/zkool2/commit/3edbd1e6993775520f3d24de07cefd3b00aa1c39))
* fetch election and import atomically ([#875](https://github.com/hhanh00/zkool2/issues/875)) ([c065b04](https://github.com/hhanh00/zkool2/commit/c065b042959eedec1f4d803f853e05e0d22ac7e1))
* fix issue with refresh of input amount widget ([#874](https://github.com/hhanh00/zkool2/issues/874)) ([cf41de8](https://github.com/hhanh00/zkool2/commit/cf41de843252e663f868b1079e773f477f640a7e))
* ledger new account ([#887](https://github.com/hhanh00/zkool2/issues/887)) ([1bde434](https://github.com/hhanh00/zkool2/commit/1bde434f553e8e2eddef9200055e3d5a00436b67))
* tooltip ([#873](https://github.com/hhanh00/zkool2/issues/873)) ([0f8fc75](https://github.com/hhanh00/zkool2/commit/0f8fc75b84e5aa3626ed2ffb41c661b4e0fe93a4))
* update to upstream crates ([#885](https://github.com/hhanh00/zkool2/issues/885)) ([2a9e331](https://github.com/hhanh00/zkool2/commit/2a9e3315107ed5da21eee3f7c7551c0f5f16d0a1))
* witness.rewind and test ([#865](https://github.com/hhanh00/zkool2/issues/865)) ([b5c0c8d](https://github.com/hhanh00/zkool2/commit/b5c0c8d66c866fc0852e62a17aff94a602e98e09))

## [6.12.0](https://github.com/hhanh00/zkool2/compare/zkool-v6.11.4...zkool-v6.12.0) (2026-03-27)


### Features

* add flag "fast" that skips downloading tx details ([#851](https://github.com/hhanh00/zkool2/issues/851)) ([d50c22d](https://github.com/hhanh00/zkool2/commit/d50c22dd636909b4fce0a8887255cda277b28184))
* Fetch tx details in the background ([#853](https://github.com/hhanh00/zkool2/issues/853)) ([517f50b](https://github.com/hhanh00/zkool2/commit/517f50bdbe6f321239436a6d4d3d40c2b00a2bcb))


### Bug Fixes

* make smoke test wait for blocks instead of sleep ([#855](https://github.com/hhanh00/zkool2/issues/855)) ([afe1f80](https://github.com/hhanh00/zkool2/commit/afe1f80d1358867f3ff8d8c9edf152ceb4ee2081))
* **test:** mine a few blocks via api instead of waiting ([#854](https://github.com/hhanh00/zkool2/issues/854)) ([f5f0f9b](https://github.com/hhanh00/zkool2/commit/f5f0f9b3649b54b8e096cda8686685f0fca8fd6a))

## [6.11.4](https://github.com/hhanh00/zkool2/compare/zkool-v6.11.3...zkool-v6.11.4) (2026-03-24)


### Bug Fixes

* escape db password and wrap in single quotes ([#840](https://github.com/hhanh00/zkool2/issues/840)) ([cb7b74d](https://github.com/hhanh00/zkool2/commit/cb7b74d906f5b41db9453e4611086f4c08960b0d))
* voting button "Next" should not be enabled when election associated with another account ([#838](https://github.com/hhanh00/zkool2/issues/838)) ([6be8aaa](https://github.com/hhanh00/zkool2/commit/6be8aaa1d57a8fcd42ad1968868231d7ce423f61))

## [6.11.3](https://github.com/hhanh00/zkool2/compare/zkool-v6.11.2...zkool-v6.11.3) (2026-03-20)


### Bug Fixes

* increase delay after voting from 2s to 15s before refreshing ([#835](https://github.com/hhanh00/zkool2/issues/835)) ([9dcb1e4](https://github.com/hhanh00/zkool2/commit/9dcb1e47003b83d19c7513d9392e9a7389d2fd56))

## [6.11.2](https://github.com/hhanh00/zkool2/compare/zkool-v6.11.1...zkool-v6.11.2) (2026-03-17)


### Bug Fixes

* DEFAULT_TX_EXPIRY_DELTA is added by the pczt builder ([#829](https://github.com/hhanh00/zkool2/issues/829)) ([8a5011c](https://github.com/hhanh00/zkool2/commit/8a5011caca7349c877b9e59aa055660f42ed1ce1))
* do not show sync snackbar when app is in background ([#832](https://github.com/hhanh00/zkool2/issues/832)) ([56355e5](https://github.com/hhanh00/zkool2/commit/56355e5328bf8416c19c31b0ab90eff518558878))
* remove redundant tooltip reset message ([#830](https://github.com/hhanh00/zkool2/issues/830)) ([a6a1bf4](https://github.com/hhanh00/zkool2/commit/a6a1bf4d46092cc5b85395a196c9bb0dc8245cf7))
* use default expiry delta (40) ([#827](https://github.com/hhanh00/zkool2/issues/827)) ([393bf2d](https://github.com/hhanh00/zkool2/commit/393bf2d52a42f75f2ed90f1b5a82cf5cb4ebc62e))

## [6.11.1](https://github.com/hhanh00/zkool2/compare/zkool-v6.11.0...zkool-v6.11.1) (2026-03-09)


### Bug Fixes

* change logic of the next button in vote & add tooltips ([#824](https://github.com/hhanh00/zkool2/issues/824)) ([b931115](https://github.com/hhanh00/zkool2/commit/b931115c26b93816df21a7900b4852832a084653))

## [6.11.0](https://github.com/hhanh00/zkool2/compare/zkool-v6.10.2...zkool-v6.11.0) (2026-03-09)


### Features

* add auto fx rate update flag ([#816](https://github.com/hhanh00/zkool2/issues/816)) ([9ba11de](https://github.com/hhanh00/zkool2/commit/9ba11de52e1efb54051ae34674c3d58e4df8637e))
* add progress bar during scanning ([#801](https://github.com/hhanh00/zkool2/issues/801)) ([9412f0e](https://github.com/hhanh00/zkool2/commit/9412f0e03153131d8f5fbc818c5ea7150ef28d1d))
* button for deleting election data ([#806](https://github.com/hhanh00/zkool2/issues/806)) ([db9db91](https://github.com/hhanh00/zkool2/commit/db9db918503f55019ee063d059b9d221cd615a6c))
* coin voting functionality ([#808](https://github.com/hhanh00/zkool2/issues/808)) ([acb36a9](https://github.com/hhanh00/zkool2/commit/acb36a9eb8091553035d09034deb6190d8bae7bd))
* fetch election from vote server ([#798](https://github.com/hhanh00/zkool2/issues/798)) ([f5761ad](https://github.com/hhanh00/zkool2/commit/f5761adc85935b96cb571f01a7370ae5791cd556))
* fetch election from vote server ([#799](https://github.com/hhanh00/zkool2/issues/799)) ([89957c4](https://github.com/hhanh00/zkool2/commit/89957c461a55af441e9e2b20ce3f556272305450))
* **graphql:** jwt authorization ([#794](https://github.com/hhanh00/zkool2/issues/794)) ([9f1b146](https://github.com/hhanh00/zkool2/commit/9f1b1463c2468e0a0503c00169a57aaae47fb4d5))
* **graphql:** read/write scope ([#822](https://github.com/hhanh00/zkool2/issues/822)) ([84e9b8d](https://github.com/hhanh00/zkool2/commit/84e9b8d2655135ceb5c3dcf503952fac18c79ed7))
* scan existing notes to compute voting power ([#800](https://github.com/hhanh00/zkool2/issues/800)) ([d26d6c5](https://github.com/hhanh00/zkool2/commit/d26d6c56e011e525f9d2da73a46f0bb5753755d1))
* submit ballot ([#804](https://github.com/hhanh00/zkool2/issues/804)) ([d3252a1](https://github.com/hhanh00/zkool2/commit/d3252a1237a84e0e785bf14c4644c04f5bc64e38))
* synchronize with voting chain ([#805](https://github.com/hhanh00/zkool2/issues/805)) ([44b6b84](https://github.com/hhanh00/zkool2/commit/44b6b84f9a2aa7db72d9d16cb3b51eaccbad2461))
* tx account update ([#812](https://github.com/hhanh00/zkool2/issues/812)) ([fc87ba2](https://github.com/hhanh00/zkool2/commit/fc87ba2034a9654bd21d8dd9650aed6fecf7bb2f))
* vote delegation ([#810](https://github.com/hhanh00/zkool2/issues/810)) ([8c40248](https://github.com/hhanh00/zkool2/commit/8c4024824d789b942e6448eb69429ea313577837))
* voting form ([#803](https://github.com/hhanh00/zkool2/issues/803)) ([0624ecf](https://github.com/hhanh00/zkool2/commit/0624ecfd3df286dd3785bbacfe9c74e7dc430053))


### Bug Fixes

* account list ui update ([#815](https://github.com/hhanh00/zkool2/issues/815)) ([e4f2fb9](https://github.com/hhanh00/zkool2/commit/e4f2fb93e4de7695630da8fdc324a9cb76a1be32))
* add error handling ([#817](https://github.com/hhanh00/zkool2/issues/817)) ([403d859](https://github.com/hhanh00/zkool2/commit/403d859d4acfc2b26ef83d9c742383939aa3ec1b))
* allow admin user to execute any command ([#796](https://github.com/hhanh00/zkool2/issues/796)) ([31742a7](https://github.com/hhanh00/zkool2/commit/31742a71db2549c26919feb596b54d908298003f))
* check that current account is associated with the vote ([#811](https://github.com/hhanh00/zkool2/issues/811)) ([b59989e](https://github.com/hhanh00/zkool2/commit/b59989e16da87c3b459ba213714bcc1142b165c5))
* put the progress bar in a modal dialog box ([#802](https://github.com/hhanh00/zkool2/issues/802)) ([a4171ad](https://github.com/hhanh00/zkool2/commit/a4171ad2a133f55cbed2dc88f54d208e44425444))
* remove dummy text ([381a0f7](https://github.com/hhanh00/zkool2/commit/381a0f73f0c3efae3507cb2af0023414641367db))
* save current account between restarts ([#814](https://github.com/hhanh00/zkool2/issues/814)) ([c67126a](https://github.com/hhanh00/zkool2/commit/c67126a00a416fa77f6c3c800fc73cdaa190dad8))
* tile overflow ([#819](https://github.com/hhanh00/zkool2/issues/819)) ([5de3abe](https://github.com/hhanh00/zkool2/commit/5de3abe9ea1b5ffce1ccb6943e6760096444dc3b))
* voting ui ([#809](https://github.com/hhanh00/zkool2/issues/809)) ([5195c06](https://github.com/hhanh00/zkool2/commit/5195c0637aa551a4b9139cb0e384a77d251c0af4))

## [6.10.2](https://github.com/hhanh00/zkool2/compare/zkool-v6.10.1...zkool-v6.10.2) (2026-02-18)


### Bug Fixes

* alpine base image for docker ([#789](https://github.com/hhanh00/zkool2/issues/789)) ([5ac91bb](https://github.com/hhanh00/zkool2/commit/5ac91bba24cafb0b9fcba6ee03cf1a79c3b6c533))

## [6.10.1](https://github.com/hhanh00/zkool2/compare/zkool-v6.10.0...zkool-v6.10.1) (2026-02-17)


### Bug Fixes

* android build break ([#770](https://github.com/hhanh00/zkool2/issues/770)) ([8d85dbb](https://github.com/hhanh00/zkool2/commit/8d85dbb6117076c0e128bbabd131f9411a011a62))
* docker build ([#779](https://github.com/hhanh00/zkool2/issues/779)) ([5e80485](https://github.com/hhanh00/zkool2/commit/5e8048561b2b52b9954a029d5dead50df65add1d))
* increase build number ([#788](https://github.com/hhanh00/zkool2/issues/788)) ([9442814](https://github.com/hhanh00/zkool2/commit/9442814144fab6637045e5357312b0ea64098ad4))
* make docker image multiplatform ([#785](https://github.com/hhanh00/zkool2/issues/785)) ([aa8f467](https://github.com/hhanh00/zkool2/commit/aa8f467f1510c5a7d88d1505f600f1a8803c8380))
* txid in csv in wrong byte order ([#776](https://github.com/hhanh00/zkool2/issues/776)) ([60b057b](https://github.com/hhanh00/zkool2/commit/60b057b07a166e37b5fd38b0826e7ab014bfeb70))

## [6.10.0](https://github.com/hhanh00/zkool2/compare/zkool-v6.9.0...zkool-v6.10.0) (2026-02-14)


### Features

* add Flatpak support ([#766](https://github.com/hhanh00/zkool2/issues/766)) ([a160ce4](https://github.com/hhanh00/zkool2/commit/a160ce4107332a35747c9debd67425cb8789c7e6))


### Bug Fixes

* add message when wallet is offline ([#763](https://github.com/hhanh00/zkool2/issues/763)) ([5ad358a](https://github.com/hhanh00/zkool2/commit/5ad358a0ccd419a01ce7334e54a454070a9d3b17))
* bind to anyip ([#767](https://github.com/hhanh00/zkool2/issues/767)) ([634140d](https://github.com/hhanh00/zkool2/commit/634140d3193635a2c3029562234694bf212c660e))
* fix address qr scan ([#760](https://github.com/hhanh00/zkool2/issues/760)) ([a28617f](https://github.com/hhanh00/zkool2/commit/a28617fd2c400ae38ebf5bfad0ee0e7fa1a9946f))
* sending to tex address ([#765](https://github.com/hhanh00/zkool2/issues/765)) ([98eea82](https://github.com/hhanh00/zkool2/commit/98eea82efdec7e64e35d4ebc3bd64af49e4128e4))

## [6.9.0](https://github.com/hhanh00/zkool2/compare/zkool-v6.8.2...zkool-v6.9.0) (2026-01-17)


### Features

* "synchronize" returns current height ([#731](https://github.com/hhanh00/zkool2/issues/731)) ([048e851](https://github.com/hhanh00/zkool2/commit/048e8519df6f7520c064b95799a2dd8afeb9c546))
* add edge from note to tx ([#733](https://github.com/hhanh00/zkool2/issues/733)) ([1f241ef](https://github.com/hhanh00/zkool2/commit/1f241efe42378438fd8db05954444da98e4e6993))
* add polling interval to coin config ([#727](https://github.com/hhanh00/zkool2/issues/727)) ([2c9c8e6](https://github.com/hhanh00/zkool2/commit/2c9c8e6b441532299edc7acfba26affef1d32c53))
* add total balance to get_balance ([#734](https://github.com/hhanh00/zkool2/issues/734)) ([f79220c](https://github.com/hhanh00/zkool2/commit/f79220c282c9bda51a95209f01e44b468b63fe27))
* cli config settings ([#737](https://github.com/hhanh00/zkool2/issues/737)) ([d55386e](https://github.com/hhanh00/zkool2/commit/d55386ed664aba14827d08f93d20ae220ea14c21))
* graphql query account main data ([#711](https://github.com/hhanh00/zkool2/issues/711)) ([0ea90e8](https://github.com/hhanh00/zkool2/commit/0ea90e8e2382a74401aaa2fedb0100c3266c4600))
* **graphql:** account_by_id, transaction_by_id, and connections ([#724](https://github.com/hhanh00/zkool2/issues/724)) ([e9d2a77](https://github.com/hhanh00/zkool2/commit/e9d2a77600aee480b857ab393ed7b3d9e77edc6d))
* **graphql:** add height & balance to account data ([#749](https://github.com/hhanh00/zkool2/issues/749)) ([c47eb69](https://github.com/hhanh00/zkool2/commit/c47eb69e9f12a1a33be6f116492edda0e3d19b1f))
* **graphql:** add outputs, memos, spends to tx details ([#750](https://github.com/hhanh00/zkool2/issues/750)) ([924beb5](https://github.com/hhanh00/zkool2/commit/924beb59b456fc1b4daaba5e05814c505ec10b74))
* **graphql:** add scope, diversifier and address to notes ([#738](https://github.com/hhanh00/zkool2/issues/738)) ([b65d9e4](https://github.com/hhanh00/zkool2/commit/b65d9e418d62345ad0f3b8da8516dea8bf3104a3))
* **graphql:** balance of account ([#715](https://github.com/hhanh00/zkool2/issues/715)) ([cd6ec77](https://github.com/hhanh00/zkool2/commit/cd6ec773f2f4e35a18d8348914acd47176c97741))
* **graphql:** CI ([#713](https://github.com/hhanh00/zkool2/issues/713)) ([9468580](https://github.com/hhanh00/zkool2/commit/9468580cfcdc1a40f04639dd8a0514ecb5ba6155))
* **graphql:** cold wallet ([#746](https://github.com/hhanh00/zkool2/issues/746)) ([cd4d242](https://github.com/hhanh00/zkool2/commit/cd4d242cdba5022053067c293d1e25f24c6042b7))
* **graphql:** create_account ([#716](https://github.com/hhanh00/zkool2/issues/716)) ([11785de](https://github.com/hhanh00/zkool2/commit/11785de7ecc4b273a52d3ddab69339392f8e0fb0))
* **graphql:** dkg (no automation) ([#740](https://github.com/hhanh00/zkool2/issues/740)) ([1e9d831](https://github.com/hhanh00/zkool2/commit/1e9d83120ad210110fe27d03b886b6b4f69dd383))
* **graphql:** dkg automation ([#741](https://github.com/hhanh00/zkool2/issues/741)) ([ca977f4](https://github.com/hhanh00/zkool2/commit/ca977f44949564180c652a4705c1607b9b277da4))
* **graphql:** edit/delete account, current_height ([#717](https://github.com/hhanh00/zkool2/issues/717)) ([2ae3514](https://github.com/hhanh00/zkool2/commit/2ae3514408ed79aa9b585ff40805375353a51388))
* **graphql:** frost signature ([#744](https://github.com/hhanh00/zkool2/issues/744)) ([ee637a3](https://github.com/hhanh00/zkool2/commit/ee637a378c513ccbe76cb2b7ec810fa749b0d152))
* **graphql:** frost signing automation ([#745](https://github.com/hhanh00/zkool2/issues/745)) ([965ebea](https://github.com/hhanh00/zkool2/commit/965ebeaf27859938a851e356d8153eda1921a241))
* **graphql:** get_addresses ([#719](https://github.com/hhanh00/zkool2/issues/719)) ([3261996](https://github.com/hhanh00/zkool2/commit/3261996352bc89d45debdea47219c1e2b9ec788b))
* **graphql:** list notes ([#721](https://github.com/hhanh00/zkool2/issues/721)) ([4fe48f9](https://github.com/hhanh00/zkool2/commit/4fe48f9b1e7c4f8ac96f13856f55b349cb000936))
* **graphql:** memos_by_transaction ([#714](https://github.com/hhanh00/zkool2/issues/714)) ([3c4bcaa](https://github.com/hhanh00/zkool2/commit/3c4bcaa01c2f4cdce791d7754411c64a3b16d535))
* **graphql:** mempool monitoring, unconfirmed txs ([#723](https://github.com/hhanh00/zkool2/issues/723)) ([9157906](https://github.com/hhanh00/zkool2/commit/91579064e6a8a71def507163faa063df3470847b))
* **graphql:** new addresses, get balance at height ([#722](https://github.com/hhanh00/zkool2/issues/722)) ([e92f29f](https://github.com/hhanh00/zkool2/commit/e92f29f10a4cefe32085ba524908714810170eb9))
* **graphql:** pczt decode in human readble form ([#743](https://github.com/hhanh00/zkool2/issues/743)) ([754465a](https://github.com/hhanh00/zkool2/commit/754465aabacb33b12d14ce3b91edbbc76d313625))
* **graphql:** prepare unsigned tx ([#742](https://github.com/hhanh00/zkool2/issues/742)) ([1f2393c](https://github.com/hhanh00/zkool2/commit/1f2393cceb6098e26114bb8fb92a9063e9536431))
* **graphql:** send funds ([#720](https://github.com/hhanh00/zkool2/issues/720)) ([1ffc3a9](https://github.com/hhanh00/zkool2/commit/1ffc3a9a6bad6f3ae48575f825a0208bae979fbf))
* **graphql:** synchronize ([#718](https://github.com/hhanh00/zkool2/issues/718)) ([9f97237](https://github.com/hhanh00/zkool2/commit/9f9723750a60197fe95994aed078ca11930a9d89))
* **graphql:** transactions_by_account ([#712](https://github.com/hhanh00/zkool2/issues/712)) ([fed31e1](https://github.com/hhanh00/zkool2/commit/fed31e19fe04ea566b380d1a9f96a5d8908b7e3f))
* remove rocket, add warp ([#726](https://github.com/hhanh00/zkool2/issues/726)) ([f15c90b](https://github.com/hhanh00/zkool2/commit/f15c90b01405d5738765bd3ccf5ee293bc31e24f))
* subscription channels for block/tx events ([#729](https://github.com/hhanh00/zkool2/issues/729)) ([1607b98](https://github.com/hhanh00/zkool2/commit/1607b98b0824c0cd6b5266b6e095dd56e43afdfd))
* subscription to tx and new blocks ([#730](https://github.com/hhanh00/zkool2/issues/730)) ([265b4f6](https://github.com/hhanh00/zkool2/commit/265b4f6d94d2187bcf707770aaf00b91ab5ebf85))
* use the best source pool for the change ([#708](https://github.com/hhanh00/zkool2/issues/708)) ([af130e1](https://github.com/hhanh00/zkool2/commit/af130e10a5bd7425a5a336d9600de9ab52fb48a9))
* use warp as the web server ([#728](https://github.com/hhanh00/zkool2/issues/728)) ([0e14ceb](https://github.com/hhanh00/zkool2/commit/0e14ceb8d47de09e4b50001c3d372acee288b1ae))


### Bug Fixes

* build warnings ([#736](https://github.com/hhanh00/zkool2/issues/736)) ([e37f9a5](https://github.com/hhanh00/zkool2/commit/e37f9a5fbc343f1e8d2e10a466476e305ee73385))
* coingecko api key required now ([#754](https://github.com/hhanh00/zkool2/issues/754)) ([bbd6c79](https://github.com/hhanh00/zkool2/commit/bbd6c7904abcdf8280edd840a940b97fa2baf1d8))
* disclaimer page showing up twice ([#752](https://github.com/hhanh00/zkool2/issues/752)) ([c6c5d89](https://github.com/hhanh00/zkool2/commit/c6c5d89d2ba122e895800cbf7358d5805767131c))
* get_notes by txid ([#747](https://github.com/hhanh00/zkool2/issues/747)) ([7d0fbaa](https://github.com/hhanh00/zkool2/commit/7d0fbaa5e482e1083806ce391a1f8700d9142da5))
* pinlock ([#748](https://github.com/hhanh00/zkool2/issues/748)) ([f1a41b2](https://github.com/hhanh00/zkool2/commit/f1a41b2f515053f7148abd1d1dc80b0f2f4f0dea))
* remove dust change policy ([#755](https://github.com/hhanh00/zkool2/issues/755)) ([5d53a3f](https://github.com/hhanh00/zkool2/commit/5d53a3f5a2d8f5b0619248384d7431f0a185e171))
* remove polling_interval from config ([#732](https://github.com/hhanh00/zkool2/issues/732)) ([e630604](https://github.com/hhanh00/zkool2/commit/e630604e20f7388abef00fa25d931f00fa312deb))
* remove transparent receiver from default ua ([#735](https://github.com/hhanh00/zkool2/issues/735)) ([8efa557](https://github.com/hhanh00/zkool2/commit/8efa55733583db8bbb79f0dcc91e5980e8c082ea))
* save send form state between pinlocks ([#756](https://github.com/hhanh00/zkool2/issues/756)) ([559a15c](https://github.com/hhanh00/zkool2/commit/559a15c015bc9c7cebd14ddde81d82bf5b43f93c))
* transparent scan on restore account ([#753](https://github.com/hhanh00/zkool2/issues/753)) ([f81ea3f](https://github.com/hhanh00/zkool2/commit/f81ea3f0881c63ab88054c183ca8bb2b4b2f9d9d))
* tx details ([#757](https://github.com/hhanh00/zkool2/issues/757)) ([842b317](https://github.com/hhanh00/zkool2/commit/842b317c65d521c15c8a0199026488417a71eb53))
* ui bugs ([#758](https://github.com/hhanh00/zkool2/issues/758)) ([6392c24](https://github.com/hhanh00/zkool2/commit/6392c244eea4b216137af54221cdd1e1263d91f7))
* use helper fn to ensure that data is loaded ([#710](https://github.com/hhanh00/zkool2/issues/710)) ([1caa053](https://github.com/hhanh00/zkool2/commit/1caa053447adea9ba3e06072afa020059458256c))

## [6.8.2](https://github.com/hhanh00/zkool2/compare/zkool-v6.8.1...zkool-v6.8.2) (2025-12-29)


### Bug Fixes

* new account from ledger ([#704](https://github.com/hhanh00/zkool2/issues/704)) ([4529d7f](https://github.com/hhanh00/zkool2/commit/4529d7f8699e656f93d7378e5dca69e7a808e37b))
* remove Ledger NU6.1 warning since the app was updated ([#707](https://github.com/hhanh00/zkool2/issues/707)) ([f78135b](https://github.com/hhanh00/zkool2/commit/f78135bd7915236678a6a0c088723b2de468cd72))
* spending sapling internal notes ([#706](https://github.com/hhanh00/zkool2/issues/706)) ([30f7eb6](https://github.com/hhanh00/zkool2/commit/30f7eb63e8856eedd011fe0552ad1cdc0cbe42de))

## [6.8.1](https://github.com/hhanh00/zkool2/compare/zkool-v6.8.0...zkool-v6.8.1) (2025-12-20)


### Bug Fixes

* update dependencies ([#702](https://github.com/hhanh00/zkool2/issues/702)) ([8e0788b](https://github.com/hhanh00/zkool2/commit/8e0788bacf8bf8d80575d48b3ee594f28bdec4f6))

## [6.8.0](https://github.com/hhanh00/zkool2/compare/zkool-v6.7.0...zkool-v6.8.0) (2025-12-03)


### Features

* derive Ledger sapling from seed ([#696](https://github.com/hhanh00/zkool2/issues/696)) ([14a72f7](https://github.com/hhanh00/zkool2/commit/14a72f7c8b67137ba8afae0aa4248183a3a59936))


### Bug Fixes

* http over tor ([#688](https://github.com/hhanh00/zkool2/issues/688)) ([cec2d29](https://github.com/hhanh00/zkool2/commit/cec2d29cdc2c5c2759dde65e9137cc654cb954d4))
* show warning when using ledger because of NU6.1 ([#691](https://github.com/hhanh00/zkool2/issues/691)) ([8b27cf3](https://github.com/hhanh00/zkool2/commit/8b27cf38911ddfaed0c0994885c422b5396f895f))

## [6.7.1](https://github.com/hhanh00/zkool2/compare/zkool-v6.7.0...zkool-v6.7.1) (2025-12-02)


### Bug Fixes

* http over tor ([#688](https://github.com/hhanh00/zkool2/issues/688)) ([cec2d29](https://github.com/hhanh00/zkool2/commit/cec2d29cdc2c5c2759dde65e9137cc654cb954d4))
* show warning when using ledger because of NU6.1 ([#691](https://github.com/hhanh00/zkool2/issues/691)) ([8b27cf3](https://github.com/hhanh00/zkool2/commit/8b27cf38911ddfaed0c0994885c422b5396f895f))

## [6.7.0](https://github.com/hhanh00/zkool2/compare/zkool-v6.6.0...zkool-v6.7.0) (2025-11-28)


### Features

* QR code transmission ([#680](https://github.com/hhanh00/zkool2/issues/680)) ([62a041d](https://github.com/hhanh00/zkool2/commit/62a041dac700636fecf1bb9f38bae2d8ababdcae))


### Bug Fixes

* ledger build ([#686](https://github.com/hhanh00/zkool2/issues/686)) ([6219c0f](https://github.com/hhanh00/zkool2/commit/6219c0fa5bf495c47cea62ccb5f2c972181fb21d))
* missing update of lwd url ([#684](https://github.com/hhanh00/zkool2/issues/684)) ([70de7d0](https://github.com/hhanh00/zkool2/commit/70de7d01ad49fec5f545381648f05212ff36ac6b))
* pass coin as parameter ([#682](https://github.com/hhanh00/zkool2/issues/682)) ([5e4d291](https://github.com/hhanh00/zkool2/commit/5e4d2910ebdd953cfef30a10212e0f2d9207bd3d))
* support for NU6.1 ([#683](https://github.com/hhanh00/zkool2/issues/683)) ([0560901](https://github.com/hhanh00/zkool2/commit/0560901ca7df6655b636ccc7ad0db12995e4e49d))

## [6.6.0](https://github.com/hhanh00/zkool2/compare/zkool-v6.5.2...zkool-v6.6.0) (2025-11-19)


### Features

* allow ledger account without sapling address ([#677](https://github.com/hhanh00/zkool2/issues/677)) ([49f0fab](https://github.com/hhanh00/zkool2/commit/49f0fabb061a7208b01d6620dbe972d9ec63d3a1))


### Bug Fixes

* account navigation ([#658](https://github.com/hhanh00/zkool2/issues/658)) ([5a5ee28](https://github.com/hhanh00/zkool2/commit/5a5ee28b76a50b4816ae1daf4beca83cb7d96d53))
* account tx history not refreshing after sync ([#675](https://github.com/hhanh00/zkool2/issues/675)) ([ce146cf](https://github.com/hhanh00/zkool2/commit/ce146cf59418993d042a7e6dba9e95823e2e383b))
* add ledger-recovery tool ([#676](https://github.com/hhanh00/zkool2/issues/676)) ([e551b92](https://github.com/hhanh00/zkool2/commit/e551b92d2655e84ee4733c1304ee96a3df52358a))
* aindex not saved for ledger accounts ([49f0fab](https://github.com/hhanh00/zkool2/commit/49f0fabb061a7208b01d6620dbe972d9ec63d3a1))
* app resize ([#661](https://github.com/hhanh00/zkool2/issues/661)) ([de91895](https://github.com/hhanh00/zkool2/commit/de918954d3997f088ab466e7d30044262e21381e))
* app state notification system mobx -&gt; riverpod ([#657](https://github.com/hhanh00/zkool2/issues/657)) ([b060512](https://github.com/hhanh00/zkool2/commit/b060512c0b55389027827d792e5abceccfa57a41))
* appsettings ([#660](https://github.com/hhanh00/zkool2/issues/660)) ([6e59339](https://github.com/hhanh00/zkool2/commit/6e59339ab831a6150f0331c64b444bfd0d8081c3))
* autosync & mempool ([#659](https://github.com/hhanh00/zkool2/issues/659)) ([ee7f63d](https://github.com/hhanh00/zkool2/commit/ee7f63d05546f8e6c457de079ea0f89304e6070e))
* change of lwd ([#662](https://github.com/hhanh00/zkool2/issues/662)) ([e53c4b0](https://github.com/hhanh00/zkool2/commit/e53c4b0e6234d780dd389a5b360189bba5d06a8e))
* don't require pin if biometrics not available ([#679](https://github.com/hhanh00/zkool2/issues/679)) ([203debc](https://github.com/hhanh00/zkool2/commit/203debc94fa40805044a71de393f50ee07c40f26))
* lock pin ([#665](https://github.com/hhanh00/zkool2/issues/665)) ([157ab79](https://github.com/hhanh00/zkool2/commit/157ab792f7340372557bf55892a0c6175bb9dc24))
* pinlock always needed even when disabled in settings ([#678](https://github.com/hhanh00/zkool2/issues/678)) ([5fd5b35](https://github.com/hhanh00/zkool2/commit/5fd5b358346cb15b4a2a4e20e0c6f7a73465e82a))
* pinlock on rest of the pages ([#666](https://github.com/hhanh00/zkool2/issues/666)) ([65a5c4f](https://github.com/hhanh00/zkool2/commit/65a5c4f878ff764a712ab361e2e62c0f7012956a))
* remove dependency on connectivity_plus and use config setting ([#654](https://github.com/hhanh00/zkool2/issues/654)) ([763de59](https://github.com/hhanh00/zkool2/commit/763de59fb3f4516d24e71f03ff38084b0d28cb4d))
* remove some ui glitch ([#668](https://github.com/hhanh00/zkool2/issues/668)) ([267a5e1](https://github.com/hhanh00/zkool2/commit/267a5e19fe69a2bdedc1a84787ef0cc6eb1aadfc))
* small ui bug ([#672](https://github.com/hhanh00/zkool2/issues/672)) ([6f3e080](https://github.com/hhanh00/zkool2/commit/6f3e080963d13bbccde2d1ed0f6179982f20d8fc))
* sync missing last chunk of messages ([#670](https://github.com/hhanh00/zkool2/issues/670)) ([a6588c1](https://github.com/hhanh00/zkool2/commit/a6588c185a2528a0edb537f367fe3c9fd8163352))
* synced_height was getting inserted for missing pools ([#664](https://github.com/hhanh00/zkool2/issues/664)) ([82fe69e](https://github.com/hhanh00/zkool2/commit/82fe69e1145358972cde4b6393a2b8ca96155ea3))
* transaction export to csv was missing tx without category ([#656](https://github.com/hhanh00/zkool2/issues/656)) ([634bb7e](https://github.com/hhanh00/zkool2/commit/634bb7ea6fc85822867bf0a0e184febbcaba5ba1))
* transparent sweep ([#663](https://github.com/hhanh00/zkool2/issues/663)) ([4d3e224](https://github.com/hhanh00/zkool2/commit/4d3e22469db48b909aa3e1f25ace90979f9fe3cf))
* UI bugs ([#671](https://github.com/hhanh00/zkool2/issues/671)) ([4ffcde0](https://github.com/hhanh00/zkool2/commit/4ffcde0014d907c8774b728ebbd2fc5e1d016205))

## [6.5.2](https://github.com/hhanh00/zkool2/compare/zkool-v6.5.1...zkool-v6.5.2) (2025-10-23)


### Bug Fixes

* add anchor corruption detection ([#648](https://github.com/hhanh00/zkool2/issues/648)) ([66ecf43](https://github.com/hhanh00/zkool2/commit/66ecf43f1022b781d89f4e4bd9a0d2dd2b3504f0))
* add db check ([#644](https://github.com/hhanh00/zkool2/issues/644)) ([2cf4775](https://github.com/hhanh00/zkool2/commit/2cf4775883d825f45ea537f3e8ac28e9479a9607))
* add debugging messages ([#647](https://github.com/hhanh00/zkool2/issues/647)) ([213a8b8](https://github.com/hhanh00/zkool2/commit/213a8b855d43157136568e0b77896834faf9bb61))
* remove out of band abort messages that could mess with the commit ([#646](https://github.com/hhanh00/zkool2/issues/646)) ([a19529f](https://github.com/hhanh00/zkool2/commit/a19529fb58e33b2efa411735c39dc5b6a5a4b925))

## [6.5.1](https://github.com/hhanh00/zkool2/compare/zkool-v6.5.0...zkool-v6.5.1) (2025-10-17)


### Bug Fixes

* build script for iso ([#636](https://github.com/hhanh00/zkool2/issues/636)) ([bd6e3b3](https://github.com/hhanh00/zkool2/commit/bd6e3b32c08b24cddd24c3bf88b0603731b079b5))
* support ledger memos ([#632](https://github.com/hhanh00/zkool2/issues/632)) ([c5677dd](https://github.com/hhanh00/zkool2/commit/c5677dd3938b92f18f97e3de0d6e557ed9ead708))

## [6.5.0](https://github.com/hhanh00/zkool2/compare/zkool-v6.4.0...zkool-v6.5.0) (2025-10-14)


### Features

* accept transparent public keys ([#592](https://github.com/hhanh00/zkool2/issues/592)) ([7bec12c](https://github.com/hhanh00/zkool2/commit/7bec12c345d37ee3dedde2b898805743bb294c81))
* generate diversified addresses for the Ledger ([f269ed4](https://github.com/hhanh00/zkool2/commit/f269ed4154c5b34be58aa59e5fb9ef6d2e03fdd9))
* import ledger accounts ([#594](https://github.com/hhanh00/zkool2/issues/594)) ([bade75c](https://github.com/hhanh00/zkool2/commit/bade75cff829cdaeb14eebfa1c8771be003df2d1))
* **ledger:** error handling ([#609](https://github.com/hhanh00/zkool2/issues/609)) ([c557065](https://github.com/hhanh00/zkool2/commit/c55706525b13c5e7e55e8790f65ba279c047b6c6))
* **ledger:** error when tx has too many I/O ([#611](https://github.com/hhanh00/zkool2/issues/611)) ([2d69baf](https://github.com/hhanh00/zkool2/commit/2d69baf66336626c9609281368a178ba86fdaf59))
* **ledger:** Ledger integration ([#591](https://github.com/hhanh00/zkool2/issues/591)) ([dbd2a65](https://github.com/hhanh00/zkool2/commit/dbd2a65544d66de15e6ea5d2f680b28b1bdca49d))
* **ledger:** move zemu under feature flag ([#610](https://github.com/hhanh00/zkool2/issues/610)) ([ecf07e9](https://github.com/hhanh00/zkool2/commit/ecf07e91c3309be39ac394c3ec5adaac007c2329))
* **ledger:** save/restore hw account ([#616](https://github.com/hhanh00/zkool2/issues/616)) ([db18458](https://github.com/hhanh00/zkool2/commit/db18458a828ee3cb284c93fdb9eb718edfda9eca))
* **ledger:** scan transparent addresses ([#607](https://github.com/hhanh00/zkool2/issues/607)) ([63d9331](https://github.com/hhanh00/zkool2/commit/63d93313582b6296807dc85f04ddd31edfac628a))
* **ledger:** show t/z address on device for verification ([#622](https://github.com/hhanh00/zkool2/issues/622)) ([8a8260c](https://github.com/hhanh00/zkool2/commit/8a8260c8e95bc7fecb509d2641c37098a203108f))
* **ledger:** support for t2t and t2z ([#608](https://github.com/hhanh00/zkool2/issues/608)) ([08013d5](https://github.com/hhanh00/zkool2/commit/08013d5812b76eac68376754efe26ab91c24d26f))
* **ledger:** support transparent addresses ([#606](https://github.com/hhanh00/zkool2/issues/606)) ([9c81e54](https://github.com/hhanh00/zkool2/commit/9c81e54480affad19c6d85b9f1472f3fd031c2ae))
* send tx with Ledger ([#595](https://github.com/hhanh00/zkool2/issues/595)) ([69546be](https://github.com/hhanh00/zkool2/commit/69546be43b8ccfd79399f56c77ee6a657ed28b41))


### Bug Fixes

* build break on CI macos ([#600](https://github.com/hhanh00/zkool2/issues/600)) ([34c44cd](https://github.com/hhanh00/zkool2/commit/34c44cd9d40e58eec3103165bf1cea7889501059))
* build break on mobile ([#598](https://github.com/hhanh00/zkool2/issues/598)) ([76dcef4](https://github.com/hhanh00/zkool2/commit/76dcef4b20a14ebc6e9a9c9f51068a41bac21670))
* build break on mobile ([#624](https://github.com/hhanh00/zkool2/issues/624)) ([4e8c429](https://github.com/hhanh00/zkool2/commit/4e8c429524b2f476587640fdaf58f1d799512706))
* conversion from USD to ZEC does not take locale into consideration ([#597](https://github.com/hhanh00/zkool2/issues/597)) ([9ea8436](https://github.com/hhanh00/zkool2/commit/9ea8436b0f670175c67635b132445b5ae6f77f6d))
* db schema upgrage ([#614](https://github.com/hhanh00/zkool2/issues/614)) ([286e493](https://github.com/hhanh00/zkool2/commit/286e4939f63c0e2fef867deba7ee49d405afaf8c))
* **ledger:** build break on mobile platforms (no support for ledger) ([#613](https://github.com/hhanh00/zkool2/issues/613)) ([9e15ed5](https://github.com/hhanh00/zkool2/commit/9e15ed5668b427414bc9727cefeeeb54e89cab5d))
* **ledger:** device thread serialization ([#626](https://github.com/hhanh00/zkool2/issues/626)) ([a009248](https://github.com/hhanh00/zkool2/commit/a0092486193950bae3aebbd61b192b54513df675))
* macos usb entitlements ([#620](https://github.com/hhanh00/zkool2/issues/620)) ([40755ff](https://github.com/hhanh00/zkool2/commit/40755ff795e863382f9f6a644155e2a8bf0f4bb3))
* no ledger build break ([#629](https://github.com/hhanh00/zkool2/issues/629)) ([96f3d87](https://github.com/hhanh00/zkool2/commit/96f3d87b98e9ec019761810ef91877097a01f566))
* remove dialog asking for scanning taddr on new accounts ([#582](https://github.com/hhanh00/zkool2/issues/582)) ([7a1ee58](https://github.com/hhanh00/zkool2/commit/7a1ee58f43008066975a2d607d57899e53e39227))
* remove extra column from query ([#618](https://github.com/hhanh00/zkool2/issues/618)) ([5c5b113](https://github.com/hhanh00/zkool2/commit/5c5b11378cdcd27f218dc5c5729257f14b64744e))
* use get_address_sapling to avoid div_list bug ([#627](https://github.com/hhanh00/zkool2/issues/627)) ([86ed507](https://github.com/hhanh00/zkool2/commit/86ed5077dd6515aaa427b7c108cafa1b84874c2a))

## [6.4.0](https://github.com/hhanh00/zkool2/compare/zkool-v6.3.1...zkool-v6.4.0) (2025-09-29)


### Features

* encrypt wallet file with age/zstd ([#579](https://github.com/hhanh00/zkool2/issues/579)) ([6943da2](https://github.com/hhanh00/zkool2/commit/6943da2c2a4286f719af58d1860162e69313528c))

## [6.3.1](https://github.com/hhanh00/zkool2/compare/zkool-v6.3.0...zkool-v6.3.1) (2025-09-28)


### Bug Fixes

* database encryption form ([#577](https://github.com/hhanh00/zkool2/issues/577)) ([a4e7a8f](https://github.com/hhanh00/zkool2/commit/a4e7a8f3dfd6d5a2843ea2f44e904f3768dfc52e))

## [6.3.0](https://github.com/hhanh00/zkool2/compare/zkool-v6.2.2...zkool-v6.3.0) (2025-09-28)


### Features

* transparent scan for addresses page ([#575](https://github.com/hhanh00/zkool2/issues/575)) ([3c3182e](https://github.com/hhanh00/zkool2/commit/3c3182ee07ea17e528301fefbe43141facc2e8be))

## [6.2.2](https://github.com/hhanh00/zkool2/compare/zkool-v6.2.1...zkool-v6.2.2) (2025-09-28)


### Bug Fixes

* db creation with no password ([#573](https://github.com/hhanh00/zkool2/issues/573)) ([aff19e8](https://github.com/hhanh00/zkool2/commit/aff19e8fc31a6dd41af96c1cbd116017d8c03930))

## [6.2.1](https://github.com/hhanh00/zkool2/compare/zkool-v6.2.0...zkool-v6.2.1) (2025-09-26)


### Bug Fixes

* add repeated password and validation to forms ([#572](https://github.com/hhanh00/zkool2/issues/572)) ([a61e1bf](https://github.com/hhanh00/zkool2/commit/a61e1bf8c5ed1e86d51db30e7246abfe1dd444f8))
* export category to tx csv as name ([#569](https://github.com/hhanh00/zkool2/issues/569)) ([48901fa](https://github.com/hhanh00/zkool2/commit/48901fa14705cfd5b91fe46473222bb40fca31e0))
* I/O of is_income in category table ([#571](https://github.com/hhanh00/zkool2/issues/571)) ([e4bf10b](https://github.com/hhanh00/zkool2/commit/e4bf10b7e4a8eff3c514190cabc389f62aa17923))

## [6.2.0](https://github.com/hhanh00/zkool2/compare/zkool-v6.1.0...zkool-v6.2.0) (2025-09-25)


### Features

* export of tx/memos/notes to csv ([#562](https://github.com/hhanh00/zkool2/issues/562)) ([9f281e7](https://github.com/hhanh00/zkool2/commit/9f281e7ce150480e75edea7504ed049917d3d9aa))
* unlock all notes & lock based on maturity ([#564](https://github.com/hhanh00/zkool2/issues/564)) ([552cc78](https://github.com/hhanh00/zkool2/commit/552cc786ef84670d12414d75d187b26a580d323f))


### Bug Fixes

* account for locked notes in max amount calculation ([#565](https://github.com/hhanh00/zkool2/issues/565)) ([2209073](https://github.com/hhanh00/zkool2/commit/22090737dd9f2162c56a7b3260dce7d7e921be74))
* chart sizes and margins ([#568](https://github.com/hhanh00/zkool2/issues/568)) ([b784c51](https://github.com/hhanh00/zkool2/commit/b784c5194c146538b266c8be21e2f008facdeb0f))
* do not show saved confirmation if canceled ([#566](https://github.com/hhanh00/zkool2/issues/566)) ([df848d8](https://github.com/hhanh00/zkool2/commit/df848d8a6f453e4a17a79f6c8657ba3d50496d0e))
* try to decode as string before bytes ([#567](https://github.com/hhanh00/zkool2/issues/567)) ([b1ee770](https://github.com/hhanh00/zkool2/commit/b1ee7703cef32521d4a0d68006fcbb46d45d0253))

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
