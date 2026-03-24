## [1.1.0](https://github.com/vernonthedev/nestforge/compare/v1.0.0...v1.1.0) (2026-02-27)

### Features

* add `run_all_features.ps1` PowerShell script to automate comprehensive project testing and CLI command verification. ([b75d7cb](https://github.com/vernonthedev/nestforge/commit/b75d7cb1f1ffd640fc2a73c52f46097efe423b81))
* added `nestforge-core` crate with request parameter, body, and input validation. ([a613f66](https://github.com/vernonthedev/nestforge/commit/a613f668e1a27ccc01d19c503c3f342c9b3caa22))
* added comprehensive api framework logging for all application requests ([955be3c](https://github.com/vernonthedev/nestforge/commit/955be3c4f8e4cfb13f1115aefec96b2e767096ac))
* added configuration module setups to handle envs ([4884589](https://github.com/vernonthedev/nestforge/commit/48845890a54575f3d02b622edffd18fb50f22af6))
* added guards, interceptors generation to the nestforge cli ([88c0670](https://github.com/vernonthedev/nestforge/commit/88c0670da5314e6174692a9bdbcda2ea0a220a61))
* added NestForge CLI commands for app scaffolding, code generation, database management, documentation, and formatting. ([eefcbe0](https://github.com/vernonthedev/nestforge/commit/eefcbe00ae77427ce7b66a7a36a8681f8355df52))
* added NestForge CLI db scaffolding for projec database management ([9aafb71](https://github.com/vernonthedev/nestforge/commit/9aafb719262a0438dce92b86775fa5afc6c60d4d))
* **documentation:** added  comprehensive documentation covering core concepts, module system, configuration, resource services, and a quick start guide, alongside an updated README. ([aabaacb](https://github.com/vernonthedev/nestforge/commit/aabaacb15756a881c22a1f841ed47f07c000ab2b))
* established the initial multi-crate NestForge framework structure, including core module system, HTTP server, CLI, and various integration crates. ([7fa06dc](https://github.com/vernonthedev/nestforge/commit/7fa06dc79be41f1522d4cb79b2231b4dad63330b))
* generated the openai skeleton docs json file ([a44e715](https://github.com/vernonthedev/nestforge/commit/a44e715fb5bfe195561f0e5bfa009ccd14254740))
* implemented `Param`, `Body`, `ValidatedBody` request extractors and `Inject` for dependency injection, demonstrated in the users controller. ([d58d0d9](https://github.com/vernonthedev/nestforge/commit/d58d0d966cd85baaa23ce042159d550f4fd092a0))
* implemented core framework components including DI container, module system, and testing utilities. ([7f2e878](https://github.com/vernonthedev/nestforge/commit/7f2e878a2a54db962f6dab107b48be733304d0c7))
* implemented lang prefixes, api versioning, interceptors & guards ([a3fad3f](https://github.com/vernonthedev/nestforge/commit/a3fad3f4935c14d0880669fd65557dbbf640bd15))
* initialized the NestForge framework with core module and dependency injection provider systems ([95433fd](https://github.com/vernonthedev/nestforge/commit/95433fd95af30d6da0d333280f0097140e8f3538))
* introduced `nestforge-db` crate for database integration and demonstrate its usage in a new example. ([6b343ad](https://github.com/vernonthedev/nestforge/commit/6b343adf6a5315bb27c9ac22785be632dd6aa0a1))
* introduced nestforge framework with core, procedural macros for DI/routing/ORM, and an ORM module. ([9be65b3](https://github.com/vernonthedev/nestforge/commit/9be65b3ac8fef0be3eb5c6a51eb2d796cf2474d6))
* properly structured module & resource generation directory structure to a nestjs schema & a joi mock ([224e530](https://github.com/vernonthedev/nestforge/commit/224e5306c41e3059a05ab33161dd06b990d6240e))
* setup NestForge CLI for application scaffolding, code generation, and database management. ([094281d](https://github.com/vernonthedev/nestforge/commit/094281ded58771458c6d3ae8b119a53bf7d187a7))
* setup the core module system with graph initialization, dependency resolution, and macro support for module definition. ([85a3d9f](https://github.com/vernonthedev/nestforge/commit/85a3d9fe22bb773e752215e7c57ebe0f93bff5d0))
* simplified the services to reduce usage of &self kind of setups ([90fdb50](https://github.com/vernonthedev/nestforge/commit/90fdb50724ec0e4bfd801dc59dc3fe7105d23eb7))

## 1.0.0 (2026-02-25)

### Features

* added `nestforge-core` crate with `ControllerDefinition` and `ModuleDefinition` traits. ([5a794fb](https://github.com/vernonthedev/nestforge/commit/5a794fbb477fcd7fa7ac219938965341f16f2ffb))
* added custom error handling that return proper json responses using `serde` ([faa9e80](https://github.com/vernonthedev/nestforge/commit/faa9e8008398caddcad866e2be21ab4271608d6d))
* added the new macros setup to the example app and also simplified all the controller code setup ([eb04d6d](https://github.com/vernonthedev/nestforge/commit/eb04d6da80ce646ed433c78fa41b302135ba33b8))
* **docs:** created documentation wikis and ensured they are synced using a github action ([3f8add0](https://github.com/vernonthedev/nestforge/commit/3f8add0255805ab9cf499ea813bcd15b04dd8fc3))
* established a public API for `nestforge` crate by re-exporting core components. ([7671ad6](https://github.com/vernonthedev/nestforge/commit/7671ad6722cc9f2bf0c1057d5886c17ff59d7e1c))
* implemented a new CLI for NestForge to scaffold applications and generate resources, controllers, and services. ([4dec36d](https://github.com/vernonthedev/nestforge/commit/4dec36d5fb7508d7e625abea4c266e4d958f45f2))
* implemented the structured example app with nestjs dir structure ([15a73c0](https://github.com/vernonthedev/nestforge/commit/15a73c05c82f7f882869fef83a123d26e78ec725))
* initialized the Rust workspace with `nestforge`, `nestforge-core` lib setup ([00ed241](https://github.com/vernonthedev/nestforge/commit/00ed241fe41b5a336e5b0dd85d842cdf5af02da0))
* introduce `Inject<T>` helper for dependency resolution from the container. ([0200c3d](https://github.com/vernonthedev/nestforge/commit/0200c3dae6543d7d4473c5c1371b0149b634be09))
* introduce `nestforge-core` with core framework components and `nestforge` as the public API crate. ([7055c93](https://github.com/vernonthedev/nestforge/commit/7055c93d7c515a1ff60989d541645394c101a898))
* setup `ModuleDefinition` trait for registering services into the dependency injection container. ([b66eb30](https://github.com/vernonthedev/nestforge/commit/b66eb303e3803af2ee7f4dc967bc8b152971af9e))
* setup the core framework components for dependency injection, modularity, and declarative routing with procedural macros. ([df092e9](https://github.com/vernonthedev/nestforge/commit/df092e9b9b85f148c6eb5de5491d2da93d455350))
* setup the core module and controller definitions, and an HTTP factory for application bootstrapping ([5bd9fa4](https://github.com/vernonthedev/nestforge/commit/5bd9fa430ace114a1e3a4040419c87c83d766580))
* setup the NestForgeFactory for HTTP server bootstrapping and module registration ([d08a235](https://github.com/vernonthedev/nestforge/commit/d08a2358aa8e2dc639c62862aa205b21c49968ca))

# Changelog

All notable changes to this project are documented in this file.

Documentation wiki: https://github.com/vernonthedev/nestforge/wiki


## [1.2.0](https://github.com/vernonthedev/nestforge/compare/v1.1.0...v1.2.0) (2026-03-01)

### Features

* feat: replace release plz with a direct workspace release and publish flow ([37837d0](https://github.com/vernonthedev/nestforge/commit/37837d0a654ec51646173a456380e515fb7690b0))
* feat: publish crate releases directly on pushes to main ([00070d6](https://github.com/vernonthedev/nestforge/commit/00070d68285bf493d9ac2ea5cc42d23787fa446b))
* feat: switch the workspace release flow to release plz with crates io publishing ([93827f5](https://github.com/vernonthedev/nestforge/commit/93827f5dcb5a3fa4b70f166c98467f2cffb2cb6c))
* feat: scaffolded request decorators directly from the cli ([a96dfbf](https://github.com/vernonthedev/nestforge/commit/a96dfbfb43a11522e326ee1b486ae077a9e61089))
* feat: scaffolded microservices first apps directly from the cli ([c4792bd](https://github.com/vernonthedev/nestforge/commit/c4792bd242fe00c374196e7d81b47e7bff3bef82))
* feat: added a dedicated microservices example app with an in process client ([43df8e5](https://github.com/vernonthedev/nestforge/commit/43df8e5297da27b2daa80dfb967ac59307dfef29))
* feat: added testing helpers for in process microservice clients ([b6bce39](https://github.com/vernonthedev/nestforge/commit/b6bce39868165f9aa246b05c422fc6a29bb2701e))
* feat: added an in process microservice client for pattern based messaging ([61c8958](https://github.com/vernonthedev/nestforge/commit/61c8958f6ef47d954888ed876c99d02f9a3829f6))
* feat: updated the grpc example to dispatch requests through microservice patterns ([ba43537](https://github.com/vernonthedev/nestforge/commit/ba4353758cabb008d355fa4b248df5d8a5a17b27))
* feat: updated the websocket example to use microservice message patterns ([ec13cc9](https://github.com/vernonthedev/nestforge/commit/ec13cc96f51ff5548d794e6cf2fd91af57317a76))
* feat: added serializer generators and documented serialized responses in the example app ([2921b69](https://github.com/vernonthedev/nestforge/commit/2921b69d9b536e8b6c5cafda0cd2d211c3078003))
* feat: added response serializer policies for shaping handler payloads ([82478e9](https://github.com/vernonthedev/nestforge/commit/82478e93524b8f47fecd17ef20616d5c42fd8b77))
* feat: scaffolded microservice pattern registries directly from the cli ([0ffa994](https://github.com/vernonthedev/nestforge/commit/0ffa9947b3e57d48dce054c1b2dda018edb3556e))
* feat: bridged grpc and websocket transports into the microservice registry ([5b72676](https://github.com/vernonthedev/nestforge/commit/5b72676af9bed01fe443e54eebeda58d5558cf5b))
* feat: added transport testing helpers for grpc websockets and microservice contexts ([ab76118](https://github.com/vernonthedev/nestforge/commit/ab76118229a50cf7248eaf6a91c7e6ed726cffa8))
* feat: added a dedicated nestforge microservices crate with message and event patterns ([263670d](https://github.com/vernonthedev/nestforge/commit/263670d1b18a858d0c8bc89f0b4fa295f48bc130))
* feat: added method aware middleware route matching and exclusions ([5639537](https://github.com/vernonthedev/nestforge/commit/56395373189a9c2261f6cfe19467be94c9336dc3))
* feat: added module graph introspection for framework diagnostics and tooling ([d2c93d8](https://github.com/vernonthedev/nestforge/commit/d2c93d8e18ec8a9f8197b7bf3d4fc9fa550b2578))
* feat: added a dedicated nestforge cache crate with response cache interceptors ([d42b9c1](https://github.com/vernonthedev/nestforge/commit/d42b9c1992b120605018c074066770e3ce8ca9e3))
* feat: added first class request decorators for custom handler extraction ([4812d83](https://github.com/vernonthedev/nestforge/commit/4812d83088f071ff5da12477cb224b9e1c643e43))
* feat: added first class response envelope helpers for standardised json payloads ([da50df1](https://github.com/vernonthedev/nestforge/commit/da50df14820704642dc699bbada9896ce3c10e6f))
* feat: added cli scaffolding for exception filters and updated generator docs ([75da36d](https://github.com/vernonthedev/nestforge/commit/75da36d7077968c82945683e970aac2a7c4b6de0))
* feat: added route and controller level exception filters to the request pipeline ([3b5ba7c](https://github.com/vernonthedev/nestforge/commit/3b5ba7c1971f70a762c2502e63cb1c3cc94c2383))
* feat: added controller level guard interceptor auth and tag metadata inheritance ([30e5457](https://github.com/vernonthedev/nestforge/commit/30e5457a939e40afa053e1b5da8c5b45b4d67698))
* feat: added cli generators for middleware and websocket gateways ([a6887a6](https://github.com/vernonthedev/nestforge/commit/a6887a620a8f53ab54f16a0af5af6276cea8556d))
* feat: added named schedule jobs and a fluent registry builder api ([33b0440](https://github.com/vernonthedev/nestforge/commit/33b044025f05e6c049a244717ce25259594c449e))
* feat: added a builder based dynamic module api with typed exports and async registration ([d8d2a6e](https://github.com/vernonthedev/nestforge/commit/d8d2a6e7a95a8450d48c49c0e8f6886e946dbd44))
* feat: added explicit testing module shutdown hooks for lifecycle aware cleanup ([29fc650](https://github.com/vernonthedev/nestforge/commit/29fc650c7421ee0d710586a91bebc3bfa93721f5))
* feat: added http and graphql testing harness helpers on top of the module runtime ([1d2585a](https://github.com/vernonthedev/nestforge/commit/1d2585a0a8ba8b9a2ea807d77c0ad1392f3ce104))
* feat: added a websocket first example app and cli transport scaffolding ([d5284e0](https://github.com/vernonthedev/nestforge/commit/d5284e0683a954a3a6404534043d7fc248af1ba5))
* feat: added a nest style middleware consumer for route targeted http middleware ([54f4469](https://github.com/vernonthedev/nestforge/commit/54f446928eb68d1a4fbd3d46a4a067032b1f7ef6))
* feat: added a dedicated nestforge websockets crate with gateway mounting helpers ([be7bbcd](https://github.com/vernonthedev/nestforge/commit/be7bbcd52726698e632f79ffbe8450d29027237b))
* feat: added transient provider factories for short lived dependency resolution ([8000af0](https://github.com/vernonthedev/nestforge/commit/8000af09d4c954495057c129889cf54dd2d39a21))
* feat: added scheduler support with lifecycle driven startup and shutdown hooks ([1fa67cf](https://github.com/vernonthedev/nestforge/commit/1fa67cf1bef3f65bcc1fa1be5bdd2465fda4f6f6))
* feat: added dynamic module refs so imports can capture runtime registration options ([3a25c13](https://github.com/vernonthedev/nestforge/commit/3a25c13eb4dbdcd357b594344f2abb7eb9afa6e6))
* feat: added request scoped provider factories that resolve from per request containers ([5bfbfed](https://github.com/vernonthedev/nestforge/commit/5bfbfed8c468f0108bd6d7adba633d9a76f860a5))
* feat: added first class request pipes for params queries and json bodies ([07f2566](https://github.com/vernonthedev/nestforge/commit/07f2566d8831804dfdbcb39247f42b6c0bb70f1b))
* feat: added global exception filters to the nestforge request pipeline ([ef59d78](https://github.com/vernonthedev/nestforge/commit/ef59d789be6ecffa99c8f62c51d1aafa369941bb))
* feat: added nest style module lifecycle hooks across the framework runtime ([7d99fb3](https://github.com/vernonthedev/nestforge/commit/7d99fb3a1a08235af002878d47c49adf5ae1a039))
* feat: added graphql dependency resolution helpers on top of resolver context ([11659c5](https://github.com/vernonthedev/nestforge/commit/11659c5c90fedad5f72de2de1390915975e767b0))
* feat: injected the nestforge container into graphql resolver context ([4394ce0](https://github.com/vernonthedev/nestforge/commit/4394ce04f8a328838ad378400cdadd7f25b70171))
* feat: added graphql and grpc generators so transport apps can grow incrementally ([afcfca0](https://github.com/vernonthedev/nestforge/commit/afcfca0dab1031122af6fe9f52c8989188a0cc24))
* feat: scaffolded graphql and grpc app templates directly from the cli ([bb41708](https://github.com/vernonthedev/nestforge/commit/bb41708aa6ce1179de27a1d8baa7031f49dfdb05))
* feat: added a grpc first example app with tonic proto scaffolding ([23d5f4d](https://github.com/vernonthedev/nestforge/commit/23d5f4d5f0e95fa61d4263a6e852f42eb09db50e))
* feat: added a dedicated nestforge grpc crate with tonic based transport factory support ([93dc326](https://github.com/vernonthedev/nestforge/commit/93dc32682a4119f0b146d87afd60e1a020c7f82b))
* feat: added a graphql first example app wired through nestforge factory and schema helpers ([f8b0ce1](https://github.com/vernonthedev/nestforge/commit/f8b0ce1ff6616a004b1f57bb2f296d315a2aed7e))
* feat: added a dedicated nestforge graphql crate with graphiql and factory integration helpers ([c8702b0](https://github.com/vernonthedev/nestforge/commit/c8702b0935b1a325c48b95d417f13bb2646a8d9b))
* feat: added optional auth extraction helpers and openapi factory helpers with updated docs ([dfd2835](https://github.com/vernonthedev/nestforge/commit/dfd2835ee4b302791bb434e003d14a4f62241da0))
* feat: added auth aware guards, common http extractors, and direct openapi router mounting support ([590c721](https://github.com/vernonthedev/nestforge/commit/590c721820594f1c67cc5e545f006f3d8a5bbf30))
* feat: generated openapi documents from module route metadata and documented controller annotations ([2752520](https://github.com/vernonthedev/nestforge/commit/2752520187f24a6961313758f522ac65de8d1de0))
* feat: added auth identities, bearer token extraction, and request scoped auth resolution ([bf62e39](https://github.com/vernonthedev/nestforge/commit/bf62e39000f2e9546926d0e515726ebf6f48e756))
* feat: executed migration files as single transactional sql scripts instead of splitting statements ([ad9880c](https://github.com/vernonthedev/nestforge/commit/ad9880c559d480f12279d25676f8a12eaa6acf92))
* feat: expanded dto validation rules and added first class typed query extraction ([83ff24c](https://github.com/vernonthedev/nestforge/commit/83ff24c9580bbca57e6a79e284b558f2d7745522))
* feat: added container override layers so testing modules can override providers transitively ([f0d3f4d](https://github.com/vernonthedev/nestforge/commit/f0d3f4dd28604d3be61c2624fb6d2116ccbc3a21))
* feat: introduced request ids, structured framework logging, and standardised http error responses ([3ac30b2](https://github.com/vernonthedev/nestforge/commit/3ac30b2c49ee6b9b5104e19e321f8c9275dc3499))
* feat: introduce procedural macros for controllers and modules, add HTTP extension traits, and integrate in-memory MongoDB. ([685c242](https://github.com/vernonthedev/nestforge/commit/685c242674b57cbc24aa6bb9044eeb7413c2b4dc))

### Fixes

* fix: retry crate publishes after crates io rate limits ([80ccdf8](https://github.com/vernonthedev/nestforge/commit/80ccdf8ff63100f9c9b74b89aaee6986a2b2a267))
* fix: publish microservice crates before grpc dependencies ([624e70c](https://github.com/vernonthedev/nestforge/commit/624e70cfa46bfe4cea646bac6f13b138b3876964))
* fix: remove protoc as a build requirement from the grpc example app ([9e92501](https://github.com/vernonthedev/nestforge/commit/9e9250162c50d031c9dbe0eacaa9a4cdfba36c3b))
* fix: remove leftover warning-only code from the example apps ([e815773](https://github.com/vernonthedev/nestforge/commit/e81577364ece6333b85fc6b6caa328c7407f3ff6))
* fix: make local workspace checks reliable without sqlite toolchain blockers ([00206a4](https://github.com/vernonthedev/nestforge/commit/00206a4982f68f967b8278848df538cd2e2cad57))
* fix: return an owned app name from the graphql example resolver ([f3445d4](https://github.com/vernonthedev/nestforge/commit/f3445d437e2701076c62ee4258970fe6bc95ce91))
* fix: add missing grpc example dependencies for module and json usage ([2e7f182](https://github.com/vernonthedev/nestforge/commit/2e7f182087a02531ae5bff0156ee2512a0fa2bba))
* fix: add missing example dependencies for graphql and websocket builds ([6eee612](https://github.com/vernonthedev/nestforge/commit/6eee612e34ec3c637c0fbfb069aab3b7d5f6500f))
* fix: resolve example app compile errors uncovered by the release flow ([734e15a](https://github.com/vernonthedev/nestforge/commit/734e15aff14fe4f7743321bf9de4622d0c0e1bfe))
* fix: make axum a runtime dependency of the nestforge crate ([77526fd](https://github.com/vernonthedev/nestforge/commit/77526fd893b5ab1f1eb7fa2ee984b8e8ad95749b))
* fix: use a safe regex evaluator for workspace version bumps ([684e682](https://github.com/vernonthedev/nestforge/commit/684e682eac62f831489a3198369377f6655cba79))
* fix: avoid uninitialized lastexitcode checks in the release script ([5e1a5c3](https://github.com/vernonthedev/nestforge/commit/5e1a5c36d9bc8d449510f489017d4457a73a0a92))
* fix: add package readmes for release and crates io metadata ([4cbbbf5](https://github.com/vernonthedev/nestforge/commit/4cbbbf56619240673f81712f3b7f54e52ccb7d5f))
* fix: point release plz changelog updates at the nestforge crate ([69247da](https://github.com/vernonthedev/nestforge/commit/69247da5197a680eb753358500c32dee22a536e2))
* fix: resolve framework compile errors across graphql http cache and websockets ([cdf9d30](https://github.com/vernonthedev/nestforge/commit/cdf9d30544afa21c8a988b69ded9d7ed348ae25a))
* fix: propagated scoped request context into graphql resolvers and helper accessors ([259aa2a](https://github.com/vernonthedev/nestforge/commit/259aa2a72cf059ba0a8dce4e90a6bd8dfac6b77e))
* fix: enforced authenticated and role based route metadata at runtime and in openapi output ([de7d618](https://github.com/vernonthedev/nestforge/commit/de7d6180b62a04da116ad882b4ecef9b1e9ff090))

### Other

* chore: update Rust dependencies. ([77739b8](https://github.com/vernonthedev/nestforge/commit/77739b8b144ca6494f663b96b6fa8687b7aa540b))
* chore: Update Rust dependencies ([0c3eda9](https://github.com/vernonthedev/nestforge/commit/0c3eda93c8199464cc567d755cc78fef2d2f225b))
* chore: merged pull request #4 from vernonthedev/feature/auto-documentation ([d4f0ba7](https://github.com/vernonthedev/nestforge/commit/d4f0ba753311c61289e1a6f0ac82f85fe5d6011a))
* chore: merged pull request #3 from vernonthedev/feature/auto-documentation ([7d4aa1c](https://github.com/vernonthedev/nestforge/commit/7d4aa1c5ef369769255d2cf8df375a068ef26fd0))
* chore: merged pull request #2 from vernonthedev/feature/auto-documentation ([dbf0905](https://github.com/vernonthedev/nestforge/commit/dbf09059d07bf6004fa90d0f2294446081ac98d2))
* docs: updated quick start flows for graphql and grpc app scaffolding ([d88b20f](https://github.com/vernonthedev/nestforge/commit/d88b20f4461d2214ea4ab5177bad3620d9a45d5e))
* chore: removed unnecessary commit file that popped up during development ([6c02a67](https://github.com/vernonthedev/nestforge/commit/6c02a6754e10079662b50ee4209102a755ab0635))
* chore: modularize NestForge into new crates, bump workspace version to 1.1.0, and add a release publishing script ([6c4ef46](https://github.com/vernonthedev/nestforge/commit/6c4ef463ab253dc253d4f65338eb51017f513e04))



## [1.2.1](https://github.com/vernonthedev/nestforge/compare/v1.2.0...v1.2.1) (2026-03-06)

### Fixes

* fix: removed all unused vars ([e8bd504](https://github.com/vernonthedev/nestforge/commit/e8bd504fe8f1c8376bb6416d631fa76127d5f56a))

### Other

* docs: updated application documentations ([53ae54b](https://github.com/vernonthedev/nestforge/commit/53ae54bd04eb7e27a07cec9e5cbf199acdeeb324))
* chore: application fixes for the primitive bugs ([2602290](https://github.com/vernonthedev/nestforge/commit/2602290d0f5c742f8a1574fa3b3e8d8fdd864e99))
* docs: updated version number to 1.2.0 in README ([543abfc](https://github.com/vernonthedev/nestforge/commit/543abfcef53e7fb96fc25412c3005ccd361a2ac7))



## [1.3.0](https://github.com/vernonthedev/nestforge/compare/v1.2.1...v1.3.0) (2026-03-10)

### Features

* feat: setup integrated openai docs autogenerations, fixes #8 ([20e8510](https://github.com/vernonthedev/nestforge/commit/20e8510b1d5b888875aba9a118885f466a72ac1f))
* feat: added comprehensive issue templates,  fixes #7 ([02a9f8e](https://github.com/vernonthedev/nestforge/commit/02a9f8e2740c6c2f960b7b5fec5e0a6826b92d13))
* feat: updated core nested layout to remove default controller & service dirs leaving them for module setups only ([3864823](https://github.com/vernonthedev/nestforge/commit/3864823045c770de921f5d83ccbb7d52ab0ff717))
* feat: implemented a Flat Module Structure for the CLI auto generations, fixes #6 ([2095ef9](https://github.com/vernonthedev/nestforge/commit/2095ef935bc865fd938f89b72c60973931c28d6b))

### Fixes

* fix(openapi): support serialized response schemas ([1d13544](https://github.com/vernonthedev/nestforge/commit/1d1354481b8e72de0cc42202b7c228b0c003dcc4))
* fix(security): added bounded GraphQL requests without trusting thirdparties, fixed #2 ([e333c48](https://github.com/vernonthedev/nestforge/commit/e333c4825bd7776a7831aa394df17aa8668d7cca))
* fix: fixed the proc error on project creation, fixes #5 ([3ce4542](https://github.com/vernonthedev/nestforge/commit/3ce454218c450f5ea7d0e00191309c1017431eea))

### Other

* refactor(dtos): updated cli to handle prompts for dto data structures ([eb6d7db](https://github.com/vernonthedev/nestforge/commit/eb6d7dbf8cc0ecb3c5f41525f20b8e1e5aa22ddf))
* docs: added flat layout structure ([410de8d](https://github.com/vernonthedev/nestforge/commit/410de8d61d7927ed328ee262f745ddd7ab8f3769))



## [1.4.0](https://github.com/vernonthedev/nestforge/compare/v1.3.0...v1.4.0) (2026-03-10)

### Features

* feat(cli): show nestforge banner in tui ([732e62f](https://github.com/vernonthedev/nestforge/commit/732e62f7d6891ce5c1a2d4e926ea8a8644a73ec5))
* feat(cli): use block-style nestforge banner ([d320a24](https://github.com/vernonthedev/nestforge/commit/d320a24e6fcc4dff6a5d5d1e8d6de9b032ddd539))
* feat(cli): add branded terminal banner ([6b4c51f](https://github.com/vernonthedev/nestforge/commit/6b4c51f2f1e0c9e007d1a7e78aa550afe2f75382))
* feat(cli): scaffold openapi in new apps ([567b1d5](https://github.com/vernonthedev/nestforge/commit/567b1d5206baa51eaeea75622049fa1d809b5c04))
* feat(cli): make generate tui step-based ([79c88ad](https://github.com/vernonthedev/nestforge/commit/79c88ad2b2b9720a6e5307be4b2e61410d76613f))
* feat(cli): add typed miette diagnostics ([031a874](https://github.com/vernonthedev/nestforge/commit/031a8745f111d621000b97b2bbf2973049fb493a))
* feat(cli): add ratatui wizards for new and generate ([4d6d2f6](https://github.com/vernonthedev/nestforge/commit/4d6d2f6bf42fd08f8bb1b18809d67d5843dfe5d9))
* feat(cli): add structured commands and rich terminal output ([8d5b2d9](https://github.com/vernonthedev/nestforge/commit/8d5b2d9e510685750a2a25963998350a0dfae7a0))

### Fixes

* fix(cli): fixed the new project scaffold to stop creating empty services folders, fixes #16 ([ee4d333](https://github.com/vernonthedev/nestforge/commit/ee4d33360e2282910521f48e790b4088b0afb1c5))
* fix(modules): fixed the bug report issue form so github can recognize it properly, fixes #17 ([a483abf](https://github.com/vernonthedev/nestforge/commit/a483abfa84732015603cdb8b9ef788a2b085dfdf))
* fix(cli): fixed the graphql starter dependency and macro imports ([a848483](https://github.com/vernonthedev/nestforge/commit/a848483aa65dfc3776f91c8ce8ed48f47ac9a389))
* fix(cli): fixed the graphql scaffold async-graphql imports ([71624b8](https://github.com/vernonthedev/nestforge/commit/71624b8fa725977d26758c0e1162ec81be2cd79c))
* fix(cli): keep new wizard transport stable ([eaf68a7](https://github.com/vernonthedev/nestforge/commit/eaf68a73fc2110c5d3efe50240b3583f1a2b4886))
* fix(cli): remove redundant tui titles ([ebff157](https://github.com/vernonthedev/nestforge/commit/ebff157af77c464c0a1be0d95feb9a662b068d04))
* fix(cli): repair microservices scaffold template ([0c2f3d9](https://github.com/vernonthedev/nestforge/commit/0c2f3d9850185abe3cd5537d24ab3dc74d0bd171))
* fix(openai): fixed the entire openapi docs generations ([e13304e](https://github.com/vernonthedev/nestforge/commit/e13304ee17bd666a00f0181eb63594270b768b9a))
* fix(openapi): use docs-relative spec urls ([4496b03](https://github.com/vernonthedev/nestforge/commit/4496b03858bbde7ac198400e3b3f6d2fdc2db8d1))
* fix(cli): use prompt wizard in git bash ([4f2928f](https://github.com/vernonthedev/nestforge/commit/4f2928f2a8cd6a36096be53d2f8192d1ac82d267))
* fix(cli): add numeric choices to generate wizard ([24f988d](https://github.com/vernonthedev/nestforge/commit/24f988dfe82edf8c7c8a4b0fa75ae8a6eb11eac2))
* fix(cli): use directional controls in generate wizard ([93b68ab](https://github.com/vernonthedev/nestforge/commit/93b68abdd4ba02a787b9759a64c564b12d7694e4))
* fix(cli): simplify ratatui field rendering ([9263823](https://github.com/vernonthedev/nestforge/commit/926382319a1bace2fd856f244f4ff710d48a4c79))
* fix(cli): clarify ratatui selectors and toggles ([453936a](https://github.com/vernonthedev/nestforge/commit/453936a0d6f4b47dfb1be054e6a28e55ea6c5d32))
* fix(cli): improve ratatui field entry and prompt fallback ([8c1e471](https://github.com/vernonthedev/nestforge/commit/8c1e4719f64920ab9d555c7cb3e8a049603843fc))

### Other

* docs: did a little docs cleanup with the new shields ([9c69a1e](https://github.com/vernonthedev/nestforge/commit/9c69a1e691918b337d23e6834f3e9731ff9313b9))
* docs(readme): added framework badges and the vscode extension section ([06a5510](https://github.com/vernonthedev/nestforge/commit/06a5510727cc70680b0b3a629f4a48ee74c79488))
* docs(community): added the missing repository health files and contribution automation, fixes #15 ([94a7598](https://github.com/vernonthedev/nestforge/commit/94a75987c62a936bc928b33c4cd9092cfcbcc438))
* chore: added dev exec example dir ([b4541c6](https://github.com/vernonthedev/nestforge/commit/b4541c6aad6b5159a84e6b8fa5e7f2af3b22b4bf))
* refactor(cli): split parser ui and diagnostics modules ([68522ba](https://github.com/vernonthedev/nestforge/commit/68522ba1619eb2c6af39261b987744793774f5f4))



## [1.4.1](https://github.com/vernonthedev/nestforge/compare/v1.4.0...v1.4.1) (2026-03-12)

### Other

* docs: removed raw rendered md files ([f7fc9c9](https://github.com/vernonthedev/nestforge/commit/f7fc9c9bdb43c2350d80220ab4ec95989cc266fd))



## [1.5.0](https://github.com/vernonthedev/nestforge/compare/v1.4.1...v1.5.0) (2026-03-13)

### Features

* feat(funding): added GitHub FUNDING metadata ([8afffb3](https://github.com/vernonthedev/nestforge/commit/8afffb30f4a5beb9b4c8c45a2db563cd600bfed9))
* feat(docs): completed the entire documentation workflow for the db based files ([a844524](https://github.com/vernonthedev/nestforge/commit/a844524b2384c1639623c4de73fdaa8d27491cc5))
* feat(docs): Added detailed docs for easier dev onboarding, fixes #14 ([b692f91](https://github.com/vernonthedev/nestforge/commit/b692f91cdc344f5b39e2b782a9d45a4bf7a7b9fb))
* feat(scaffold): add root lib barrels and document prelude ([df35447](https://github.com/vernonthedev/nestforge/commit/df354476a27140ec3ca013735674cbbf81a367a9))
* feat(cli): scaffold app lib and simplify templates, fixes #21 ([ecdc931](https://github.com/vernonthedev/nestforge/commit/ecdc9315293b442ff4aa8ac081c081fde103394d))
* feat(cli): enabled TUI when terminal is interactive ([c9bc1fc](https://github.com/vernonthedev/nestforge/commit/c9bc1fca93bc4e3e793993815da49a63fbec1f83))
* feat(cli): added interactive mode to support `bash` kind of terminals ([7d7e354](https://github.com/vernonthedev/nestforge/commit/7d7e354e43e6827d2e3247ebbbe3f9f5a3c8820e))
* feat(cli): allow Docs command to accept topic and no-tui options, fixes #20 ([01a2ed5](https://github.com/vernonthedev/nestforge/commit/01a2ed57d89ba524726704b54c06084194373654))
* feat(cli): add Docs command and separate openapi alias ([ea440f1](https://github.com/vernonthedev/nestforge/commit/ea440f1e1d65667d9ca0136876c15e52ac79056c))
* feat(macros): add injectable service registration ([770568a](https://github.com/vernonthedev/nestforge/commit/770568ac2c51b0dc34c2e257bab95452b5a5c3f2))
* feat(core): add injectable registration primitives ([f814251](https://github.com/vernonthedev/nestforge/commit/f8142519cc40350313c5245ede8653e9ee04fb90))

### Fixes

* fix(tests): replace manual dto derive with response_dto ([ed90f2d](https://github.com/vernonthedev/nestforge/commit/ed90f2d508b539da7cf11576c051c2b30b1b6c42))
* fix(cli): update module generation for nested layout ([0da89b6](https://github.com/vernonthedev/nestforge/commit/0da89b6c25ddb399436f68455be0cc6446c085c7))
* fix(cli): register services as providers ([68fcd49](https://github.com/vernonthedev/nestforge/commit/68fcd496971052e8ca3e2c0c61308a0ae207981a))
* fix(core): added service module injection structure, fixes #19 ([9ce6f4f](https://github.com/vernonthedev/nestforge/commit/9ce6f4ff5979d3408c37f30920e0a8b082cffee5))
* fix(dependencies): updated Iru from 0.12.5 to 0.16.3 ([d9ebb8e](https://github.com/vernonthedev/nestforge/commit/d9ebb8e705f5ac309319cf9a6dd402b93033396a))

### Other

* chore: merge pull request #22 from vernonthedev/develop ([6112453](https://github.com/vernonthedev/nestforge/commit/6112453fa8c0e0c629d111d88b1744bd677af0c6))
* docs: updated docs with the newly worked on features for injectable service structure & cli changes ([d9e4975](https://github.com/vernonthedev/nestforge/commit/d9e4975867b77bfddfa1cc03b0d15ff137701026))
* docs(cli): clarify OpenAPI docs and prelude guidance ([7deaa7c](https://github.com/vernonthedev/nestforge/commit/7deaa7cd33726f7e9cd4a10297bc88c1cbd8439e))
* chore(cli): drop redundant tui reliability notes ([bfd7838](https://github.com/vernonthedev/nestforge/commit/bfd7838bf7c2d517f9b891d75bf7b94bc0b80560))
* chore(vendor): add ratatui vendor tree and adjust lockfile ([6376f09](https://github.com/vernonthedev/nestforge/commit/6376f09bed6f90488a5b4a6546e9ea75e419a2d8))
* chore: simplified the service case study usage from derifs ([c147356](https://github.com/vernonthedev/nestforge/commit/c1473561d90bd69804562f23b61b8d718d40059d))
* docs(readme): document injectable services ([5554041](https://github.com/vernonthedev/nestforge/commit/5554041c84c72e0b7f1e733c9c4f4bb2cd55e22b))



## [1.6.0](https://github.com/vernonthedev/nestforge/compare/v1.6.0...v1.6.0) (2026-03-13)

### Features

* feat(funding): added GitHub FUNDING metadata ([8afffb3](https://github.com/vernonthedev/nestforge/commit/8afffb30f4a5beb9b4c8c45a2db563cd600bfed9))
* feat(docs): completed the entire documentation workflow for the db based files ([a844524](https://github.com/vernonthedev/nestforge/commit/a844524b2384c1639623c4de73fdaa8d27491cc5))
* feat(docs): Added detailed docs for easier dev onboarding, fixes #14 ([b692f91](https://github.com/vernonthedev/nestforge/commit/b692f91cdc344f5b39e2b782a9d45a4bf7a7b9fb))
* feat(scaffold): add root lib barrels and document prelude ([df35447](https://github.com/vernonthedev/nestforge/commit/df354476a27140ec3ca013735674cbbf81a367a9))
* feat(cli): scaffold app lib and simplify templates, fixes #21 ([ecdc931](https://github.com/vernonthedev/nestforge/commit/ecdc9315293b442ff4aa8ac081c081fde103394d))
* feat(cli): enabled TUI when terminal is interactive ([c9bc1fc](https://github.com/vernonthedev/nestforge/commit/c9bc1fca93bc4e3e793993815da49a63fbec1f83))
* feat(cli): added interactive mode to support `bash` kind of terminals ([7d7e354](https://github.com/vernonthedev/nestforge/commit/7d7e354e43e6827d2e3247ebbbe3f9f5a3c8820e))
* feat(cli): allow Docs command to accept topic and no-tui options, fixes #20 ([01a2ed5](https://github.com/vernonthedev/nestforge/commit/01a2ed57d89ba524726704b54c06084194373654))
* feat(cli): add Docs command and separate openapi alias ([ea440f1](https://github.com/vernonthedev/nestforge/commit/ea440f1e1d65667d9ca0136876c15e52ac79056c))
* feat(macros): add injectable service registration ([770568a](https://github.com/vernonthedev/nestforge/commit/770568ac2c51b0dc34c2e257bab95452b5a5c3f2))
* feat(core): add injectable registration primitives ([f814251](https://github.com/vernonthedev/nestforge/commit/f8142519cc40350313c5245ede8653e9ee04fb90))
* feat(cli): show nestforge banner in tui ([732e62f](https://github.com/vernonthedev/nestforge/commit/732e62f7d6891ce5c1a2d4e926ea8a8644a73ec5))
* feat(cli): use block-style nestforge banner ([d320a24](https://github.com/vernonthedev/nestforge/commit/d320a24e6fcc4dff6a5d5d1e8d6de9b032ddd539))
* feat(cli): add branded terminal banner ([6b4c51f](https://github.com/vernonthedev/nestforge/commit/6b4c51f2f1e0c9e007d1a7e78aa550afe2f75382))
* feat(cli): scaffold openapi in new apps ([567b1d5](https://github.com/vernonthedev/nestforge/commit/567b1d5206baa51eaeea75622049fa1d809b5c04))
* feat(cli): make generate tui step-based ([79c88ad](https://github.com/vernonthedev/nestforge/commit/79c88ad2b2b9720a6e5307be4b2e61410d76613f))
* feat(cli): add typed miette diagnostics ([031a874](https://github.com/vernonthedev/nestforge/commit/031a8745f111d621000b97b2bbf2973049fb493a))
* feat(cli): add ratatui wizards for new and generate ([4d6d2f6](https://github.com/vernonthedev/nestforge/commit/4d6d2f6bf42fd08f8bb1b18809d67d5843dfe5d9))
* feat(cli): add structured commands and rich terminal output ([8d5b2d9](https://github.com/vernonthedev/nestforge/commit/8d5b2d9e510685750a2a25963998350a0dfae7a0))
* feat: setup integrated openai docs autogenerations, fixes #8 ([20e8510](https://github.com/vernonthedev/nestforge/commit/20e8510b1d5b888875aba9a118885f466a72ac1f))
* feat: added comprehensive issue templates,  fixes #7 ([02a9f8e](https://github.com/vernonthedev/nestforge/commit/02a9f8e2740c6c2f960b7b5fec5e0a6826b92d13))
* feat: updated core nested layout to remove default controller & service dirs leaving them for module setups only ([3864823](https://github.com/vernonthedev/nestforge/commit/3864823045c770de921f5d83ccbb7d52ab0ff717))
* feat: implemented a Flat Module Structure for the CLI auto generations, fixes #6 ([2095ef9](https://github.com/vernonthedev/nestforge/commit/2095ef935bc865fd938f89b72c60973931c28d6b))
* feat: replace release plz with a direct workspace release and publish flow ([37837d0](https://github.com/vernonthedev/nestforge/commit/37837d0a654ec51646173a456380e515fb7690b0))
* feat: publish crate releases directly on pushes to main ([00070d6](https://github.com/vernonthedev/nestforge/commit/00070d68285bf493d9ac2ea5cc42d23787fa446b))
* feat: switch the workspace release flow to release plz with crates io publishing ([93827f5](https://github.com/vernonthedev/nestforge/commit/93827f5dcb5a3fa4b70f166c98467f2cffb2cb6c))
* feat: scaffolded request decorators directly from the cli ([a96dfbf](https://github.com/vernonthedev/nestforge/commit/a96dfbfb43a11522e326ee1b486ae077a9e61089))
* feat: scaffolded microservices first apps directly from the cli ([c4792bd](https://github.com/vernonthedev/nestforge/commit/c4792bd242fe00c374196e7d81b47e7bff3bef82))
* feat: added a dedicated microservices example app with an in process client ([43df8e5](https://github.com/vernonthedev/nestforge/commit/43df8e5297da27b2daa80dfb967ac59307dfef29))
* feat: added testing helpers for in process microservice clients ([b6bce39](https://github.com/vernonthedev/nestforge/commit/b6bce39868165f9aa246b05c422fc6a29bb2701e))
* feat: added an in process microservice client for pattern based messaging ([61c8958](https://github.com/vernonthedev/nestforge/commit/61c8958f6ef47d954888ed876c99d02f9a3829f6))
* feat: updated the grpc example to dispatch requests through microservice patterns ([ba43537](https://github.com/vernonthedev/nestforge/commit/ba4353758cabb008d355fa4b248df5d8a5a17b27))
* feat: updated the websocket example to use microservice message patterns ([ec13cc9](https://github.com/vernonthedev/nestforge/commit/ec13cc96f51ff5548d794e6cf2fd91af57317a76))
* feat: added serializer generators and documented serialized responses in the example app ([2921b69](https://github.com/vernonthedev/nestforge/commit/2921b69d9b536e8b6c5cafda0cd2d211c3078003))
* feat: added response serializer policies for shaping handler payloads ([82478e9](https://github.com/vernonthedev/nestforge/commit/82478e93524b8f47fecd17ef20616d5c42fd8b77))
* feat: scaffolded microservice pattern registries directly from the cli ([0ffa994](https://github.com/vernonthedev/nestforge/commit/0ffa9947b3e57d48dce054c1b2dda018edb3556e))
* feat: bridged grpc and websocket transports into the microservice registry ([5b72676](https://github.com/vernonthedev/nestforge/commit/5b72676af9bed01fe443e54eebeda58d5558cf5b))
* feat: added transport testing helpers for grpc websockets and microservice contexts ([ab76118](https://github.com/vernonthedev/nestforge/commit/ab76118229a50cf7248eaf6a91c7e6ed726cffa8))
* feat: added a dedicated nestforge microservices crate with message and event patterns ([263670d](https://github.com/vernonthedev/nestforge/commit/263670d1b18a858d0c8bc89f0b4fa295f48bc130))
* feat: added method aware middleware route matching and exclusions ([5639537](https://github.com/vernonthedev/nestforge/commit/56395373189a9c2261f6cfe19467be94c9336dc3))
* feat: added module graph introspection for framework diagnostics and tooling ([d2c93d8](https://github.com/vernonthedev/nestforge/commit/d2c93d8e18ec8a9f8197b7bf3d4fc9fa550b2578))
* feat: added a dedicated nestforge cache crate with response cache interceptors ([d42b9c1](https://github.com/vernonthedev/nestforge/commit/d42b9c1992b120605018c074066770e3ce8ca9e3))
* feat: added first class request decorators for custom handler extraction ([4812d83](https://github.com/vernonthedev/nestforge/commit/4812d83088f071ff5da12477cb224b9e1c643e43))
* feat: added first class response envelope helpers for standardised json payloads ([da50df1](https://github.com/vernonthedev/nestforge/commit/da50df14820704642dc699bbada9896ce3c10e6f))
* feat: added cli scaffolding for exception filters and updated generator docs ([75da36d](https://github.com/vernonthedev/nestforge/commit/75da36d7077968c82945683e970aac2a7c4b6de0))
* feat: added route and controller level exception filters to the request pipeline ([3b5ba7c](https://github.com/vernonthedev/nestforge/commit/3b5ba7c1971f70a762c2502e63cb1c3cc94c2383))
* feat: added controller level guard interceptor auth and tag metadata inheritance ([30e5457](https://github.com/vernonthedev/nestforge/commit/30e5457a939e40afa053e1b5da8c5b45b4d67698))
* feat: added cli generators for middleware and websocket gateways ([a6887a6](https://github.com/vernonthedev/nestforge/commit/a6887a620a8f53ab54f16a0af5af6276cea8556d))
* feat: added named schedule jobs and a fluent registry builder api ([33b0440](https://github.com/vernonthedev/nestforge/commit/33b044025f05e6c049a244717ce25259594c449e))
* feat: added a builder based dynamic module api with typed exports and async registration ([d8d2a6e](https://github.com/vernonthedev/nestforge/commit/d8d2a6e7a95a8450d48c49c0e8f6886e946dbd44))
* feat: added explicit testing module shutdown hooks for lifecycle aware cleanup ([29fc650](https://github.com/vernonthedev/nestforge/commit/29fc650c7421ee0d710586a91bebc3bfa93721f5))
* feat: added http and graphql testing harness helpers on top of the module runtime ([1d2585a](https://github.com/vernonthedev/nestforge/commit/1d2585a0a8ba8b9a2ea807d77c0ad1392f3ce104))
* feat: added a websocket first example app and cli transport scaffolding ([d5284e0](https://github.com/vernonthedev/nestforge/commit/d5284e0683a954a3a6404534043d7fc248af1ba5))
* feat: added a nest style middleware consumer for route targeted http middleware ([54f4469](https://github.com/vernonthedev/nestforge/commit/54f446928eb68d1a4fbd3d46a4a067032b1f7ef6))
* feat: added a dedicated nestforge websockets crate with gateway mounting helpers ([be7bbcd](https://github.com/vernonthedev/nestforge/commit/be7bbcd52726698e632f79ffbe8450d29027237b))
* feat: added transient provider factories for short lived dependency resolution ([8000af0](https://github.com/vernonthedev/nestforge/commit/8000af09d4c954495057c129889cf54dd2d39a21))
* feat: added scheduler support with lifecycle driven startup and shutdown hooks ([1fa67cf](https://github.com/vernonthedev/nestforge/commit/1fa67cf1bef3f65bcc1fa1be5bdd2465fda4f6f6))
* feat: added dynamic module refs so imports can capture runtime registration options ([3a25c13](https://github.com/vernonthedev/nestforge/commit/3a25c13eb4dbdcd357b594344f2abb7eb9afa6e6))
* feat: added request scoped provider factories that resolve from per request containers ([5bfbfed](https://github.com/vernonthedev/nestforge/commit/5bfbfed8c468f0108bd6d7adba633d9a76f860a5))
* feat: added first class request pipes for params queries and json bodies ([07f2566](https://github.com/vernonthedev/nestforge/commit/07f2566d8831804dfdbcb39247f42b6c0bb70f1b))
* feat: added global exception filters to the nestforge request pipeline ([ef59d78](https://github.com/vernonthedev/nestforge/commit/ef59d789be6ecffa99c8f62c51d1aafa369941bb))
* feat: added nest style module lifecycle hooks across the framework runtime ([7d99fb3](https://github.com/vernonthedev/nestforge/commit/7d99fb3a1a08235af002878d47c49adf5ae1a039))
* feat: added graphql dependency resolution helpers on top of resolver context ([11659c5](https://github.com/vernonthedev/nestforge/commit/11659c5c90fedad5f72de2de1390915975e767b0))
* feat: injected the nestforge container into graphql resolver context ([4394ce0](https://github.com/vernonthedev/nestforge/commit/4394ce04f8a328838ad378400cdadd7f25b70171))
* feat: added graphql and grpc generators so transport apps can grow incrementally ([afcfca0](https://github.com/vernonthedev/nestforge/commit/afcfca0dab1031122af6fe9f52c8989188a0cc24))
* feat: scaffolded graphql and grpc app templates directly from the cli ([bb41708](https://github.com/vernonthedev/nestforge/commit/bb41708aa6ce1179de27a1d8baa7031f49dfdb05))
* feat: added a grpc first example app with tonic proto scaffolding ([23d5f4d](https://github.com/vernonthedev/nestforge/commit/23d5f4d5f0e95fa61d4263a6e852f42eb09db50e))
* feat: added a dedicated nestforge grpc crate with tonic based transport factory support ([93dc326](https://github.com/vernonthedev/nestforge/commit/93dc32682a4119f0b146d87afd60e1a020c7f82b))
* feat: added a graphql first example app wired through nestforge factory and schema helpers ([f8b0ce1](https://github.com/vernonthedev/nestforge/commit/f8b0ce1ff6616a004b1f57bb2f296d315a2aed7e))
* feat: added a dedicated nestforge graphql crate with graphiql and factory integration helpers ([c8702b0](https://github.com/vernonthedev/nestforge/commit/c8702b0935b1a325c48b95d417f13bb2646a8d9b))
* feat: added optional auth extraction helpers and openapi factory helpers with updated docs ([dfd2835](https://github.com/vernonthedev/nestforge/commit/dfd2835ee4b302791bb434e003d14a4f62241da0))
* feat: added auth aware guards, common http extractors, and direct openapi router mounting support ([590c721](https://github.com/vernonthedev/nestforge/commit/590c721820594f1c67cc5e545f006f3d8a5bbf30))
* feat: generated openapi documents from module route metadata and documented controller annotations ([2752520](https://github.com/vernonthedev/nestforge/commit/2752520187f24a6961313758f522ac65de8d1de0))
* feat: added auth identities, bearer token extraction, and request scoped auth resolution ([bf62e39](https://github.com/vernonthedev/nestforge/commit/bf62e39000f2e9546926d0e515726ebf6f48e756))
* feat: executed migration files as single transactional sql scripts instead of splitting statements ([ad9880c](https://github.com/vernonthedev/nestforge/commit/ad9880c559d480f12279d25676f8a12eaa6acf92))
* feat: expanded dto validation rules and added first class typed query extraction ([83ff24c](https://github.com/vernonthedev/nestforge/commit/83ff24c9580bbca57e6a79e284b558f2d7745522))
* feat: added container override layers so testing modules can override providers transitively ([f0d3f4d](https://github.com/vernonthedev/nestforge/commit/f0d3f4dd28604d3be61c2624fb6d2116ccbc3a21))
* feat: introduced request ids, structured framework logging, and standardised http error responses ([3ac30b2](https://github.com/vernonthedev/nestforge/commit/3ac30b2c49ee6b9b5104e19e321f8c9275dc3499))
* feat: introduce procedural macros for controllers and modules, add HTTP extension traits, and integrate in-memory MongoDB. ([685c242](https://github.com/vernonthedev/nestforge/commit/685c242674b57cbc24aa6bb9044eeb7413c2b4dc))
* feat(documentation): added  comprehensive documentation covering core concepts, module system, configuration, resource services, and a quick start guide, alongside an updated README. ([aabaacb](https://github.com/vernonthedev/nestforge/commit/aabaacb15756a881c22a1f841ed47f07c000ab2b))
* feat: added NestForge CLI commands for app scaffolding, code generation, database management, documentation, and formatting. ([eefcbe0](https://github.com/vernonthedev/nestforge/commit/eefcbe00ae77427ce7b66a7a36a8681f8355df52))
* feat: properly structured module & resource generation directory structure to a nestjs schema & a joi mock ([224e530](https://github.com/vernonthedev/nestforge/commit/224e5306c41e3059a05ab33161dd06b990d6240e))
* feat: added configuration module setups to handle envs ([4884589](https://github.com/vernonthedev/nestforge/commit/48845890a54575f3d02b622edffd18fb50f22af6))
* feat: added comprehensive api framework logging for all application requests ([955be3c](https://github.com/vernonthedev/nestforge/commit/955be3c4f8e4cfb13f1115aefec96b2e767096ac))
* feat: added guards, interceptors generation to the nestforge cli ([88c0670](https://github.com/vernonthedev/nestforge/commit/88c0670da5314e6174692a9bdbcda2ea0a220a61))
* feat: implemented lang prefixes, api versioning, interceptors & guards ([a3fad3f](https://github.com/vernonthedev/nestforge/commit/a3fad3f4935c14d0880669fd65557dbbf640bd15))
* feat: implemented `Param`, `Body`, `ValidatedBody` request extractors and `Inject` for dependency injection, demonstrated in the users controller. ([d58d0d9](https://github.com/vernonthedev/nestforge/commit/d58d0d966cd85baaa23ce042159d550f4fd092a0))
* feat: simplified the services to reduce usage of &self kind of setups ([90fdb50](https://github.com/vernonthedev/nestforge/commit/90fdb50724ec0e4bfd801dc59dc3fe7105d23eb7))
* feat: add `run_all_features.ps1` PowerShell script to automate comprehensive project testing and CLI command verification. ([b75d7cb](https://github.com/vernonthedev/nestforge/commit/b75d7cb1f1ffd640fc2a73c52f46097efe423b81))
* feat: generated the openai skeleton docs json file ([a44e715](https://github.com/vernonthedev/nestforge/commit/a44e715fb5bfe195561f0e5bfa009ccd14254740))
* feat: established the initial multi-crate NestForge framework structure, including core module system, HTTP server, CLI, and various integration crates. ([7fa06dc](https://github.com/vernonthedev/nestforge/commit/7fa06dc79be41f1522d4cb79b2231b4dad63330b))
* feat: added NestForge CLI db scaffolding for projec database management ([9aafb71](https://github.com/vernonthedev/nestforge/commit/9aafb719262a0438dce92b86775fa5afc6c60d4d))
* feat: setup NestForge CLI for application scaffolding, code generation, and database management. ([094281d](https://github.com/vernonthedev/nestforge/commit/094281ded58771458c6d3ae8b119a53bf7d187a7))
* feat: introduced nestforge framework with core, procedural macros for DI/routing/ORM, and an ORM module. ([9be65b3](https://github.com/vernonthedev/nestforge/commit/9be65b3ac8fef0be3eb5c6a51eb2d796cf2474d6))
* feat: introduced `nestforge-db` crate for database integration and demonstrate its usage in a new example. ([6b343ad](https://github.com/vernonthedev/nestforge/commit/6b343adf6a5315bb27c9ac22785be632dd6aa0a1))
* feat: added `nestforge-core` crate with request parameter, body, and input validation. ([a613f66](https://github.com/vernonthedev/nestforge/commit/a613f668e1a27ccc01d19c503c3f342c9b3caa22))
* feat: implemented core framework components including DI container, module system, and testing utilities. ([7f2e878](https://github.com/vernonthedev/nestforge/commit/7f2e878a2a54db962f6dab107b48be733304d0c7))
* feat: initialized the NestForge framework with core module and dependency injection provider systems ([95433fd](https://github.com/vernonthedev/nestforge/commit/95433fd95af30d6da0d333280f0097140e8f3538))
* feat: setup the core module system with graph initialization, dependency resolution, and macro support for module definition. ([85a3d9f](https://github.com/vernonthedev/nestforge/commit/85a3d9fe22bb773e752215e7c57ebe0f93bff5d0))
* feat(docs): created documentation wikis and ensured they are synced using a github action ([3f8add0](https://github.com/vernonthedev/nestforge/commit/3f8add0255805ab9cf499ea813bcd15b04dd8fc3))
* feat: implemented a new CLI for NestForge to scaffold applications and generate resources, controllers, and services. ([4dec36d](https://github.com/vernonthedev/nestforge/commit/4dec36d5fb7508d7e625abea4c266e4d958f45f2))
* feat: added the new macros setup to the example app and also simplified all the controller code setup ([eb04d6d](https://github.com/vernonthedev/nestforge/commit/eb04d6da80ce646ed433c78fa41b302135ba33b8))
* feat: setup the core framework components for dependency injection, modularity, and declarative routing with procedural macros. ([df092e9](https://github.com/vernonthedev/nestforge/commit/df092e9b9b85f148c6eb5de5491d2da93d455350))
* feat: introduce `nestforge-core` with core framework components and `nestforge` as the public API crate. ([7055c93](https://github.com/vernonthedev/nestforge/commit/7055c93d7c515a1ff60989d541645394c101a898))
* feat: introduce `Inject<T>` helper for dependency resolution from the container. ([0200c3d](https://github.com/vernonthedev/nestforge/commit/0200c3dae6543d7d4473c5c1371b0149b634be09))
* feat: added custom error handling that return proper json responses using `serde` ([faa9e80](https://github.com/vernonthedev/nestforge/commit/faa9e8008398caddcad866e2be21ab4271608d6d))
* feat: implemented the structured example app with nestjs dir structure ([15a73c0](https://github.com/vernonthedev/nestforge/commit/15a73c05c82f7f882869fef83a123d26e78ec725))
* feat: setup the core module and controller definitions, and an HTTP factory for application bootstrapping ([5bd9fa4](https://github.com/vernonthedev/nestforge/commit/5bd9fa430ace114a1e3a4040419c87c83d766580))
* feat: added `nestforge-core` crate with `ControllerDefinition` and `ModuleDefinition` traits. ([5a794fb](https://github.com/vernonthedev/nestforge/commit/5a794fbb477fcd7fa7ac219938965341f16f2ffb))
* feat: established a public API for `nestforge` crate by re-exporting core components. ([7671ad6](https://github.com/vernonthedev/nestforge/commit/7671ad6722cc9f2bf0c1057d5886c17ff59d7e1c))
* feat: setup the NestForgeFactory for HTTP server bootstrapping and module registration ([d08a235](https://github.com/vernonthedev/nestforge/commit/d08a2358aa8e2dc639c62862aa205b21c49968ca))
* feat: setup `ModuleDefinition` trait for registering services into the dependency injection container. ([b66eb30](https://github.com/vernonthedev/nestforge/commit/b66eb303e3803af2ee7f4dc967bc8b152971af9e))
* feat: initialized the Rust workspace with `nestforge`, `nestforge-core` lib setup ([00ed241](https://github.com/vernonthedev/nestforge/commit/00ed241fe41b5a336e5b0dd85d842cdf5af02da0))

### Fixes

* fix(tests): replace manual dto derive with response_dto ([ed90f2d](https://github.com/vernonthedev/nestforge/commit/ed90f2d508b539da7cf11576c051c2b30b1b6c42))
* fix(cli): update module generation for nested layout ([0da89b6](https://github.com/vernonthedev/nestforge/commit/0da89b6c25ddb399436f68455be0cc6446c085c7))
* fix(cli): register services as providers ([68fcd49](https://github.com/vernonthedev/nestforge/commit/68fcd496971052e8ca3e2c0c61308a0ae207981a))
* fix(core): added service module injection structure, fixes #19 ([9ce6f4f](https://github.com/vernonthedev/nestforge/commit/9ce6f4ff5979d3408c37f30920e0a8b082cffee5))
* fix(dependencies): updated Iru from 0.12.5 to 0.16.3 ([d9ebb8e](https://github.com/vernonthedev/nestforge/commit/d9ebb8e705f5ac309319cf9a6dd402b93033396a))
* fix(cli): fixed the new project scaffold to stop creating empty services folders, fixes #16 ([ee4d333](https://github.com/vernonthedev/nestforge/commit/ee4d33360e2282910521f48e790b4088b0afb1c5))
* fix(modules): fixed the bug report issue form so github can recognize it properly, fixes #17 ([a483abf](https://github.com/vernonthedev/nestforge/commit/a483abfa84732015603cdb8b9ef788a2b085dfdf))
* fix(cli): fixed the graphql starter dependency and macro imports ([a848483](https://github.com/vernonthedev/nestforge/commit/a848483aa65dfc3776f91c8ce8ed48f47ac9a389))
* fix(cli): fixed the graphql scaffold async-graphql imports ([71624b8](https://github.com/vernonthedev/nestforge/commit/71624b8fa725977d26758c0e1162ec81be2cd79c))
* fix(cli): keep new wizard transport stable ([eaf68a7](https://github.com/vernonthedev/nestforge/commit/eaf68a73fc2110c5d3efe50240b3583f1a2b4886))
* fix(cli): remove redundant tui titles ([ebff157](https://github.com/vernonthedev/nestforge/commit/ebff157af77c464c0a1be0d95feb9a662b068d04))
* fix(cli): repair microservices scaffold template ([0c2f3d9](https://github.com/vernonthedev/nestforge/commit/0c2f3d9850185abe3cd5537d24ab3dc74d0bd171))
* fix(openai): fixed the entire openapi docs generations ([e13304e](https://github.com/vernonthedev/nestforge/commit/e13304ee17bd666a00f0181eb63594270b768b9a))
* fix(openapi): use docs-relative spec urls ([4496b03](https://github.com/vernonthedev/nestforge/commit/4496b03858bbde7ac198400e3b3f6d2fdc2db8d1))
* fix(cli): use prompt wizard in git bash ([4f2928f](https://github.com/vernonthedev/nestforge/commit/4f2928f2a8cd6a36096be53d2f8192d1ac82d267))
* fix(cli): add numeric choices to generate wizard ([24f988d](https://github.com/vernonthedev/nestforge/commit/24f988dfe82edf8c7c8a4b0fa75ae8a6eb11eac2))
* fix(cli): use directional controls in generate wizard ([93b68ab](https://github.com/vernonthedev/nestforge/commit/93b68abdd4ba02a787b9759a64c564b12d7694e4))
* fix(cli): simplify ratatui field rendering ([9263823](https://github.com/vernonthedev/nestforge/commit/926382319a1bace2fd856f244f4ff710d48a4c79))
* fix(cli): clarify ratatui selectors and toggles ([453936a](https://github.com/vernonthedev/nestforge/commit/453936a0d6f4b47dfb1be054e6a28e55ea6c5d32))
* fix(cli): improve ratatui field entry and prompt fallback ([8c1e471](https://github.com/vernonthedev/nestforge/commit/8c1e4719f64920ab9d555c7cb3e8a049603843fc))
* fix(openapi): support serialized response schemas ([1d13544](https://github.com/vernonthedev/nestforge/commit/1d1354481b8e72de0cc42202b7c228b0c003dcc4))
* fix(security): added bounded GraphQL requests without trusting thirdparties, fixed #2 ([e333c48](https://github.com/vernonthedev/nestforge/commit/e333c4825bd7776a7831aa394df17aa8668d7cca))
* fix: fixed the proc error on project creation, fixes #5 ([3ce4542](https://github.com/vernonthedev/nestforge/commit/3ce454218c450f5ea7d0e00191309c1017431eea))
* fix: removed all unused vars ([e8bd504](https://github.com/vernonthedev/nestforge/commit/e8bd504fe8f1c8376bb6416d631fa76127d5f56a))
* fix: retry crate publishes after crates io rate limits ([80ccdf8](https://github.com/vernonthedev/nestforge/commit/80ccdf8ff63100f9c9b74b89aaee6986a2b2a267))
* fix: publish microservice crates before grpc dependencies ([624e70c](https://github.com/vernonthedev/nestforge/commit/624e70cfa46bfe4cea646bac6f13b138b3876964))
* fix: remove protoc as a build requirement from the grpc example app ([9e92501](https://github.com/vernonthedev/nestforge/commit/9e9250162c50d031c9dbe0eacaa9a4cdfba36c3b))
* fix: remove leftover warning-only code from the example apps ([e815773](https://github.com/vernonthedev/nestforge/commit/e81577364ece6333b85fc6b6caa328c7407f3ff6))
* fix: make local workspace checks reliable without sqlite toolchain blockers ([00206a4](https://github.com/vernonthedev/nestforge/commit/00206a4982f68f967b8278848df538cd2e2cad57))
* fix: return an owned app name from the graphql example resolver ([f3445d4](https://github.com/vernonthedev/nestforge/commit/f3445d437e2701076c62ee4258970fe6bc95ce91))
* fix: add missing grpc example dependencies for module and json usage ([2e7f182](https://github.com/vernonthedev/nestforge/commit/2e7f182087a02531ae5bff0156ee2512a0fa2bba))
* fix: add missing example dependencies for graphql and websocket builds ([6eee612](https://github.com/vernonthedev/nestforge/commit/6eee612e34ec3c637c0fbfb069aab3b7d5f6500f))
* fix: resolve example app compile errors uncovered by the release flow ([734e15a](https://github.com/vernonthedev/nestforge/commit/734e15aff14fe4f7743321bf9de4622d0c0e1bfe))
* fix: make axum a runtime dependency of the nestforge crate ([77526fd](https://github.com/vernonthedev/nestforge/commit/77526fd893b5ab1f1eb7fa2ee984b8e8ad95749b))
* fix: use a safe regex evaluator for workspace version bumps ([684e682](https://github.com/vernonthedev/nestforge/commit/684e682eac62f831489a3198369377f6655cba79))
* fix: avoid uninitialized lastexitcode checks in the release script ([5e1a5c3](https://github.com/vernonthedev/nestforge/commit/5e1a5c36d9bc8d449510f489017d4457a73a0a92))
* fix: add package readmes for release and crates io metadata ([4cbbbf5](https://github.com/vernonthedev/nestforge/commit/4cbbbf56619240673f81712f3b7f54e52ccb7d5f))
* fix: point release plz changelog updates at the nestforge crate ([69247da](https://github.com/vernonthedev/nestforge/commit/69247da5197a680eb753358500c32dee22a536e2))
* fix: resolve framework compile errors across graphql http cache and websockets ([cdf9d30](https://github.com/vernonthedev/nestforge/commit/cdf9d30544afa21c8a988b69ded9d7ed348ae25a))
* fix: propagated scoped request context into graphql resolvers and helper accessors ([259aa2a](https://github.com/vernonthedev/nestforge/commit/259aa2a72cf059ba0a8dce4e90a6bd8dfac6b77e))
* fix: enforced authenticated and role based route metadata at runtime and in openapi output ([de7d618](https://github.com/vernonthedev/nestforge/commit/de7d6180b62a04da116ad882b4ecef9b1e9ff090))

### Other

* chore: merge pull request #22 from vernonthedev/develop ([6112453](https://github.com/vernonthedev/nestforge/commit/6112453fa8c0e0c629d111d88b1744bd677af0c6))
* docs: updated docs with the newly worked on features for injectable service structure & cli changes ([d9e4975](https://github.com/vernonthedev/nestforge/commit/d9e4975867b77bfddfa1cc03b0d15ff137701026))
* docs(cli): clarify OpenAPI docs and prelude guidance ([7deaa7c](https://github.com/vernonthedev/nestforge/commit/7deaa7cd33726f7e9cd4a10297bc88c1cbd8439e))
* chore(cli): drop redundant tui reliability notes ([bfd7838](https://github.com/vernonthedev/nestforge/commit/bfd7838bf7c2d517f9b891d75bf7b94bc0b80560))
* chore(vendor): add ratatui vendor tree and adjust lockfile ([6376f09](https://github.com/vernonthedev/nestforge/commit/6376f09bed6f90488a5b4a6546e9ea75e419a2d8))
* chore: simplified the service case study usage from derifs ([c147356](https://github.com/vernonthedev/nestforge/commit/c1473561d90bd69804562f23b61b8d718d40059d))
* docs(readme): document injectable services ([5554041](https://github.com/vernonthedev/nestforge/commit/5554041c84c72e0b7f1e733c9c4f4bb2cd55e22b))
* docs: removed raw rendered md files ([f7fc9c9](https://github.com/vernonthedev/nestforge/commit/f7fc9c9bdb43c2350d80220ab4ec95989cc266fd))
* docs: did a little docs cleanup with the new shields ([9c69a1e](https://github.com/vernonthedev/nestforge/commit/9c69a1e691918b337d23e6834f3e9731ff9313b9))
* docs(readme): added framework badges and the vscode extension section ([06a5510](https://github.com/vernonthedev/nestforge/commit/06a5510727cc70680b0b3a629f4a48ee74c79488))
* docs(community): added the missing repository health files and contribution automation, fixes #15 ([94a7598](https://github.com/vernonthedev/nestforge/commit/94a75987c62a936bc928b33c4cd9092cfcbcc438))
* chore: added dev exec example dir ([b4541c6](https://github.com/vernonthedev/nestforge/commit/b4541c6aad6b5159a84e6b8fa5e7f2af3b22b4bf))
* refactor(cli): split parser ui and diagnostics modules ([68522ba](https://github.com/vernonthedev/nestforge/commit/68522ba1619eb2c6af39261b987744793774f5f4))
* refactor(dtos): updated cli to handle prompts for dto data structures ([eb6d7db](https://github.com/vernonthedev/nestforge/commit/eb6d7dbf8cc0ecb3c5f41525f20b8e1e5aa22ddf))
* docs: added flat layout structure ([410de8d](https://github.com/vernonthedev/nestforge/commit/410de8d61d7927ed328ee262f745ddd7ab8f3769))
* docs: updated application documentations ([53ae54b](https://github.com/vernonthedev/nestforge/commit/53ae54bd04eb7e27a07cec9e5cbf199acdeeb324))
* chore: application fixes for the primitive bugs ([2602290](https://github.com/vernonthedev/nestforge/commit/2602290d0f5c742f8a1574fa3b3e8d8fdd864e99))
* docs: updated version number to 1.2.0 in README ([543abfc](https://github.com/vernonthedev/nestforge/commit/543abfcef53e7fb96fc25412c3005ccd361a2ac7))
* chore: update Rust dependencies. ([77739b8](https://github.com/vernonthedev/nestforge/commit/77739b8b144ca6494f663b96b6fa8687b7aa540b))
* chore: Update Rust dependencies ([0c3eda9](https://github.com/vernonthedev/nestforge/commit/0c3eda93c8199464cc567d755cc78fef2d2f225b))
* chore: merged pull request #4 from vernonthedev/feature/auto-documentation ([d4f0ba7](https://github.com/vernonthedev/nestforge/commit/d4f0ba753311c61289e1a6f0ac82f85fe5d6011a))
* chore: merged pull request #3 from vernonthedev/feature/auto-documentation ([7d4aa1c](https://github.com/vernonthedev/nestforge/commit/7d4aa1c5ef369769255d2cf8df375a068ef26fd0))
* chore: merged pull request #2 from vernonthedev/feature/auto-documentation ([dbf0905](https://github.com/vernonthedev/nestforge/commit/dbf09059d07bf6004fa90d0f2294446081ac98d2))
* docs: updated quick start flows for graphql and grpc app scaffolding ([d88b20f](https://github.com/vernonthedev/nestforge/commit/d88b20f4461d2214ea4ab5177bad3620d9a45d5e))
* chore: removed unnecessary commit file that popped up during development ([6c02a67](https://github.com/vernonthedev/nestforge/commit/6c02a6754e10079662b50ee4209102a755ab0635))
* chore: modularize NestForge into new crates, bump workspace version to 1.1.0, and add a release publishing script ([6c4ef46](https://github.com/vernonthedev/nestforge/commit/6c4ef463ab253dc253d4f65338eb51017f513e04))
* chore: merge pull request #1 from vernonthedev/develop ([a83d69a](https://github.com/vernonthedev/nestforge/commit/a83d69a6bd01d970838d5a64c042501b788dcec0))
* refactor: cleaned the appconfig module setup by using vec in main app mods ([493a789](https://github.com/vernonthedev/nestforge/commit/493a7897aef82cc873bdd84f1bc1be98369110d9))
* refactor: reorganised the entire example app structure to a cleaner nestjs mimic ([97734b8](https://github.com/vernonthedev/nestforge/commit/97734b8452adae793781c0c2e76b332f80adee23))
* refactor: reorganised the file structure ([48cf560](https://github.com/vernonthedev/nestforge/commit/48cf56030c7ffda7142a75139065041906fdb812))
* refactor: created a new settings module to test out the config module ([5d3a19e](https://github.com/vernonthedev/nestforge/commit/5d3a19eb76d932c9c29b64ca129978f23b5204fc))
* refactor: added nestjs like versioning controller implementation ([22920a4](https://github.com/vernonthedev/nestforge/commit/22920a4dc82740af62144be3fc74cd58262a55ff))
* chore: added custom guard usage to the example app ([0e4aa12](https://github.com/vernonthedev/nestforge/commit/0e4aa123a697a97d2636c3ba9013d325bb5501d4))
* chore: simplified the guards & interceptors ([2817606](https://github.com/vernonthedev/nestforge/commit/28176068032df2e6f46b685346f2c6b506484165))
* refactor: simplified main app modules and main bootstrap namings ([bfbc288](https://github.com/vernonthedev/nestforge/commit/bfbc28833e1cc0cb7b1fdda26316498f082c813a))
* refactor: simplified the services business logic ([b661e6a](https://github.com/vernonthedev/nestforge/commit/b661e6a392db44c83703396ae6f0dc896d5d55ca))
* refactor: simplified the controller error handlings to more readable code ([7a874e8](https://github.com/vernonthedev/nestforge/commit/7a874e88e93031774101b380bba9615c0817a677))
* refactor: simplifed the api return listing data types ([fcca8a2](https://github.com/vernonthedev/nestforge/commit/fcca8a2085ecdd6f663489ac48bf3245ed73566c))
* refactor: cleaned and reduced dto complexity by replacing the complex dto structure with macros ([a18aeef](https://github.com/vernonthedev/nestforge/commit/a18aeefde834b8ab7d47c01884a36605a8709448))
* refactor: fixed the remaining testing bugs ([02caa3a](https://github.com/vernonthedev/nestforge/commit/02caa3ad92528cf863078680c6077edf88f400f7))
* docs: added project governance files including contribution guidelines and license, and update gitignore. ([111fd20](https://github.com/vernonthedev/nestforge/commit/111fd208bef2c561113274c2be35aa480b23fa8c))
* chore(fix): patched a checkout of untrusted code in trusted context ([6cafa94](https://github.com/vernonthedev/nestforge/commit/6cafa94fddb71a62086231a36c86bab61ab5f704))
* docs(release): updated docs to reflect a newly updated version of the application to `crate.io` ([87709d1](https://github.com/vernonthedev/nestforge/commit/87709d1d7a357d1cd0fff3c83148b5cb6c07fa87))
* docs: cleaned and removed app footer unused line ([17d0d29](https://github.com/vernonthedev/nestforge/commit/17d0d29fd389cde4f9707b0b560af02b7de2548a))
* docs: updated file routes causing add new pages as we do a wiki sync ([0f4eae7](https://github.com/vernonthedev/nestforge/commit/0f4eae74257c8fa06a341af40cb5f0e2567b18d8))
* docs: added comprehensive project structure, API, core concepts, macros, example app, file map, quick start, and CLI documentation ([897a3af](https://github.com/vernonthedev/nestforge/commit/897a3aff2f91a8ce12fe9040bb5e86143d26811e))
* docs: cleaned up the docs ([265fe83](https://github.com/vernonthedev/nestforge/commit/265fe836edcef322be843fb9aea288d4397844cf))
* docs: updated the README content to add the application nestforge cli setup commands ([ca14be7](https://github.com/vernonthedev/nestforge/commit/ca14be7b0fe7daabe1fb1d0d7da22ee1a96d0297))
* refactor(fix): proper fixed the app url handling for the apps generated using the nestforge cli ([8c82959](https://github.com/vernonthedev/nestforge/commit/8c82959263826e72356c3b66a709e1d2114a2d4c))
* refactor(fix): fixed a main module import implementation bug ([ed209ca](https://github.com/vernonthedev/nestforge/commit/ed209ca0f5cde230546e1e2d1d5ba3163ac3adbb))
* chore: added a stand alone app based workspace for the cli generated apps to fix the pkg workspace snug ([a2159e6](https://github.com/vernonthedev/nestforge/commit/a2159e6eb8f65dc23de383d240a67870da646334))
* refactor: cleaned the app module, & fixed the module macros engine ([65d83ae](https://github.com/vernonthedev/nestforge/commit/65d83ae9f59a5cedcad5dfa8436590db19c01cbc))
* refactor: added a new module abstraction to core framework ([ba20fcc](https://github.com/vernonthedev/nestforge/commit/ba20fcc4077014b47d5d0a654233788830f2dbd9))
* refactor: added the new mem store implementation to the example users app ([e48e6bb](https://github.com/vernonthedev/nestforge/commit/e48e6bbf9cedd288c26889c04eae2347999d8ef0))
* refactor: created an in memory store to store the ARlocks setup for cleaner service implementations in user applications ([9259988](https://github.com/vernonthedev/nestforge/commit/9259988ce6015f2aaa1334b83a2acc2676b2ffcc))
* chore: cleaned user controller removing unused imports ([83fe398](https://github.com/vernonthedev/nestforge/commit/83fe398350486b6b43ffee069480db84da7377ed))
* chore: added user updating to the example application ([fd046c7](https://github.com/vernonthedev/nestforge/commit/fd046c7bac38efe149b4065d5121bdabe040e883))
* chore: updated and added post requests to example application ([59abb6e](https://github.com/vernonthedev/nestforge/commit/59abb6e756a026208067aa8d1bd82144079a189e))
* refactor: updated the controllers to use the newly implemented error handling from nestforge core ([2ee9a28](https://github.com/vernonthedev/nestforge/commit/2ee9a285f5592ed3f154b94b05aae3a608b9729a))
* chore: added serge to test out service & controller implementations in the example app ([a851676](https://github.com/vernonthedev/nestforge/commit/a85167629ea31b62a3aee29f199962ed17eec1ee))
* chore: added funny emoji to rep nestforge ([1eabb5a](https://github.com/vernonthedev/nestforge/commit/1eabb5aec5a0aaf9c2feb0aefc64b3d7336aff86))
* docs: added a basic README file ([6be1a36](https://github.com/vernonthedev/nestforge/commit/6be1a3610c0048e1a92dfb5d571c75780f90f1b4))
* chore: initialize `nestforge-core`, `nestforge-http`, and `nestforge` crates with their respective dependencies. ([893e52a](https://github.com/vernonthedev/nestforge/commit/893e52aa70390944ccf6959d92c412b0a8188f1e))
* chore: initial commit ([510e4fd](https://github.com/vernonthedev/nestforge/commit/510e4fd91d3df0ccf281df883c2132babbce3622))



## [1.7.0](https://github.com/vernonthedev/nestforge/compare/v1.6.0...v1.7.0) (2026-03-24)

### Features

* feat(config): add centralized configuration system with type-safe, validated environment loading ([4ab1cf9](https://github.com/vernonthedev/nestforge/commit/4ab1cf91519c4960b4dc02c78c250dc1f3403b37))

### Other

* chore: merged pull request #26 from vernonthedev/feat/nestforge-config-module ([cbc0726](https://github.com/vernonthedev/nestforge/commit/cbc07267951b7f9686f4599d17bd751862008d7d))
* refactor(config): simplify config API with typed getters and EnvStore ([56d0f39](https://github.com/vernonthedev/nestforge/commit/56d0f39ddd1539401f1407881b333ffd27b07c46))
* refactor(config): add typed getters and ConfigRegistration, simplify config loading ([6acd372](https://github.com/vernonthedev/nestforge/commit/6acd372994e1cdd76baef283fa3532f1655a8f5c))
* refactor(config): simplify config API by removing Arc and adding typed getters ([9d70ef5](https://github.com/vernonthedev/nestforge/commit/9d70ef5d78b18b4a1a9d8a5d8e5900e1d9838170))
* refactor(config): simplify configuration system by removing trait-based approach ([e7b3aaf](https://github.com/vernonthedev/nestforge/commit/e7b3aafc3493987f2db175c3282b7a77877a87fc))
* refactor(config): simplify configuration system by removing validation and auto-deriving, fixes #23 ([9c0f614](https://github.com/vernonthedev/nestforge/commit/9c0f61405819f2772852fe01610e7278bd43e96e))
* docs: added stable 1.6.0 version update ([cffafa5](https://github.com/vernonthedev/nestforge/commit/cffafa5e52b9b1799357865d7edfc02257797aed))
* docs: updated FUNDING.yml links ([dbf8812](https://github.com/vernonthedev/nestforge/commit/dbf8812909e2ead6b1aa0209c15d9f44709608a7))



## [1.8.0](https://github.com/vernonthedev/nestforge/compare/v1.6.0...v1.8.0) (2026-03-24)

### Features

* feat(config): add centralized configuration system with type-safe, validated environment loading ([4ab1cf9](https://github.com/vernonthedev/nestforge/commit/4ab1cf91519c4960b4dc02c78c250dc1f3403b37))

### Other

* chore: merged pull request #26 from vernonthedev/feat/nestforge-config-module ([cbc0726](https://github.com/vernonthedev/nestforge/commit/cbc07267951b7f9686f4599d17bd751862008d7d))
* refactor(config): simplify config API with typed getters and EnvStore ([56d0f39](https://github.com/vernonthedev/nestforge/commit/56d0f39ddd1539401f1407881b333ffd27b07c46))
* refactor(config): add typed getters and ConfigRegistration, simplify config loading ([6acd372](https://github.com/vernonthedev/nestforge/commit/6acd372994e1cdd76baef283fa3532f1655a8f5c))
* refactor(config): simplify config API by removing Arc and adding typed getters ([9d70ef5](https://github.com/vernonthedev/nestforge/commit/9d70ef5d78b18b4a1a9d8a5d8e5900e1d9838170))
* refactor(config): simplify configuration system by removing trait-based approach ([e7b3aaf](https://github.com/vernonthedev/nestforge/commit/e7b3aafc3493987f2db175c3282b7a77877a87fc))
* refactor(config): simplify configuration system by removing validation and auto-deriving, fixes #23 ([9c0f614](https://github.com/vernonthedev/nestforge/commit/9c0f61405819f2772852fe01610e7278bd43e96e))
* docs: added stable 1.6.0 version update ([cffafa5](https://github.com/vernonthedev/nestforge/commit/cffafa5e52b9b1799357865d7edfc02257797aed))
* docs: updated FUNDING.yml links ([dbf8812](https://github.com/vernonthedev/nestforge/commit/dbf8812909e2ead6b1aa0209c15d9f44709608a7))

