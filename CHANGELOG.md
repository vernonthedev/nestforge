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
