use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
    bracketed,
    parse::{Parse, ParseStream},
    parse_macro_input, parse_quote,
    punctuated::Punctuated,
    spanned::Spanned,
    Attribute, Data, DeriveInput, Expr, Field, Fields, FnArg, GenericArgument, Ident, ImplItem,
    ImplItemFn, ItemImpl, ItemStruct, LitStr, Meta, PatType, PathArguments, ReturnType, Token,
    Type,
};

/// The `#[controller]` macro is your starting point for defining a route group.
///
/// It takes a base path (like `"/users"`) and attaches it to your struct.
/// Behind the scenes, this implements the `ControllerBasePath` trait, which
/// NestForge uses later to mount your routes at the right URL.
#[proc_macro_attribute]
pub fn controller(attr: TokenStream, item: TokenStream) -> TokenStream {
    /*
    We expect the attribute to be a simple string literal: #[controller("/path")]
    This path will be used as a prefix for all routes in the controller.
    */
    let base_path = parse_macro_input!(attr as LitStr);
    let input = parse_macro_input!(item as ItemStruct);

    let name = &input.ident;
    let path = base_path.value();

    /*
    We keep your original struct but add the metadata trait implementation.
    This allows the framework to discover the base path at runtime.
    */
    let expanded = quote! {
        #input

        impl nestforge::ControllerBasePath for #name {
            fn base_path() -> &'static str {
                #path
            }
        }
    };

    TokenStream::from(expanded)
}

/// `#[injectable]` marks a struct as something NestForge can manage for you.
///
/// By default, it assumes your struct implements `Default`. If you need more
/// control, you can provide a factory function: `#[injectable(factory = my_factory)]`.
///
/// It also automatically adds `#[derive(Clone)]` to your struct, because providers
/// need to be shared across the application.
#[proc_macro_attribute]
pub fn injectable(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as InjectableArgs);
    let mut input = parse_macro_input!(item as ItemStruct);

    /*
    Providers must be Clone so they can be passed around the DI container.
    We check if Clone is already derived; if not, we add it.
    */
    ensure_derive_trait(&mut input.attrs, "Clone");

    let name = &input.ident;

    /*
    We decide how to register the provider based on whether a factory was provided.
    If a factory is present, we wrap it in a closure that converts the result into an `IntoInjectableResult`.
    Otherwise, we just use the Default trait.
    */
    let register_body = if let Some(factory) = args.factory {
        quote! {
            let value: Self =
                nestforge::IntoInjectableResult::into_injectable_result((#factory)())?;
            container.register(value)?;
            Ok(())
        }
    } else {
        quote! {
            container.register(<Self as std::default::Default>::default())?;
            Ok(())
        }
    };

    let expanded = quote! {
        #input

        impl nestforge::Injectable for #name {
            fn register(container: &nestforge::Container) -> anyhow::Result<()> {
                #register_body
            }
        }
    };

    TokenStream::from(expanded)
}

/// `#[routes]` is where the magic happens for your controllers.
///
/// It looks at all the methods in your `impl` block and finds ones marked with
/// `#[get]`, `#[post]`, etc. It then generates a `ControllerDefinition` that
/// knows how to build an Axum router with all those routes wired up.
///
/// It also handles:
/// - Extracting guards, interceptors, and filters.
/// - Merging controller-level metadata with method-level metadata.
/// - Generating documentation for your API.
#[proc_macro_attribute]
pub fn routes(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as ItemImpl);

    let self_ty = input.self_ty.clone();

    /*
    First, we pull out any metadata from the top of the impl block.
    This includes things like `#[tag(...)]`, `#[authenticated]`, or `#[roles(...)]` that apply to all routes.
    */
    let controller_meta = extract_controller_route_meta(&mut input);

    let mut route_calls = Vec::new();
    let mut route_docs = Vec::new();

    /*
    Now we loop through every method to see if it's a route.
    We look for methods decorated with `#[get]`, `#[post]`, etc.
    */
    for impl_item in &mut input.items {
        let ImplItem::Fn(ref mut method) = impl_item else {
            continue;
        };

        /*
        Extract all the "middleware-like" metadata for this specific method.
        This includes guards, interceptors, and exception filters.
        */
        let (guards, interceptors, exception_filters) = extract_pipeline_meta(method);
        let version = extract_version_meta(method);
        let mut doc_meta = extract_route_doc_meta(method);

        /*
        Merge the controller-level settings (like tags or auth) into the route.
        Route-level settings generally add to or override controller-level ones.
        */
        doc_meta.tags = merge_string_lists(controller_meta.tags.clone(), doc_meta.tags);
        doc_meta.required_roles = merge_string_lists(
            controller_meta.required_roles.clone(),
            doc_meta.required_roles,
        );
        doc_meta.requires_auth = controller_meta.requires_auth
            || doc_meta.requires_auth
            || !doc_meta.required_roles.is_empty();

        let guards = merge_type_lists(controller_meta.guards.clone(), guards);
        let interceptors = merge_type_lists(controller_meta.interceptors.clone(), interceptors);
        let exception_filters =
            merge_type_lists(controller_meta.exception_filters.clone(), exception_filters);

        /*
        If the method has an HTTP attribute (like #[get("/")]), we process it.
        This is where we generate the router configuration code.
        */
        if let Some((http_method, path)) = extract_route_meta(method) {
            let method_name = &method.sig.ident;
            let path_lit = LitStr::new(&path, method.sig.ident.span());

            /*
            We generate the code to initialize all the guards and interceptors.
            These are instantiated as Arcs and passed to the route builder.
            */
            let guard_inits = guards.iter().map(|ty| {
                quote! { std::sync::Arc::new(<#ty as std::default::Default>::default()) as std::sync::Arc<dyn nestforge::Guard> }
            });

            /*
            Special handling for auth and role guards.
            If authentication is required, we add the standard RequireAuthenticationGuard.
            If roles are required, we add the RoleRequirementsGuard.
            */
            let auth_guard_init = if doc_meta.requires_auth && doc_meta.required_roles.is_empty() {
                quote! {
                    std::sync::Arc::new(nestforge::RequireAuthenticationGuard::default())
                        as std::sync::Arc<dyn nestforge::Guard>
                }
            } else {
                quote! {}
            };
            let role_guard_init = if doc_meta.required_roles.is_empty() {
                quote! {}
            } else {
                let roles = doc_meta
                    .required_roles
                    .iter()
                    .map(|role| LitStr::new(role, method.sig.ident.span()));
                quote! {
                    std::sync::Arc::new(nestforge::RoleRequirementsGuard::new([#(#roles),*]))
                        as std::sync::Arc<dyn nestforge::Guard>
                }
            };

            let interceptor_inits = interceptors.iter().map(|ty| {
                quote! { std::sync::Arc::new(<#ty as std::default::Default>::default()) as std::sync::Arc<dyn nestforge::Interceptor> }
            });
            let exception_filter_inits = exception_filters.iter().map(|ty| {
                quote! { std::sync::Arc::new(<#ty as std::default::Default>::default()) as std::sync::Arc<dyn nestforge::ExceptionFilter> }
            });

            let guard_tokens = if doc_meta.requires_auth || !doc_meta.required_roles.is_empty() {
                quote! { vec![#(#guard_inits,)* #auth_guard_init #role_guard_init] }
            } else {
                quote! { vec![#(#guard_inits),*] }
            };

            let version_tokens = if let Some(version) = &version {
                let lit = LitStr::new(version, method.sig.ident.span());
                quote! { Some(#lit) }
            } else {
                quote! { None }
            };

            /*
            We build the actual call to the framework's RouteBuilder.
            This corresponds to `builder.get(...)`, `builder.post(...)`, etc.
            */
            let call = match http_method.as_str() {
                "get" => quote! {
                    builder = builder.get_with_pipeline(
                        #path_lit,
                        Self::#method_name,
                        #guard_tokens,
                        vec![#(#interceptor_inits),*],
                        vec![#(#exception_filter_inits),*],
                        #version_tokens
                    );
                },
                "post" => quote! {
                    builder = builder.post_with_pipeline(
                        #path_lit,
                        Self::#method_name,
                        #guard_tokens,
                        vec![#(#interceptor_inits),*],
                        vec![#(#exception_filter_inits),*],
                        #version_tokens
                    );
                },
                "put" => quote! {
                    builder = builder.put_with_pipeline(
                        #path_lit,
                        Self::#method_name,
                        #guard_tokens,
                        vec![#(#interceptor_inits),*],
                        vec![#(#exception_filter_inits),*],
                        #version_tokens
                    );
                },
                "delete" => quote! {
                    builder = builder.delete_with_pipeline(
                        #path_lit,
                        Self::#method_name,
                        #guard_tokens,
                        vec![#(#interceptor_inits),*],
                        vec![#(#exception_filter_inits),*],
                        #version_tokens
                    );
                },
                _ => continue,
            };

            route_calls.push(call);

            let method_lit = LitStr::new(&http_method.to_uppercase(), method.sig.ident.span());
            let response_docs = if doc_meta.responses.is_empty() {
                quote! {
                    vec![nestforge::RouteResponseDocumentation {
                        status: 200,
                        description: "OK".to_string(),
                        schema: None,
                    }]
                }
            } else {
                let responses = doc_meta.responses.iter().map(|response| {
                    let description = LitStr::new(&response.description, method.sig.ident.span());
                    let status = response.status;
                    quote! {
                        nestforge::RouteResponseDocumentation {
                            status: #status,
                            description: #description.to_string(),
                            schema: None,
                        }
                    }
                });
                quote! { vec![#(#responses),*] }
            };
            let request_schema_tokens = infer_request_body_doc_tokens(method);
            let response_schema_tokens = infer_response_body_doc_tokens(&method.sig.output);
            let summary_tokens = if let Some(summary) = &doc_meta.summary {
                let summary_lit = LitStr::new(summary, method.sig.ident.span());
                quote! { doc = doc.with_summary(#summary_lit); }
            } else {
                quote! {}
            };
            let description_tokens = if let Some(description) = &doc_meta.description {
                let description_lit = LitStr::new(description, method.sig.ident.span());
                quote! { doc = doc.with_description(#description_lit); }
            } else {
                quote! {}
            };
            let tag_tokens = if doc_meta.tags.is_empty() {
                quote! {}
            } else {
                let tags = doc_meta
                    .tags
                    .iter()
                    .map(|tag| LitStr::new(tag, method.sig.ident.span()));
                quote! { doc = doc.with_tags([#(#tags),*]); }
            };
            let auth_tokens = if doc_meta.requires_auth {
                quote! { doc = doc.requires_auth(); }
            } else {
                quote! {}
            };
            let role_tokens = if doc_meta.required_roles.is_empty() {
                quote! {}
            } else {
                let roles = doc_meta
                    .required_roles
                    .iter()
                    .map(|role| LitStr::new(role, method.sig.ident.span()));
                quote! { doc = doc.with_required_roles([#(#roles),*]); }
            };

            route_docs.push(quote! {
              {
                  let mut doc = nestforge::RouteDocumentation::new(
                      #method_lit,
                      nestforge::RouteBuilder::<#self_ty>::full_path(#path_lit, #version_tokens),
                  )
                  .with_responses(#response_docs);
                  #summary_tokens
                  #description_tokens
                    #tag_tokens
                    #auth_tokens
                    #role_tokens
                    #request_schema_tokens
                    #response_schema_tokens
                    doc
                }
            });
        }
    }

    let expanded = quote! {
        #input

        impl nestforge::ControllerDefinition for #self_ty {
            fn router() -> axum::Router<nestforge::Container> {
                nestforge::framework_log_event(
                    "controller_register",
                    &[("controller", std::string::String::from(std::any::type_name::<#self_ty>()))] as &[(&str, std::string::String)],
                );
                let mut builder = nestforge::RouteBuilder::<#self_ty>::new();
                #(#route_calls)*
                builder.build()
            }
        }

        impl nestforge::DocumentedController for #self_ty {
            fn route_docs() -> Vec<nestforge::RouteDocumentation> {
                vec![#(#route_docs),*]
            }
        }
    };

    TokenStream::from(expanded)
}

/*
#[module(
    imports = [AuthModule],
    controllers = [AppController, UsersController],
    providers = [AppConfig { ... }, UsersService::new()],
    exports = [UsersService]
)]
Generates ModuleDefinition for the struct.
*/
#[proc_macro_attribute]
pub fn module(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as ModuleArgs);
    let input = parse_macro_input!(item as ItemStruct);

    let name = &input.ident;

    let controller_calls = args.controllers.iter().map(|ty| {
        quote! { <#ty as nestforge::ControllerDefinition>::router() }
    });
    let controller_doc_calls = args.controllers.iter().map(|ty| {
        quote! { docs.extend(<#ty as nestforge::DocumentedController>::route_docs()); }
    });

    let provider_regs = args.providers.iter().map(build_provider_registration);

    let import_refs = args.imports.iter().map(|ty| {
        quote! { nestforge::ModuleRef::of::<#ty>() }
    });
    let module_init_hooks = args.on_module_init.iter().map(|expr| {
        quote! { #expr as nestforge::LifecycleHook }
    });
    let module_destroy_hooks = args.on_module_destroy.iter().map(|expr| {
        quote! { #expr as nestforge::LifecycleHook }
    });
    let application_bootstrap_hooks = args.on_application_bootstrap.iter().map(|expr| {
        quote! { #expr as nestforge::LifecycleHook }
    });
    let application_shutdown_hooks = args.on_application_shutdown.iter().map(|expr| {
        quote! { #expr as nestforge::LifecycleHook }
    });

    let exported_types = args.exports.iter().map(|ty| {
        quote! { std::any::type_name::<#ty>() }
    });
    let global_flag = args.global;

    let expanded = quote! {
        #input

        impl nestforge::ModuleDefinition for #name {
            fn register(container: &nestforge::Container) -> anyhow::Result<()> {
                #(#provider_regs)*
                Ok(())
            }

            fn imports() -> Vec<nestforge::ModuleRef> {
                vec![
                    #(#import_refs),*
                ]
            }

            fn exports() -> Vec<&'static str> {
                vec![
                    #(#exported_types),*
                ]
            }

            fn is_global() -> bool {
                #global_flag
            }

            fn controllers() -> Vec<axum::Router<nestforge::Container>> {
                vec![
                    #(#controller_calls),*
                ]
            }

            fn route_docs() -> Vec<nestforge::RouteDocumentation> {
                let mut docs = Vec::new();
                #(#controller_doc_calls)*
                docs
            }

            fn on_module_init() -> Vec<nestforge::LifecycleHook> {
                vec![#(#module_init_hooks),*]
            }

            fn on_module_destroy() -> Vec<nestforge::LifecycleHook> {
                vec![#(#module_destroy_hooks),*]
            }

            fn on_application_bootstrap() -> Vec<nestforge::LifecycleHook> {
                vec![#(#application_bootstrap_hooks),*]
            }

            fn on_application_shutdown() -> Vec<nestforge::LifecycleHook> {
                vec![#(#application_shutdown_hooks),*]
            }
        }
    };

    TokenStream::from(expanded)
}

/*
Method route attributes are markers consumed by #[routes].
*/
#[proc_macro_attribute]
pub fn get(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn post(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn put(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn delete(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn version(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn use_guard(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn use_interceptor(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn use_exception_filter(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn summary(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn description(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn tag(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn response(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn authenticated(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn roles(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

fn build_openapi_schema_impl(input: &ItemStruct) -> TokenStream2 {
    let name = &input.ident;
    let schema_body = build_openapi_schema_body(input);

    quote! {
        impl nestforge::OpenApiSchema for #name {
            fn schema_name() -> Option<&'static str> {
                Some(stringify!(#name))
            }

            fn schema() -> nestforge::serde_json::Value {
                #schema_body
            }
        }
    }
}

fn build_openapi_schema_body(input: &ItemStruct) -> TokenStream2 {
    let Fields::Named(fields) = &input.fields else {
        return quote! {
            nestforge::serde_json::json!({
                "type": "object",
                "properties": {},
                "required": []
            })
        };
    };

    let property_builders = fields
        .named
        .iter()
        .filter_map(build_openapi_property_tokens);
    let required_fields = fields
        .named
        .iter()
        .filter_map(required_field_literal)
        .collect::<Vec<_>>();

    quote! {{
        let mut properties = nestforge::serde_json::Map::new();
        #(#property_builders)*
        nestforge::serde_json::json!({
            "type": "object",
            "properties": properties,
            "required": [#(#required_fields),*]
        })
    }}
}

fn build_openapi_property_tokens(field: &Field) -> Option<TokenStream2> {
    let field_ident = field.ident.as_ref()?;
    let field_name = LitStr::new(&field_ident.to_string(), field_ident.span());
    let field_ty = &field.ty;
    let rules = parse_validate_rules(&field.attrs);
    let schema_expr = schema_expression_for_type(field_ty);
    let validations = validation_schema_mutations(&rules);

    Some(quote! {
        {
            let mut property = #schema_expr;
            #validations
            properties.insert(#field_name.to_string(), property);
        }
    })
}

fn required_field_literal(field: &Field) -> Option<LitStr> {
    let field_ident = field.ident.as_ref()?;
    let rules = parse_validate_rules(&field.attrs);
    if is_option_any(&field.ty) && !rules.required {
        return None;
    }

    Some(LitStr::new(&field_ident.to_string(), field_ident.span()))
}

fn validation_schema_mutations(rules: &ValidateRules) -> TokenStream2 {
    let mut tokens = Vec::new();

    if rules.email {
        tokens.push(quote! {
            if let Some(object) = property.as_object_mut() {
                object.insert(
                    "format".to_string(),
                    nestforge::serde_json::Value::String("email".to_string()),
                );
            }
        });
    }

    if let Some(min_length) = rules.min_length {
        tokens.push(quote! {
            if let Some(object) = property.as_object_mut() {
                object.insert(
                    "minLength".to_string(),
                    nestforge::serde_json::json!(#min_length),
                );
            }
        });
    }

    if let Some(max_length) = rules.max_length {
        tokens.push(quote! {
            if let Some(object) = property.as_object_mut() {
                object.insert(
                    "maxLength".to_string(),
                    nestforge::serde_json::json!(#max_length),
                );
            }
        });
    }

    if let Some(min) = &rules.min {
        tokens.push(quote! {
            if let Some(object) = property.as_object_mut() {
                object.insert(
                    "minimum".to_string(),
                    nestforge::serde_json::json!(#min),
                );
            }
        });
    }

    if let Some(max) = &rules.max {
        tokens.push(quote! {
            if let Some(object) = property.as_object_mut() {
                object.insert(
                    "maximum".to_string(),
                    nestforge::serde_json::json!(#max),
                );
            }
        });
    }

    quote! { #(#tokens)* }
}

#[proc_macro_attribute]
pub fn dto(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as ItemStruct);
    let schema_impl = build_openapi_schema_impl(&input);

    input.attrs.push(parse_quote!(
        #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, nestforge::Validate)]
    ));

    TokenStream::from(quote! {
        #input
        #schema_impl
    })
}

#[proc_macro_attribute]
pub fn identifiable(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemStruct);
    let name = &input.ident;

    let Some((id_field_name, id_field_ty)) = find_id_field(&input.fields) else {
        return syn::Error::new(
            input.ident.span(),
            "identifiable requires an `id: u64` field or a field marked with #[id]",
        )
        .to_compile_error()
        .into();
    };
    let ty_ok = matches!(id_field_ty, Type::Path(ref tp) if tp.path.is_ident("u64"));
    if !ty_ok {
        return syn::Error::new(
            id_field_ty.span(),
            "identifiable id field must be of type `u64`",
        )
        .to_compile_error()
        .into();
    }

    TokenStream::from(quote! {
        #input

        impl nestforge::Identifiable for #name {
            fn id(&self) -> u64 {
                self.#id_field_name
            }

            fn set_id(&mut self, id: u64) {
                self.#id_field_name = id;
            }
        }
    })
}

#[proc_macro_attribute]
pub fn response_dto(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as ItemStruct);
    let schema_impl = build_openapi_schema_impl(&input);

    input
        .attrs
        .push(parse_quote!(#[derive(Debug, Clone, serde::Serialize)]));

    TokenStream::from(quote! {
        #input
        #schema_impl
    })
}

#[proc_macro_attribute]
pub fn entity_dto(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as ItemStruct);

    let Some((id_field_name, id_field_ty)) = find_id_field(&input.fields) else {
        return syn::Error::new(
            input.ident.span(),
            "entity_dto requires an `id: u64` field or a field marked with #[id]",
        )
        .to_compile_error()
        .into();
    };
    let ty_ok = matches!(id_field_ty, Type::Path(ref tp) if tp.path.is_ident("u64"));
    if !ty_ok {
        return syn::Error::new(
            id_field_ty.span(),
            "entity_dto id field must be of type `u64`",
        )
        .to_compile_error()
        .into();
    }

    input.attrs.push(parse_quote!(
        #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, nestforge::Validate)]
    ));

    let name = &input.ident;
    let schema_impl = build_openapi_schema_impl(&input);

    TokenStream::from(quote! {
        #input

        impl nestforge::Identifiable for #name {
            fn id(&self) -> u64 {
                self.#id_field_name
            }

            fn set_id(&mut self, id: u64) {
                self.#id_field_name = id;
            }
        }

        #schema_impl
    })
}

#[proc_macro_attribute]
pub fn entity(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as EntityArgs);
    let mut input = parse_macro_input!(item as ItemStruct);

    let name = &input.ident;
    let Some((id_field_name, id_field_ty)) = extract_id_field(&mut input.fields) else {
        return syn::Error::new(
            input.ident.span(),
            "#[entity(...)] requires exactly one field annotated with #[id]",
        )
        .to_compile_error()
        .into();
    };

    let table_name = args.table.value();
    let id_column = id_field_name.to_string();

    let expanded = quote! {
        #input

        impl nestforge::EntityMeta for #name {
            type Id = #id_field_ty;

            fn table_name() -> &'static str {
                #table_name
            }

            fn id_column() -> &'static str {
                #id_column
            }

            fn id_value(&self) -> &Self::Id {
                &self.#id_field_name
            }
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn id(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_derive(Identifiable, attributes(id))]
pub fn derive_identifiable(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let name = &input.ident;

    let Data::Struct(data) = &input.data else {
        return syn::Error::new(
            input.ident.span(),
            "Identifiable can only be derived on structs",
        )
        .to_compile_error()
        .into();
    };

    let Some((id_field_name, id_field_ty)) = find_id_field(&data.fields) else {
        return syn::Error::new(
            input.ident.span(),
            "Identifiable derive requires an `id: u64` field or a field marked with #[id]",
        )
        .to_compile_error()
        .into();
    };

    let ty_ok = matches!(id_field_ty, Type::Path(ref tp) if tp.path.is_ident("u64"));
    if !ty_ok {
        return syn::Error::new(
            id_field_ty.span(),
            "Identifiable id field must be of type `u64`",
        )
        .to_compile_error()
        .into();
    }

    let expanded = quote! {
        impl nestforge::Identifiable for #name {
            fn id(&self) -> u64 {
                self.#id_field_name
            }

            fn set_id(&mut self, id: u64) {
                self.#id_field_name = id;
            }
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(Validate, attributes(validate))]
pub fn derive_validate(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let name = &input.ident;

    let Data::Struct(data) = &input.data else {
        return syn::Error::new(
            input.ident.span(),
            "Validate can only be derived on structs",
        )
        .to_compile_error()
        .into();
    };

    let Fields::Named(fields) = &data.fields else {
        return syn::Error::new(input.ident.span(), "Validate derive requires named fields")
            .to_compile_error()
            .into();
    };

    let mut checks = Vec::new();
    for field in &fields.named {
        let Some(field_ident) = &field.ident else {
            continue;
        };
        let field_name_lit = field_ident.to_string();
        let rules = parse_validate_rules(&field.attrs);
        if !rules.has_rules() {
            continue;
        }

        let is_string = is_type_named(&field.ty, "String");
        let is_option_string = is_option_of(&field.ty, "String");
        let is_option_any = is_option_any(&field.ty);
        let is_numeric = is_numeric_type(&field.ty);
        let is_option_numeric = is_option_numeric_type(&field.ty);

        if rules.required {
            if is_string {
                checks.push(quote! {
                    if self.#field_ident.trim().is_empty() {
                        errors.push(nestforge::ValidationIssue {
                            field: #field_name_lit,
                            message: format!("{} is required", #field_name_lit),
                        });
                    }
                });
            } else if is_option_string {
                checks.push(quote! {
                    match &self.#field_ident {
                        Some(v) if !v.trim().is_empty() => {}
                        _ => {
                            errors.push(nestforge::ValidationIssue {
                                field: #field_name_lit,
                                message: format!("{} is required", #field_name_lit),
                            });
                        }
                    }
                });
            } else if is_option_any {
                checks.push(quote! {
                    if self.#field_ident.is_none() {
                        errors.push(nestforge::ValidationIssue {
                            field: #field_name_lit,
                            message: format!("{} is required", #field_name_lit),
                        });
                    }
                });
            }
        }

        if rules.email {
            if is_string {
                checks.push(quote! {
                    if !self.#field_ident.trim().is_empty() && !self.#field_ident.contains('@') {
                        errors.push(nestforge::ValidationIssue {
                            field: #field_name_lit,
                            message: format!("{} must be a valid email", #field_name_lit),
                        });
                    }
                });
            } else if is_option_string {
                checks.push(quote! {
                    if let Some(v) = &self.#field_ident {
                        if !v.trim().is_empty() && !v.contains('@') {
                            errors.push(nestforge::ValidationIssue {
                                field: #field_name_lit,
                                message: format!("{} must be a valid email", #field_name_lit),
                            });
                        }
                    }
                });
            }
        }

        if let Some(min_length) = rules.min_length {
            if is_string {
                checks.push(quote! {
                    if self.#field_ident.len() < #min_length {
                        errors.push(nestforge::ValidationIssue {
                            field: #field_name_lit,
                            message: format!("{} must be at least {} characters", #field_name_lit, #min_length),
                        });
                    }
                });
            } else if is_option_string {
                checks.push(quote! {
                    if let Some(v) = &self.#field_ident {
                        if v.len() < #min_length {
                            errors.push(nestforge::ValidationIssue {
                                field: #field_name_lit,
                                message: format!("{} must be at least {} characters", #field_name_lit, #min_length),
                            });
                        }
                    }
                });
            }
        }

        if let Some(max_length) = rules.max_length {
            if is_string {
                checks.push(quote! {
                    if self.#field_ident.len() > #max_length {
                        errors.push(nestforge::ValidationIssue {
                            field: #field_name_lit,
                            message: format!("{} must be at most {} characters", #field_name_lit, #max_length),
                        });
                    }
                });
            } else if is_option_string {
                checks.push(quote! {
                    if let Some(v) = &self.#field_ident {
                        if v.len() > #max_length {
                            errors.push(nestforge::ValidationIssue {
                                field: #field_name_lit,
                                message: format!("{} must be at most {} characters", #field_name_lit, #max_length),
                            });
                        }
                    }
                });
            }
        }

        if let Some(min) = &rules.min {
            if is_numeric {
                checks.push(quote! {
                    if self.#field_ident < #min {
                        errors.push(nestforge::ValidationIssue {
                            field: #field_name_lit,
                            message: format!("{} must be at least {}", #field_name_lit, #min),
                        });
                    }
                });
            } else if is_option_numeric {
                checks.push(quote! {
                    if let Some(v) = self.#field_ident {
                        if v < #min {
                            errors.push(nestforge::ValidationIssue {
                                field: #field_name_lit,
                                message: format!("{} must be at least {}", #field_name_lit, #min),
                            });
                        }
                    }
                });
            }
        }

        if let Some(max) = &rules.max {
            if is_numeric {
                checks.push(quote! {
                    if self.#field_ident > #max {
                        errors.push(nestforge::ValidationIssue {
                            field: #field_name_lit,
                            message: format!("{} must be at most {}", #field_name_lit, #max),
                        });
                    }
                });
            } else if is_option_numeric {
                checks.push(quote! {
                    if let Some(v) = self.#field_ident {
                        if v > #max {
                            errors.push(nestforge::ValidationIssue {
                                field: #field_name_lit,
                                message: format!("{} must be at most {}", #field_name_lit, #max),
                            });
                        }
                    }
                });
            }
        }
    }

    let expanded = quote! {
        impl nestforge::Validate for #name {
            fn validate(&self) -> Result<(), nestforge::ValidationErrors> {
                let mut errors = Vec::new();
                #(#checks)*
                if errors.is_empty() {
                    Ok(())
                } else {
                    Err(nestforge::ValidationErrors::new(errors))
                }
            }
        }
    };

    TokenStream::from(expanded)
}

/* -------- helpers -------- */

fn extract_route_meta(method: &mut ImplItemFn) -> Option<(String, String)> {
    let mut found: Option<(String, String)> = None;
    let mut kept_attrs: Vec<Attribute> = Vec::new();

    for attr in method.attrs.drain(..) {
        let Some((verb, path)) = parse_route_attr(&attr) else {
            kept_attrs.push(attr);
            continue;
        };

        if found.is_none() {
            found = Some((verb, path));
        }
    }

    method.attrs = kept_attrs;
    found
}

fn extract_pipeline_meta(method: &mut ImplItemFn) -> (Vec<Type>, Vec<Type>, Vec<Type>) {
    let mut guards = Vec::new();
    let mut interceptors = Vec::new();
    let mut exception_filters = Vec::new();
    let mut kept_attrs: Vec<Attribute> = Vec::new();

    for attr in method.attrs.drain(..) {
        let ident = attr
            .path()
            .segments
            .last()
            .map(|seg| seg.ident.to_string())
            .unwrap_or_default();

        if ident == "use_guard" {
            if let Ok(ty) = attr.parse_args::<Type>() {
                guards.push(ty);
            }
            continue;
        }

        if ident == "use_interceptor" {
            if let Ok(ty) = attr.parse_args::<Type>() {
                interceptors.push(ty);
            }
            continue;
        }

        if ident == "use_exception_filter" {
            if let Ok(ty) = attr.parse_args::<Type>() {
                exception_filters.push(ty);
            }
            continue;
        }

        kept_attrs.push(attr);
    }

    method.attrs = kept_attrs;
    (guards, interceptors, exception_filters)
}

#[derive(Default)]
struct ControllerRouteMeta {
    guards: Vec<Type>,
    interceptors: Vec<Type>,
    exception_filters: Vec<Type>,
    tags: Vec<String>,
    requires_auth: bool,
    required_roles: Vec<String>,
}

fn extract_controller_route_meta(input: &mut ItemImpl) -> ControllerRouteMeta {
    let mut meta = ControllerRouteMeta::default();
    let mut kept_attrs: Vec<Attribute> = Vec::new();

    for attr in input.attrs.drain(..) {
        let ident = attr
            .path()
            .segments
            .last()
            .map(|seg| seg.ident.to_string())
            .unwrap_or_default();

        match ident.as_str() {
            "use_guard" => {
                if let Ok(ty) = attr.parse_args::<Type>() {
                    meta.guards.push(ty);
                }
            }
            "use_interceptor" => {
                if let Ok(ty) = attr.parse_args::<Type>() {
                    meta.interceptors.push(ty);
                }
            }
            "use_exception_filter" => {
                if let Ok(ty) = attr.parse_args::<Type>() {
                    meta.exception_filters.push(ty);
                }
            }
            "tag" => {
                if let Ok(lit) = attr.parse_args::<LitStr>() {
                    meta.tags.push(lit.value());
                }
            }
            "authenticated" => {
                meta.requires_auth = true;
            }
            "roles" => {
                if let Ok(values) =
                    attr.parse_args_with(Punctuated::<LitStr, Token![,]>::parse_terminated)
                {
                    meta.required_roles
                        .extend(values.into_iter().map(|value| value.value()));
                    meta.requires_auth = true;
                }
            }
            _ => kept_attrs.push(attr),
        }
    }

    input.attrs = kept_attrs;
    meta
}

fn extract_version_meta(method: &mut ImplItemFn) -> Option<String> {
    let mut version: Option<String> = None;
    let mut kept_attrs: Vec<Attribute> = Vec::new();

    for attr in method.attrs.drain(..) {
        let ident = attr
            .path()
            .segments
            .last()
            .map(|seg| seg.ident.to_string())
            .unwrap_or_default();

        if ident == "version" {
            if let Ok(lit) = attr.parse_args::<LitStr>() {
                version = Some(lit.value());
            }
            continue;
        }

        kept_attrs.push(attr);
    }

    method.attrs = kept_attrs;
    version
}

#[derive(Default)]
struct RouteDocMeta {
    summary: Option<String>,
    description: Option<String>,
    tags: Vec<String>,
    responses: Vec<RouteResponseMeta>,
    requires_auth: bool,
    required_roles: Vec<String>,
}

struct RouteResponseMeta {
    status: u16,
    description: String,
}

fn extract_route_doc_meta(method: &mut ImplItemFn) -> RouteDocMeta {
    let mut meta = RouteDocMeta::default();
    let mut kept_attrs: Vec<Attribute> = Vec::new();

    for attr in method.attrs.drain(..) {
        let ident = attr
            .path()
            .segments
            .last()
            .map(|seg| seg.ident.to_string())
            .unwrap_or_default();

        match ident.as_str() {
            "summary" => {
                if let Ok(lit) = attr.parse_args::<LitStr>() {
                    meta.summary = Some(lit.value());
                }
            }
            "description" => {
                if let Ok(lit) = attr.parse_args::<LitStr>() {
                    meta.description = Some(lit.value());
                }
            }
            "tag" => {
                if let Ok(lit) = attr.parse_args::<LitStr>() {
                    meta.tags.push(lit.value());
                }
            }
            "response" => {
                if let Ok(response) = attr.parse_args::<RouteResponseArgs>() {
                    meta.responses.push(RouteResponseMeta {
                        status: response.status,
                        description: response.description.value(),
                    });
                }
            }
            "authenticated" => {
                meta.requires_auth = true;
            }
            "roles" => {
                if let Ok(values) =
                    attr.parse_args_with(Punctuated::<LitStr, Token![,]>::parse_terminated)
                {
                    meta.required_roles
                        .extend(values.into_iter().map(|value| value.value()));
                    meta.requires_auth = true;
                }
            }
            _ => kept_attrs.push(attr),
        }
    }

    method.attrs = kept_attrs;
    meta
}

fn merge_string_lists(primary: Vec<String>, secondary: Vec<String>) -> Vec<String> {
    let mut merged = primary;
    for value in secondary {
        if !merged.contains(&value) {
            merged.push(value);
        }
    }
    merged
}

fn merge_type_lists(primary: Vec<Type>, secondary: Vec<Type>) -> Vec<Type> {
    let mut merged = primary;
    for ty in secondary {
        if !merged
            .iter()
            .any(|existing| quote!(#existing).to_string() == quote!(#ty).to_string())
        {
            merged.push(ty);
        }
    }
    merged
}

fn parse_route_attr(attr: &Attribute) -> Option<(String, String)> {
    /*
    Support both:
    - #[get("/")]
    - #[nestforge::get("/")]
    */
    let ident = attr.path().segments.last()?.ident.to_string();

    if ident != "get" && ident != "post" && ident != "put" && ident != "delete" {
        return None;
    }

    let path = match &attr.meta {
        Meta::List(_) => attr.parse_args::<LitStr>().ok()?.value(),
        _ => return None,
    };

    Some((ident, path))
}

fn infer_request_body_doc_tokens(method: &ImplItemFn) -> TokenStream2 {
    let Some(payload_ty) = method
        .sig
        .inputs
        .iter()
        .find_map(extract_request_payload_type)
    else {
        return quote! {};
    };

    let schema_expr = schema_expression_for_type(&payload_ty);
    quote! {
        doc = doc.with_request_body_schema(#schema_expr);
        doc = doc.with_schema_components(nestforge::openapi_schema_components_for::<#payload_ty>());
    }
}

fn infer_response_body_doc_tokens(output: &ReturnType) -> TokenStream2 {
    let Some(schema_doc) = extract_response_payload_doc(output) else {
        return quote! {};
    };

    schema_doc
}

fn extract_request_payload_type(arg: &FnArg) -> Option<Type> {
    let FnArg::Typed(PatType { ty, .. }) = arg else {
        return None;
    };

    extract_inner_type_named(ty, &["ValidatedBody", "Body", "Json"])
}

fn extract_response_payload_doc(output: &ReturnType) -> Option<TokenStream2> {
    let ReturnType::Type(_, ty) = output else {
        return None;
    };

    response_payload_doc_tokens(ty)
}

fn response_payload_doc_tokens(ty: &Type) -> Option<TokenStream2> {
    if let Some((value_ty, serializer_ty)) =
        extract_two_inner_types_named(ty, &["ApiSerializedResult"])
    {
        return Some(quote! {
            doc = doc.with_success_response_schema(
                nestforge::openapi_schema_for::<<#serializer_ty as nestforge::ResponseSerializer<#value_ty>>::Output>()
            );
            doc = doc.with_schema_components(
                nestforge::openapi_schema_components_for::<<#serializer_ty as nestforge::ResponseSerializer<#value_ty>>::Output>()
            );
        });
    }

    if let Some(inner) = extract_inner_type_named(ty, &["ApiEnvelopeResult"]) {
        let schema_expr = quote! {{
            nestforge::serde_json::json!({
                "type": "object",
                "properties": {
                    "success": nestforge::openapi_schema_for::<bool>(),
                    "data": nestforge::openapi_schema_for::<#inner>()
                },
                "required": ["success", "data"]
            })
        }};
        return Some(quote! {
            doc = doc.with_success_response_schema(#schema_expr);
            doc = doc.with_schema_components(nestforge::openapi_schema_components_for::<#inner>());
        });
    }

    if let Some(inner) = extract_inner_type_named(ty, &["ApiResult", "Json"]) {
        return response_payload_doc_tokens(&inner).or_else(|| {
            let schema_expr = schema_expression_for_type(&inner);
            Some(quote! {
                doc = doc.with_success_response_schema(#schema_expr);
                doc = doc.with_schema_components(nestforge::openapi_schema_components_for::<#inner>());
            })
        });
    }

    if let Some(inner) = extract_inner_type_named(ty, &["Result"]) {
        return response_payload_doc_tokens(&inner).or_else(|| {
            let schema_expr = schema_expression_for_type(&inner);
            Some(quote! {
                doc = doc.with_success_response_schema(#schema_expr);
                doc = doc.with_schema_components(nestforge::openapi_schema_components_for::<#inner>());
            })
        });
    }

    if let Some((value_ty, serializer_ty)) = extract_serialized_types(ty) {
        return Some(quote! {
            doc = doc.with_success_response_schema(
                nestforge::openapi_schema_for::<<#serializer_ty as nestforge::ResponseSerializer<#value_ty>>::Output>()
            );
            doc = doc.with_schema_components(
                nestforge::openapi_schema_components_for::<<#serializer_ty as nestforge::ResponseSerializer<#value_ty>>::Output>()
            );
        });
    }

    if let Some(inner) = extract_inner_type_named(ty, &["ResponseEnvelope"]) {
        let schema_expr = quote!({
            nestforge::serde_json::json!({
                "type": "object",
                "properties": {
                    "success": nestforge::openapi_schema_for::<bool>(),
                    "data": nestforge::openapi_schema_for::<#inner>()
                },
                "required": ["success", "data"]
            })
        });
        return Some(quote! {
            doc = doc.with_success_response_schema(#schema_expr);
            doc = doc.with_schema_components(nestforge::openapi_schema_components_for::<#inner>());
        });
    }

    let schema_expr = schema_expression_for_type(ty);
    Some(quote! {
        doc = doc.with_success_response_schema(#schema_expr);
        doc = doc.with_schema_components(nestforge::openapi_schema_components_for::<#ty>());
    })
}

fn schema_expression_for_type(ty: &Type) -> TokenStream2 {
    if let Some(inner) = extract_inner_type_named(ty, &["Vec", "List"]) {
        return quote! { nestforge::openapi_array_schema_for::<#inner>() };
    }

    if let Some(inner) = extract_inner_type_named(ty, &["Option"]) {
        return quote! { nestforge::openapi_nullable_schema_for::<#inner>() };
    }

    quote! { nestforge::openapi_schema_for::<#ty>() }
}

fn extract_inner_type_named(ty: &Type, names: &[&str]) -> Option<Type> {
    let Type::Path(type_path) = ty else {
        return None;
    };
    let segment = type_path.path.segments.last()?;
    if !names.iter().any(|name| segment.ident == *name) {
        return None;
    }

    let PathArguments::AngleBracketed(args) = &segment.arguments else {
        return None;
    };

    args.args.iter().find_map(|arg| match arg {
        GenericArgument::Type(inner) => Some(inner.clone()),
        _ => None,
    })
}

fn extract_serialized_types(ty: &Type) -> Option<(Type, Type)> {
    extract_two_inner_types_named(ty, &["Serialized"])
}

fn extract_two_inner_types_named(ty: &Type, names: &[&str]) -> Option<(Type, Type)> {
    let Type::Path(type_path) = ty else {
        return None;
    };
    let segment = type_path.path.segments.last()?;
    if !names.iter().any(|name| segment.ident == *name) {
        return None;
    }

    let PathArguments::AngleBracketed(args) = &segment.arguments else {
        return None;
    };

    let mut types = args.args.iter().filter_map(|arg| match arg {
        GenericArgument::Type(inner) => Some(inner.clone()),
        _ => None,
    });

    let value_ty = types.next()?;
    let serializer_ty = types.next()?;
    Some((value_ty, serializer_ty))
}

/* -------- module parser -------- */

struct ModuleArgs {
    imports: Vec<Type>,
    controllers: Vec<Type>,
    providers: Vec<Expr>,
    exports: Vec<Type>,
    on_module_init: Vec<Expr>,
    on_module_destroy: Vec<Expr>,
    on_application_bootstrap: Vec<Expr>,
    on_application_shutdown: Vec<Expr>,
    global: bool,
}

#[derive(Default)]
struct InjectableArgs {
    factory: Option<Expr>,
}

struct RouteResponseArgs {
    status: u16,
    description: LitStr,
}

struct EntityArgs {
    table: LitStr,
}

impl Parse for EntityArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let key: Ident = input.parse()?;
        if key != "table" {
            return Err(syn::Error::new(
                key.span(),
                "Unsupported entity key. Use `table = \"...\"`.",
            ));
        }
        input.parse::<Token![=]>()?;
        let table = input.parse::<LitStr>()?;
        Ok(Self { table })
    }
}

impl Parse for RouteResponseArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut status = None;
        let mut description = None;

        while !input.is_empty() {
            let key: Ident = input.parse()?;
            input.parse::<Token![=]>()?;

            if key == "status" {
                let value = input.parse::<syn::LitInt>()?;
                status = Some(value.base10_parse()?);
            } else if key == "description" {
                description = Some(input.parse::<LitStr>()?);
            } else {
                return Err(syn::Error::new(
                    key.span(),
                    "Unsupported response key. Use `status = ...` and `description = \"...\"`.",
                ));
            }

            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(Self {
            status: status.ok_or_else(|| {
                syn::Error::new(input.span(), "response metadata requires `status = ...`")
            })?,
            description: description.ok_or_else(|| {
                syn::Error::new(
                    input.span(),
                    "response metadata requires `description = \"...\"`",
                )
            })?,
        })
    }
}

impl Parse for ModuleArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut imports: Vec<Type> = Vec::new();
        let mut controllers: Vec<Type> = Vec::new();
        let mut providers: Vec<Expr> = Vec::new();
        let mut exports: Vec<Type> = Vec::new();
        let mut on_module_init: Vec<Expr> = Vec::new();
        let mut on_module_destroy: Vec<Expr> = Vec::new();
        let mut on_application_bootstrap: Vec<Expr> = Vec::new();
        let mut on_application_shutdown: Vec<Expr> = Vec::new();
        let mut global = false;

        while !input.is_empty() {
            let key: Ident = input.parse()?;
            input.parse::<Token![=]>()?;

            if key == "imports" {
                imports = parse_bracket_list::<Type>(input)?;
            } else if key == "controllers" {
                controllers = parse_bracket_list::<Type>(input)?;
            } else if key == "providers" {
                providers = parse_bracket_list::<Expr>(input)?;
            } else if key == "exports" {
                exports = parse_bracket_list::<Type>(input)?;
            } else if key == "on_module_init" {
                on_module_init = parse_bracket_list::<Expr>(input)?;
            } else if key == "on_module_destroy" {
                on_module_destroy = parse_bracket_list::<Expr>(input)?;
            } else if key == "on_application_bootstrap" {
                on_application_bootstrap = parse_bracket_list::<Expr>(input)?;
            } else if key == "on_application_shutdown" {
                on_application_shutdown = parse_bracket_list::<Expr>(input)?;
            } else if key == "global" {
                let lit: syn::LitBool = input.parse()?;
                global = lit.value;
            } else {
                return Err(syn::Error::new(
                    key.span(),
                    "Unsupported module key. Use `imports`, `controllers`, `providers`, `exports`, lifecycle hook lists, or `global`.",
                ));
            }

            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(Self {
            imports,
            controllers,
            providers,
            exports,
            on_module_init,
            on_module_destroy,
            on_application_bootstrap,
            on_application_shutdown,
            global,
        })
    }
}

impl Parse for InjectableArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.is_empty() {
            return Ok(Self::default());
        }

        let key: Ident = input.parse()?;
        if key != "factory" {
            return Err(syn::Error::new(
                key.span(),
                "Unsupported injectable key. Use `factory = some_fn`.",
            ));
        }
        input.parse::<Token![=]>()?;
        let factory = input.parse::<Expr>()?;

        if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
        }

        if !input.is_empty() {
            return Err(syn::Error::new(
                input.span(),
                "Unexpected tokens in #[injectable(...)]",
            ));
        }

        Ok(Self {
            factory: Some(factory),
        })
    }
}

fn parse_bracket_list<T>(input: ParseStream) -> syn::Result<Vec<T>>
where
    T: Parse,
{
    let content;
    bracketed!(content in input);

    let items: Punctuated<T, Token![,]> = content.parse_terminated(T::parse, Token![,])?;
    Ok(items.into_iter().collect())
}

fn build_provider_registration(expr: &Expr) -> TokenStream2 {
    if is_provider_builder_expr(expr) {
        quote! { nestforge::register_provider(container, #expr)?; }
    } else if let Some(ty) = injectable_type_expr(expr) {
        quote! { nestforge::register_injectable::<#ty>(container)?; }
    } else {
        quote! { nestforge::register_provider(container, nestforge::Provider::value(#expr))?; }
    }
}

fn is_provider_builder_expr(expr: &Expr) -> bool {
    let Expr::Call(call) = expr else {
        return false;
    };
    let Expr::Path(path_expr) = call.func.as_ref() else {
        return false;
    };

    let mut segments = path_expr.path.segments.iter().rev();
    let Some(method) = segments.next() else {
        return false;
    };

    if method.ident != "value" && method.ident != "factory" {
        return false;
    }

    let Some(provider) = segments.next() else {
        return false;
    };

    provider.ident == "Provider"
}

fn injectable_type_expr(expr: &Expr) -> Option<Type> {
    let Expr::Path(path) = expr else {
        return None;
    };

    Some(Type::Path(syn::TypePath {
        qself: None,
        path: path.path.clone(),
    }))
}

fn ensure_derive_trait(attrs: &mut Vec<Attribute>, trait_name: &str) {
    for attr in attrs.iter_mut() {
        if !attr.path().is_ident("derive") {
            continue;
        }

        let Ok(mut derives) =
            attr.parse_args_with(Punctuated::<syn::Path, Token![,]>::parse_terminated)
        else {
            continue;
        };

        if derives.iter().any(|path| path.is_ident(trait_name)) {
            return;
        }

        derives.push(parse_quote!(Clone));
        *attr = parse_quote!(#[derive(#derives)]);
        return;
    }

    attrs.push(parse_quote!(#[derive(Clone)]));
}

fn extract_id_field(fields: &mut Fields) -> Option<(Ident, Type)> {
    let Fields::Named(named_fields) = fields else {
        return None;
    };

    let mut found: Option<(Ident, Type)> = None;

    for field in &mut named_fields.named {
        let has_id_attr = remove_id_attr(field);
        if !has_id_attr {
            continue;
        }

        let field_name = field.ident.clone()?;
        let field_ty = field.ty.clone();

        if found.is_some() {
            return None;
        }

        found = Some((field_name, field_ty));
    }

    found
}

fn remove_id_attr(field: &mut Field) -> bool {
    let mut kept = Vec::new();
    let mut has_id = false;

    for attr in field.attrs.drain(..) {
        let is_id = attr
            .path()
            .segments
            .last()
            .map(|seg| seg.ident == "id")
            .unwrap_or(false);
        if is_id {
            has_id = true;
        } else {
            kept.push(attr);
        }
    }

    field.attrs = kept;
    has_id
}

fn find_id_field(fields: &Fields) -> Option<(Ident, Type)> {
    let Fields::Named(named_fields) = fields else {
        return None;
    };

    let mut by_attr: Option<(Ident, Type)> = None;
    let mut by_name: Option<(Ident, Type)> = None;

    for field in &named_fields.named {
        let field_ident = field.ident.clone()?;
        if field_ident == "id" {
            by_name = Some((field_ident.clone(), field.ty.clone()));
        }
        let has_id_attr = field.attrs.iter().any(|attr| {
            attr.path()
                .segments
                .last()
                .map(|s| s.ident == "id")
                .unwrap_or(false)
        });
        if has_id_attr {
            by_attr = Some((field_ident, field.ty.clone()));
        }
    }

    by_attr.or(by_name)
}

#[derive(Default)]
struct ValidateRules {
    required: bool,
    email: bool,
    min_length: Option<usize>,
    max_length: Option<usize>,
    min: Option<syn::Lit>,
    max: Option<syn::Lit>,
}

impl ValidateRules {
    fn has_rules(&self) -> bool {
        self.required
            || self.email
            || self.min_length.is_some()
            || self.max_length.is_some()
            || self.min.is_some()
            || self.max.is_some()
    }
}

fn parse_validate_rules(attrs: &[Attribute]) -> ValidateRules {
    let mut rules = ValidateRules::default();

    for attr in attrs {
        let is_validate = attr
            .path()
            .segments
            .last()
            .map(|seg| seg.ident == "validate")
            .unwrap_or(false);
        if !is_validate {
            continue;
        }

        let _ = attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("required") {
                rules.required = true;
            } else if meta.path.is_ident("email") {
                rules.email = true;
            } else if meta.path.is_ident("min_length") {
                let value = meta.value()?.parse::<syn::LitInt>()?;
                rules.min_length = Some(value.base10_parse()?);
            } else if meta.path.is_ident("max_length") {
                let value = meta.value()?.parse::<syn::LitInt>()?;
                rules.max_length = Some(value.base10_parse()?);
            } else if meta.path.is_ident("min") {
                rules.min = Some(meta.value()?.parse::<syn::Lit>()?);
            } else if meta.path.is_ident("max") {
                rules.max = Some(meta.value()?.parse::<syn::Lit>()?);
            }
            Ok(())
        });
    }

    rules
}

fn is_type_named(ty: &Type, name: &str) -> bool {
    match ty {
        Type::Path(tp) => tp.path.is_ident(name),
        _ => false,
    }
}

fn is_option_any(ty: &Type) -> bool {
    match ty {
        Type::Path(tp) => tp
            .path
            .segments
            .last()
            .map(|seg| seg.ident == "Option")
            .unwrap_or(false),
        _ => false,
    }
}

fn is_option_of(ty: &Type, inner_name: &str) -> bool {
    let Type::Path(tp) = ty else {
        return false;
    };
    let Some(seg) = tp.path.segments.last() else {
        return false;
    };
    if seg.ident != "Option" {
        return false;
    }
    let syn::PathArguments::AngleBracketed(args) = &seg.arguments else {
        return false;
    };
    let Some(syn::GenericArgument::Type(ref inner_ty)) = args.args.first() else {
        return false;
    };
    is_type_named(inner_ty, inner_name)
}

fn is_numeric_type(ty: &Type) -> bool {
    let Type::Path(tp) = ty else {
        return false;
    };
    tp.path.is_ident("u8")
        || tp.path.is_ident("u16")
        || tp.path.is_ident("u32")
        || tp.path.is_ident("u64")
        || tp.path.is_ident("usize")
        || tp.path.is_ident("i8")
        || tp.path.is_ident("i16")
        || tp.path.is_ident("i32")
        || tp.path.is_ident("i64")
        || tp.path.is_ident("isize")
        || tp.path.is_ident("f32")
        || tp.path.is_ident("f64")
}

fn is_option_numeric_type(ty: &Type) -> bool {
    let Type::Path(tp) = ty else {
        return false;
    };
    let Some(seg) = tp.path.segments.last() else {
        return false;
    };
    if seg.ident != "Option" {
        return false;
    }
    let syn::PathArguments::AngleBracketed(args) = &seg.arguments else {
        return false;
    };
    let Some(syn::GenericArgument::Type(ref inner_ty)) = args.args.first() else {
        return false;
    };
    is_numeric_type(inner_ty)
}

#[proc_macro_derive(Config, attributes(config, serde))]
pub fn derive_config(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = &input.generics.split_for_impl();

    let Data::Struct(data) = &input.data else {
        return syn::Error::new(input.ident.span(), "Config can only be derived on structs")
            .to_compile_error()
            .into();
    };

    let Fields::Named(fields) = &data.fields else {
        return syn::Error::new(input.ident.span(), "Config derive requires named fields")
            .to_compile_error()
            .into();
    };

    let mut env_lookups: Vec<TokenStream2> = Vec::new();
    let mut field_idents: Vec<Ident> = Vec::new();

    for field in &fields.named {
        let Some(field_ident) = &field.ident.clone() else {
            continue;
        };

        let env_key = field_ident.to_string().to_uppercase();
        let field_ty = &field.ty;
        let is_option = is_option_type(field_ty);

        let lookup_expr = if is_option {
            quote! {
                let #field_ident: #field_ty = <#field_ty as nestforge_config::ConfigField>::from_env(&env).ok();
            }
        } else {
            let default_tokens = match field_ty {
                Type::Path(tp) if tp.path.is_ident("String") => quote!(String::new()),
                Type::Path(tp) if tp.path.is_ident("u8") => quote!(0u8),
                Type::Path(tp) if tp.path.is_ident("u16") => quote!(0u16),
                Type::Path(tp) if tp.path.is_ident("u32") => quote!(0u32),
                Type::Path(tp) if tp.path.is_ident("u64") => quote!(0u64),
                Type::Path(tp) if tp.path.is_ident("usize") => quote!(0usize),
                Type::Path(tp) if tp.path.is_ident("i8") => quote!(0i8),
                Type::Path(tp) if tp.path.is_ident("i16") => quote!(0i16),
                Type::Path(tp) if tp.path.is_ident("i32") => quote!(0i32),
                Type::Path(tp) if tp.path.is_ident("i64") => quote!(0i64),
                Type::Path(tp) if tp.path.is_ident("isize") => quote!(0isize),
                Type::Path(tp) if tp.path.is_ident("f32") => quote!(0.0f32),
                Type::Path(tp) if tp.path.is_ident("f64") => quote!(0.0f64),
                Type::Path(tp) if tp.path.is_ident("bool") => quote!(false),
                _ => quote!(std::default::Default::default()),
            };
            quote! {
                let #field_ident: #field_ty = <#field_ty as nestforge_config::ConfigField>::from_env_or(&env, #env_key, #default_tokens)?;
            }
        };

        env_lookups.push(lookup_expr);
        field_idents.push(field_ident.clone());
    }

    let field_idents2 = field_idents.clone();

    let expanded = quote! {
        impl #impl_generics nestforge_config::FromEnv for #name #ty_generics #where_clause {
            fn from_env(env: &nestforge_config::EnvStore) -> Result<Self, nestforge_config::ConfigError> {
                #(#env_lookups)*

                Ok(Self {
                    #(#field_idents),*
                })
            }

            fn config_key() -> &'static str {
                stringify!(#name)
            }
        }

        impl #impl_generics std::default::Default for #name #ty_generics #where_clause {
            fn default() -> Self {
                Self {
                    #(#field_idents2: std::default::Default::default()),*
                }
            }
        }
    };

    TokenStream::from(expanded)
}

fn is_option_type(ty: &Type) -> bool {
    match ty {
        Type::Path(tp) => tp
            .path
            .segments
            .last()
            .map(|seg| seg.ident == "Option")
            .unwrap_or(false),
        _ => false,
    }
}
