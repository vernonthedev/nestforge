use anyhow::Result;
use axum::Router;
use std::{collections::HashSet, future::Future, sync::Arc};

use crate::{
    framework_log_event, Container, DocumentedController, RegisterProvider, RouteDocumentation,
};

pub type LifecycleHook = fn(&Container) -> Result<()>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleGraphEntry {
    pub name: &'static str,
    pub imports: Vec<&'static str>,
    pub exports: Vec<&'static str>,
    pub controller_count: usize,
    pub is_global: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ModuleGraphReport {
    pub modules: Vec<ModuleGraphEntry>,
}

impl ModuleGraphReport {
    pub fn module(&self, name: &str) -> Option<&ModuleGraphEntry> {
        self.modules.iter().find(|module| module.name == name)
    }
}

/// Metadata implemented by the `#[controller("/path")]` attribute macro.
/// It defines the base path where all routes within the controller will live.
pub trait ControllerBasePath {
    fn base_path() -> &'static str;
}

/**
 * A `ControllerDefinition` provides the structure for NestForge to build an Axum router.
 *
 * The `#[routes]` macro automatically generates implementations of this trait.
 * It handles wiring up handler functions with their required guards, interceptors,
 * and pipes.
 *
 * Custom routers can implement this trait directly if needed, though the macro
 * approach is typically sufficient for most use cases.
 */
pub trait ControllerDefinition: Send + Sync + 'static {
    /// Builds the Axum router for this controller, using the framework container
    /// for its internal state.
    fn router() -> Router<Container>;
}

#[derive(Clone)]
pub struct ModuleRef {
    pub name: &'static str,
    pub register: Arc<dyn Fn(&Container) -> Result<()> + Send + Sync>,
    pub controllers: Arc<dyn Fn() -> Vec<Router<Container>> + Send + Sync>,
    pub route_docs: Arc<dyn Fn() -> Vec<RouteDocumentation> + Send + Sync>,
    pub on_module_init: Arc<dyn Fn() -> Vec<LifecycleHook> + Send + Sync>,
    pub on_module_destroy: Arc<dyn Fn() -> Vec<LifecycleHook> + Send + Sync>,
    pub on_application_bootstrap: Arc<dyn Fn() -> Vec<LifecycleHook> + Send + Sync>,
    pub on_application_shutdown: Arc<dyn Fn() -> Vec<LifecycleHook> + Send + Sync>,
    pub imports: Arc<dyn Fn() -> Vec<ModuleRef> + Send + Sync>,
    pub exports: Arc<dyn Fn() -> Vec<&'static str> + Send + Sync>,
    pub is_global: Arc<dyn Fn() -> bool + Send + Sync>,
}

impl ModuleRef {
    pub fn of<M: ModuleDefinition>() -> Self {
        Self {
            name: M::module_name(),
            register: Arc::new(M::register),
            controllers: Arc::new(M::controllers),
            route_docs: Arc::new(M::route_docs),
            on_module_init: Arc::new(M::on_module_init),
            on_module_destroy: Arc::new(M::on_module_destroy),
            on_application_bootstrap: Arc::new(M::on_application_bootstrap),
            on_application_shutdown: Arc::new(M::on_application_shutdown),
            imports: Arc::new(M::imports),
            exports: Arc::new(M::exports),
            is_global: Arc::new(M::is_global),
        }
    }

    pub fn dynamic(
        name: &'static str,
        register: impl Fn(&Container) -> Result<()> + Send + Sync + 'static,
    ) -> Self {
        Self {
            name,
            register: Arc::new(register),
            controllers: Arc::new(Vec::new),
            route_docs: Arc::new(Vec::new),
            on_module_init: Arc::new(Vec::new),
            on_module_destroy: Arc::new(Vec::new),
            on_application_bootstrap: Arc::new(Vec::new),
            on_application_shutdown: Arc::new(Vec::new),
            imports: Arc::new(Vec::new),
            exports: Arc::new(Vec::new),
            is_global: Arc::new(|| false),
        }
    }

    pub fn builder(name: &'static str) -> DynamicModuleBuilder {
        DynamicModuleBuilder::new(name)
    }

    pub fn with_imports(
        mut self,
        imports: impl Fn() -> Vec<ModuleRef> + Send + Sync + 'static,
    ) -> Self {
        self.imports = Arc::new(imports);
        self
    }

    pub fn with_exports(
        mut self,
        exports: impl Fn() -> Vec<&'static str> + Send + Sync + 'static,
    ) -> Self {
        self.exports = Arc::new(exports);
        self
    }

    pub fn with_controllers(
        mut self,
        controllers: impl Fn() -> Vec<Router<Container>> + Send + Sync + 'static,
    ) -> Self {
        self.controllers = Arc::new(controllers);
        self
    }

    pub fn with_route_docs(
        mut self,
        route_docs: impl Fn() -> Vec<RouteDocumentation> + Send + Sync + 'static,
    ) -> Self {
        self.route_docs = Arc::new(route_docs);
        self
    }

    pub fn with_module_init_hooks(
        mut self,
        hooks: impl Fn() -> Vec<LifecycleHook> + Send + Sync + 'static,
    ) -> Self {
        self.on_module_init = Arc::new(hooks);
        self
    }

    pub fn with_module_destroy_hooks(
        mut self,
        hooks: impl Fn() -> Vec<LifecycleHook> + Send + Sync + 'static,
    ) -> Self {
        self.on_module_destroy = Arc::new(hooks);
        self
    }

    pub fn with_application_bootstrap_hooks(
        mut self,
        hooks: impl Fn() -> Vec<LifecycleHook> + Send + Sync + 'static,
    ) -> Self {
        self.on_application_bootstrap = Arc::new(hooks);
        self
    }

    pub fn with_application_shutdown_hooks(
        mut self,
        hooks: impl Fn() -> Vec<LifecycleHook> + Send + Sync + 'static,
    ) -> Self {
        self.on_application_shutdown = Arc::new(hooks);
        self
    }

    pub fn as_global(mut self) -> Self {
        self.is_global = Arc::new(|| true);
        self
    }

    pub fn with_is_global(mut self, is_global: bool) -> Self {
        self.is_global = Arc::new(move || is_global);
        self
    }
}

type RegistrationStep = Arc<dyn Fn(&Container) -> Result<()> + Send + Sync>;

pub struct DynamicModuleBuilder {
    name: &'static str,
    registration_steps: Vec<RegistrationStep>,
    imports: Vec<ModuleRef>,
    exports: Vec<&'static str>,
    controllers: Vec<Router<Container>>,
    route_docs: Vec<RouteDocumentation>,
    on_module_init: Vec<LifecycleHook>,
    on_module_destroy: Vec<LifecycleHook>,
    on_application_bootstrap: Vec<LifecycleHook>,
    on_application_shutdown: Vec<LifecycleHook>,
    is_global: bool,
}

impl DynamicModuleBuilder {
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            registration_steps: Vec::new(),
            imports: Vec::new(),
            exports: Vec::new(),
            controllers: Vec::new(),
            route_docs: Vec::new(),
            on_module_init: Vec::new(),
            on_module_destroy: Vec::new(),
            on_application_bootstrap: Vec::new(),
            on_application_shutdown: Vec::new(),
            is_global: false,
        }
    }

    pub fn register(
        mut self,
        register: impl Fn(&Container) -> Result<()> + Send + Sync + 'static,
    ) -> Self {
        self.registration_steps.push(Arc::new(register));
        self
    }

    pub fn register_provider<P>(self, provider: P) -> Self
    where
        P: RegisterProvider + Send + 'static,
    {
        let provider = Arc::new(std::sync::Mutex::new(Some(provider)));
        self.register(move |container| {
            let provider = provider
                .lock()
                .map_err(|_| anyhow::anyhow!("Dynamic module provider lock poisoned"))?
                .take()
                .ok_or_else(|| anyhow::anyhow!("Dynamic module provider already registered"))?;
            provider.register(container)
        })
    }

    pub fn provider_value<T>(self, value: T) -> Self
    where
        T: Clone + Send + Sync + 'static,
    {
        self.register(move |container| {
            container.register(value.clone())?;
            Ok(())
        })
    }

    pub fn provider_factory<T, F>(self, factory: F) -> Self
    where
        T: Send + Sync + 'static,
        F: Fn(&Container) -> Result<T> + Send + Sync + 'static,
    {
        self.register(move |container| {
            container.register(factory(container)?)?;
            Ok(())
        })
    }

    pub fn provider_async<T, F, Fut>(self, factory: F) -> Self
    where
        T: Send + Sync + 'static,
        F: Fn(Container) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<T>> + Send + 'static,
    {
        self.register(move |container| {
            let value = block_on_dynamic_registration(factory(container.clone()))?;
            container.register(value)?;
            Ok(())
        })
    }

    pub fn import(mut self, module: ModuleRef) -> Self {
        self.imports.push(module);
        self
    }

    pub fn export<T>(mut self) -> Self
    where
        T: 'static,
    {
        self.exports.push(std::any::type_name::<T>());
        self
    }

    pub fn controller<C>(mut self) -> Self
    where
        C: ControllerDefinition + DocumentedController,
    {
        self.controllers.push(C::router());
        self.route_docs.extend(C::route_docs());
        self
    }

    pub fn on_module_init(mut self, hook: LifecycleHook) -> Self {
        self.on_module_init.push(hook);
        self
    }

    pub fn on_module_destroy(mut self, hook: LifecycleHook) -> Self {
        self.on_module_destroy.push(hook);
        self
    }

    pub fn on_application_bootstrap(mut self, hook: LifecycleHook) -> Self {
        self.on_application_bootstrap.push(hook);
        self
    }

    pub fn on_application_shutdown(mut self, hook: LifecycleHook) -> Self {
        self.on_application_shutdown.push(hook);
        self
    }

    pub fn global(mut self) -> Self {
        self.is_global = true;
        self
    }

    pub fn build(self) -> ModuleRef {
        let registration_steps = Arc::new(self.registration_steps);
        let imports = self.imports;
        let exports = self.exports;
        let controllers = self.controllers;
        let route_docs = self.route_docs;
        let on_module_init = self.on_module_init;
        let on_module_destroy = self.on_module_destroy;
        let on_application_bootstrap = self.on_application_bootstrap;
        let on_application_shutdown = self.on_application_shutdown;
        let is_global = self.is_global;

        ModuleRef::dynamic(self.name, move |container| {
            for step in registration_steps.iter() {
                step(container)?;
            }
            Ok(())
        })
        .with_imports(move || imports.clone())
        .with_exports(move || exports.clone())
        .with_controllers(move || controllers.clone())
        .with_route_docs(move || route_docs.clone())
        .with_module_init_hooks(move || on_module_init.clone())
        .with_module_destroy_hooks(move || on_module_destroy.clone())
        .with_application_bootstrap_hooks(move || on_application_bootstrap.clone())
        .with_application_shutdown_hooks(move || on_application_shutdown.clone())
        .with_is_global(is_global)
    }
}

/**
 * `ModuleDefinition` is the contract for defining a NestForge module.
 *
 * Modules are the building blocks of a NestForge application. They group related
 * controllers and providers together, making the codebase easier to manage and
 * reason about.
 *
 * The `#[module]` macro automatically generates implementations of this trait.
 * Manual implementation is possible for more control over the module setup.
 *
 * ### Manual Implementation Example
 * ```rust
 * use nestforge::{ModuleDefinition, Container, ModuleRef};
 * use anyhow::Result;
 *
 * struct MyModule;
 *
 * impl ModuleDefinition for MyModule {
 *     fn register(container: &Container) -> Result<()> {
 *         // Register providers here
 *         Ok(())
 *     }
 *
 *     fn imports() -> Vec<ModuleRef> {
 *         // List dependent modules
 *         vec![]
 *     }
 * }
 * ```
 */
pub trait ModuleDefinition: Send + Sync + 'static {
    /// The name of the module, used for debugging and error messages.
    /// By default, it uses the type name of the struct.
    fn module_name() -> &'static str {
        std::any::type_name::<Self>()
    }

    /// Where the magic happens: register your module's providers into the container.
    fn register(container: &Container) -> Result<()>;

    /// List any other modules that this module depends on.
    /// NestForge will make sure they're initialized first.
    fn imports() -> Vec<ModuleRef> {
        Vec::new()
    }

    /// If true, the providers in this module will be available globally.
    fn is_global() -> bool {
        false
    }

    /// List the providers that this module makes available to other modules
    /// that import it.
    fn exports() -> Vec<&'static str> {
        Vec::new()
    }

    /// Returns the Axum routers for the controllers defined in this module.
    fn controllers() -> Vec<Router<Container>> {
        Vec::new()
    }

    /// Returns the documentation for the routes defined in this module.
    fn route_docs() -> Vec<RouteDocumentation> {
        Vec::new()
    }

    /// A hook that runs after the module has been initialized.
    fn on_module_init() -> Vec<LifecycleHook> {
        Vec::new()
    }

    /// A hook that runs when the module is being destroyed.
    fn on_module_destroy() -> Vec<LifecycleHook> {
        Vec::new()
    }

    /// A hook that runs after the entire application has been bootstrapped.
    fn on_application_bootstrap() -> Vec<LifecycleHook> {
        Vec::new()
    }

    /// A hook that runs when the application is shutting down.
    fn on_application_shutdown() -> Vec<LifecycleHook> {
        Vec::new()
    }
}

pub struct InitializedModule {
    pub controllers: Vec<Router<Container>>,
    module_init_hooks: Vec<LifecycleHook>,
    module_destroy_hooks: Vec<LifecycleHook>,
    application_bootstrap_hooks: Vec<LifecycleHook>,
    application_shutdown_hooks: Vec<LifecycleHook>,
}

impl InitializedModule {
    pub fn run_module_init(&self, container: &Container) -> Result<()> {
        run_lifecycle_hooks("module_init", &self.module_init_hooks, container)
    }

    pub fn run_module_destroy(&self, container: &Container) -> Result<()> {
        run_lifecycle_hooks("module_destroy", &self.module_destroy_hooks, container)
    }

    pub fn run_application_bootstrap(&self, container: &Container) -> Result<()> {
        run_lifecycle_hooks(
            "application_bootstrap",
            &self.application_bootstrap_hooks,
            container,
        )
    }

    pub fn run_application_shutdown(&self, container: &Container) -> Result<()> {
        run_lifecycle_hooks(
            "application_shutdown",
            &self.application_shutdown_hooks,
            container,
        )
    }
}

pub fn initialize_module_runtime<M: ModuleDefinition>(
    container: &Container,
) -> Result<InitializedModule> {
    let mut state = ModuleGraphState::default();
    visit_module(ModuleRef::of::<M>(), container, &mut state)?;
    state.module_destroy_hooks.reverse();
    state.application_shutdown_hooks.reverse();
    Ok(InitializedModule {
        controllers: state.controllers,
        module_init_hooks: state.module_init_hooks,
        module_destroy_hooks: state.module_destroy_hooks,
        application_bootstrap_hooks: state.application_bootstrap_hooks,
        application_shutdown_hooks: state.application_shutdown_hooks,
    })
}

pub fn initialize_module_graph<M: ModuleDefinition>(
    container: &Container,
) -> Result<Vec<Router<Container>>> {
    Ok(initialize_module_runtime::<M>(container)?.controllers)
}

pub fn collect_module_route_docs<M: ModuleDefinition>() -> Result<Vec<RouteDocumentation>> {
    let mut state = DocumentationGraphState::default();
    visit_module_docs(ModuleRef::of::<M>(), &mut state)?;
    Ok(state.route_docs)
}

pub fn collect_module_graph<M: ModuleDefinition>() -> Result<ModuleGraphReport> {
    let mut state = ModuleIntrospectionState::default();
    visit_module_graph(ModuleRef::of::<M>(), &mut state)?;
    Ok(ModuleGraphReport {
        modules: state.modules,
    })
}

#[derive(Default)]
struct ModuleGraphState {
    visited: HashSet<&'static str>,
    visiting: HashSet<&'static str>,
    stack: Vec<&'static str>,
    controllers: Vec<Router<Container>>,
    module_init_hooks: Vec<LifecycleHook>,
    module_destroy_hooks: Vec<LifecycleHook>,
    application_bootstrap_hooks: Vec<LifecycleHook>,
    application_shutdown_hooks: Vec<LifecycleHook>,
    global_modules: HashSet<&'static str>,
}

#[derive(Default)]
struct DocumentationGraphState {
    visited: HashSet<&'static str>,
    visiting: HashSet<&'static str>,
    stack: Vec<&'static str>,
    route_docs: Vec<RouteDocumentation>,
}

#[derive(Default)]
struct ModuleIntrospectionState {
    visited: HashSet<&'static str>,
    visiting: HashSet<&'static str>,
    stack: Vec<&'static str>,
    modules: Vec<ModuleGraphEntry>,
}

fn visit_module(
    module: ModuleRef,
    container: &Container,
    state: &mut ModuleGraphState,
) -> Result<()> {
    /*
    If we've already set up this module, we're done.
    This prevents re-processing modules that are imported by multiple other modules.
    */
    if state.visited.contains(module.name) {
        return Ok(());
    }

    /*
    This check prevents infinite loops if two modules import each other.
    We track the current recursion stack in `visiting`.
    */
    if state.visiting.contains(module.name) {
        let mut cycle = state.stack.clone();
        cycle.push(module.name);
        anyhow::bail!("Detected module import cycle: {}", cycle.join(" -> "));
    }

    state.visiting.insert(module.name);
    state.stack.push(module.name);
    framework_log_event("module_register", &[("module", module.name.to_string())]);

    /*
    First, we recursively initialize every module that this one imports.
    This ensures that dependencies are always ready before they're needed.
    DFS traversal guarantees leaf modules are initialized first.
    */
    for imported in (module.imports)() {
        visit_module(imported, container, state)?;
    }

    /*
    Now we run the actual registration logic for this module (usually adding providers).
    This is where `Provider::value(...)` or `Provider::factory(...)` items get added to the DI container.
    */
    (module.register)(container)
        .map_err(|err| anyhow::anyhow!("Failed to register module `{}`: {}", module.name, err))?;

    /*
    We collect all the components and hooks from this module into the global state.
    This includes controllers, lifecycle hooks, and global flags.
    */
    state.controllers.extend((module.controllers)());
    state.module_init_hooks.extend((module.on_module_init)());
    state
        .module_destroy_hooks
        .extend((module.on_module_destroy)());
    state
        .application_bootstrap_hooks
        .extend((module.on_application_bootstrap)());
    state
        .application_shutdown_hooks
        .extend((module.on_application_shutdown)());

    if (module.is_global)() {
        state.global_modules.insert(module.name);
    }

    /*
    Finally, we verify that every type this module promised to export
    was actually registered in the container. This catches typos or missing providers early.
    */
    for export in (module.exports)() {
        let is_registered = container.is_type_registered_name(export).map_err(|err| {
            anyhow::anyhow!(
                "Failed to verify exports for module `{}`: {}",
                module.name,
                err
            )
        })?;
        if !is_registered {
            anyhow::bail!(
                "Module `{}` exports `{}` but that provider is not registered in the container",
                module.name,
                export
            );
        }
    }

    state.stack.pop();
    state.visiting.remove(module.name);
    state.visited.insert(module.name);

    Ok(())
}

fn run_lifecycle_hooks(
    phase: &'static str,
    hooks: &[LifecycleHook],
    container: &Container,
) -> Result<()> {
    for hook in hooks {
        framework_log_event("lifecycle_hook_run", &[("phase", phase.to_string())]);
        hook(container)?;
    }

    Ok(())
}

fn block_on_dynamic_registration<T, Fut>(future: Fut) -> Result<T>
where
    T: Send + 'static,
    Fut: Future<Output = Result<T>> + Send + 'static,
{
    if tokio::runtime::Handle::try_current().is_ok() {
        return std::thread::spawn(move || {
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(anyhow::Error::new)?
                .block_on(future)
        })
        .join()
        .map_err(|_| anyhow::anyhow!("Dynamic module async registration thread panicked"))?;
    }

    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(anyhow::Error::new)?
        .block_on(future)
}

fn visit_module_docs(module: ModuleRef, state: &mut DocumentationGraphState) -> Result<()> {
    if state.visited.contains(module.name) {
        return Ok(());
    }

    if state.visiting.contains(module.name) {
        let mut cycle = state.stack.clone();
        cycle.push(module.name);
        anyhow::bail!(
            "Detected module import cycle while collecting route docs: {}",
            cycle.join(" -> ")
        );
    }

    state.visiting.insert(module.name);
    state.stack.push(module.name);

    for imported in (module.imports)() {
        visit_module_docs(imported, state)?;
    }

    state.route_docs.extend((module.route_docs)());
    state.stack.pop();
    state.visiting.remove(module.name);
    state.visited.insert(module.name);
    Ok(())
}

fn visit_module_graph(module: ModuleRef, state: &mut ModuleIntrospectionState) -> Result<()> {
    if state.visited.contains(module.name) {
        return Ok(());
    }

    if state.visiting.contains(module.name) {
        let mut cycle = state.stack.clone();
        cycle.push(module.name);
        anyhow::bail!(
            "Detected module import cycle while collecting module graph: {}",
            cycle.join(" -> ")
        );
    }

    state.visiting.insert(module.name);
    state.stack.push(module.name);

    let imports = (module.imports)();
    let import_names = imports.iter().map(|import| import.name).collect::<Vec<_>>();

    for imported in imports {
        visit_module_graph(imported, state)?;
    }

    state.modules.push(ModuleGraphEntry {
        name: module.name,
        imports: import_names,
        exports: (module.exports)(),
        controller_count: (module.controllers)().len(),
        is_global: (module.is_global)(),
    });

    state.stack.pop();
    state.visiting.remove(module.name);
    state.visited.insert(module.name);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    struct ImportedConfig;
    struct AppService;

    struct ImportedModule;
    impl ModuleDefinition for ImportedModule {
        fn register(container: &Container) -> Result<()> {
            container.register(ImportedConfig)?;
            Ok(())
        }
    }

    struct AppModule;
    impl ModuleDefinition for AppModule {
        fn imports() -> Vec<ModuleRef> {
            vec![ModuleRef::of::<ImportedModule>()]
        }

        fn register(container: &Container) -> Result<()> {
            let imported = container.resolve::<ImportedConfig>()?;
            container.register(AppService::from(imported))?;
            Ok(())
        }
    }

    impl From<std::sync::Arc<ImportedConfig>> for AppService {
        fn from(_: std::sync::Arc<ImportedConfig>) -> Self {
            Self
        }
    }

    #[test]
    fn registers_imported_modules_before_local_providers() {
        let container = Container::new();
        let result = initialize_module_graph::<AppModule>(&container);

        assert!(result.is_ok(), "module graph registration should succeed");
        assert!(container.resolve::<ImportedConfig>().is_ok());
        assert!(container.resolve::<AppService>().is_ok());
    }

    struct SharedImportedModule;
    impl ModuleDefinition for SharedImportedModule {
        fn register(container: &Container) -> Result<()> {
            container.register(SharedMarker)?;
            Ok(())
        }
    }

    struct SharedMarker;
    struct LeftModule;
    struct RightModule;
    struct RootModule;

    impl ModuleDefinition for LeftModule {
        fn imports() -> Vec<ModuleRef> {
            vec![ModuleRef::of::<SharedImportedModule>()]
        }

        fn register(_container: &Container) -> Result<()> {
            Ok(())
        }
    }

    impl ModuleDefinition for RightModule {
        fn imports() -> Vec<ModuleRef> {
            vec![ModuleRef::of::<SharedImportedModule>()]
        }

        fn register(_container: &Container) -> Result<()> {
            Ok(())
        }
    }

    impl ModuleDefinition for RootModule {
        fn imports() -> Vec<ModuleRef> {
            vec![
                ModuleRef::of::<LeftModule>(),
                ModuleRef::of::<RightModule>(),
            ]
        }

        fn register(_container: &Container) -> Result<()> {
            Ok(())
        }
    }

    #[test]
    fn deduplicates_shared_imported_modules() {
        let container = Container::new();
        let result = initialize_module_graph::<RootModule>(&container);

        assert!(
            result.is_ok(),
            "shared imported modules should only register once"
        );
        assert!(container.resolve::<SharedMarker>().is_ok());
    }

    struct CycleA;
    struct CycleB;

    impl ModuleDefinition for CycleA {
        fn imports() -> Vec<ModuleRef> {
            vec![ModuleRef::of::<CycleB>()]
        }

        fn register(_container: &Container) -> Result<()> {
            Ok(())
        }
    }

    impl ModuleDefinition for CycleB {
        fn imports() -> Vec<ModuleRef> {
            vec![ModuleRef::of::<CycleA>()]
        }

        fn register(_container: &Container) -> Result<()> {
            Ok(())
        }
    }

    #[test]
    fn detects_module_import_cycles() {
        let container = Container::new();
        let err = initialize_module_graph::<CycleA>(&container).unwrap_err();

        assert!(
            err.to_string().contains("Detected module import cycle"),
            "error should include cycle detection message"
        );
    }

    struct MissingDependency;
    struct BrokenModule;

    impl ModuleDefinition for BrokenModule {
        fn register(container: &Container) -> Result<()> {
            let _ = container.resolve_in_module::<MissingDependency>(Self::module_name())?;
            Ok(())
        }
    }

    #[test]
    fn module_registration_error_includes_module_and_type_context() {
        let container = Container::new();
        let err = initialize_module_graph::<BrokenModule>(&container).unwrap_err();
        let message = err.to_string();

        assert!(message.contains("Failed to register module"));
        assert!(message.contains("BrokenModule"));
        assert!(message.contains("MissingDependency"));
    }

    #[derive(Clone)]
    struct HookLog(Arc<Mutex<Vec<&'static str>>>);

    fn push_hook(container: &Container, label: &'static str) -> Result<()> {
        let log = container.resolve::<HookLog>()?;
        log.0
            .lock()
            .expect("hook log should be writable")
            .push(label);
        Ok(())
    }

    fn hook_init(container: &Container) -> Result<()> {
        push_hook(container, "module_init")
    }

    fn hook_bootstrap(container: &Container) -> Result<()> {
        push_hook(container, "application_bootstrap")
    }

    fn hook_destroy(container: &Container) -> Result<()> {
        push_hook(container, "module_destroy")
    }

    fn hook_shutdown(container: &Container) -> Result<()> {
        push_hook(container, "application_shutdown")
    }

    struct HookModule;

    impl ModuleDefinition for HookModule {
        fn register(container: &Container) -> Result<()> {
            container.register(HookLog(Arc::new(Mutex::new(Vec::new()))))?;
            Ok(())
        }

        fn on_module_init() -> Vec<LifecycleHook> {
            vec![hook_init]
        }

        fn on_application_bootstrap() -> Vec<LifecycleHook> {
            vec![hook_bootstrap]
        }

        fn on_module_destroy() -> Vec<LifecycleHook> {
            vec![hook_destroy]
        }

        fn on_application_shutdown() -> Vec<LifecycleHook> {
            vec![hook_shutdown]
        }
    }

    #[test]
    fn initialized_runtime_runs_lifecycle_hooks_in_expected_order() {
        let container = Container::new();
        let runtime = initialize_module_runtime::<HookModule>(&container)
            .expect("module runtime should initialize");

        runtime
            .run_module_init(&container)
            .expect("module init hooks should run");
        runtime
            .run_application_bootstrap(&container)
            .expect("application bootstrap hooks should run");
        runtime
            .run_module_destroy(&container)
            .expect("module destroy hooks should run");
        runtime
            .run_application_shutdown(&container)
            .expect("application shutdown hooks should run");

        let log = container
            .resolve::<HookLog>()
            .expect("hook log should resolve");
        let entries = log.0.lock().expect("hook log should be readable").clone();

        assert_eq!(
            entries,
            vec![
                "module_init",
                "application_bootstrap",
                "module_destroy",
                "application_shutdown"
            ]
        );
    }

    #[derive(Clone)]
    struct DynamicConfig {
        label: &'static str,
    }

    struct DynamicRootModule;

    impl ModuleDefinition for DynamicRootModule {
        fn imports() -> Vec<ModuleRef> {
            let config = DynamicConfig { label: "captured" };
            vec![ModuleRef::dynamic("DynamicConfigModule", move |container| {
                container.register(config.clone())?;
                Ok(())
            })
            .with_exports(|| vec![std::any::type_name::<DynamicConfig>()])]
        }

        fn register(_container: &Container) -> Result<()> {
            Ok(())
        }
    }

    #[test]
    fn dynamic_module_ref_can_capture_runtime_configuration() {
        let container = Container::new();
        initialize_module_graph::<DynamicRootModule>(&container)
            .expect("dynamic module graph should initialize");

        let config = container
            .resolve::<DynamicConfig>()
            .expect("dynamic config should resolve");
        assert_eq!(config.label, "captured");
    }

    struct BuilderRootModule;

    impl ModuleDefinition for BuilderRootModule {
        fn imports() -> Vec<ModuleRef> {
            vec![ModuleRef::builder("BuilderConfigModule")
                .provider_value(DynamicConfig { label: "builder" })
                .export::<DynamicConfig>()
                .build()]
        }

        fn register(_container: &Container) -> Result<()> {
            Ok(())
        }
    }

    #[test]
    fn dynamic_module_builder_registers_typed_exports() {
        let container = Container::new();
        initialize_module_graph::<BuilderRootModule>(&container)
            .expect("builder-based dynamic module should initialize");

        let config = container
            .resolve::<DynamicConfig>()
            .expect("builder config should resolve");
        assert_eq!(config.label, "builder");
    }

    #[derive(Clone)]
    struct AsyncConfig {
        label: &'static str,
    }

    struct AsyncRootModule;

    impl ModuleDefinition for AsyncRootModule {
        fn imports() -> Vec<ModuleRef> {
            vec![ModuleRef::builder("AsyncConfigModule")
                .provider_async(|_container| async { Ok(AsyncConfig { label: "async" }) })
                .export::<AsyncConfig>()
                .build()]
        }

        fn register(_container: &Container) -> Result<()> {
            Ok(())
        }
    }

    #[test]
    fn dynamic_module_builder_supports_async_provider_registration() {
        let container = Container::new();
        initialize_module_graph::<AsyncRootModule>(&container)
            .expect("async dynamic module should initialize");

        let config = container
            .resolve::<AsyncConfig>()
            .expect("async config should resolve");
        assert_eq!(config.label, "async");
    }

    #[test]
    fn collects_module_graph_report_entries() {
        let report = collect_module_graph::<RootModule>().expect("module graph should collect");
        let root = report
            .module(std::any::type_name::<RootModule>())
            .expect("root module should exist");

        assert_eq!(root.imports.len(), 2);
        assert_eq!(report.modules.len(), 4);
    }
}
