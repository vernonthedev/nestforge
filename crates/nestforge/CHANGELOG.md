# Changelog

All notable changes to the `nestforge` crate are documented in this file.

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

## [1.2.1](https://github.com/vernonthedev/nestforge/compare/v1.2.0...v1.2.1) (2026-03-06)

### Fixes

* fix: removed all unused vars ([e8bd504](https://github.com/vernonthedev/nestforge/commit/e8bd504fe8f1c8376bb6416d631fa76127d5f56a))

### Other

* docs: updated application documentations ([53ae54b](https://github.com/vernonthedev/nestforge/commit/53ae54bd04eb7e27a07cec9e5cbf199acdeeb324))
* chore: application fixes for the primitive bugs ([2602290](https://github.com/vernonthedev/nestforge/commit/2602290d0f5c742f8a1574fa3b3e8d8fdd864e99))
* docs: updated version number to 1.2.0 in README ([543abfc](https://github.com/vernonthedev/nestforge/commit/543abfcef53e7fb96fc25412c3005ccd361a2ac7))

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

