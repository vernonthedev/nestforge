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
