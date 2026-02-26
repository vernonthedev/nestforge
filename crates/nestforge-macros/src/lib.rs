use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
    bracketed,
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    spanned::Spanned,
    parse_quote, Attribute, Data, DeriveInput, Expr, Field, Fields, Ident, ImplItem, ImplItemFn,
    ItemImpl, ItemStruct, LitStr, Meta, Token, Type,
};

/*
#[controller("/users")]
Adds ControllerBasePath metadata to the struct.
*/
#[proc_macro_attribute]
pub fn controller(attr: TokenStream, item: TokenStream) -> TokenStream {
    let base_path = parse_macro_input!(attr as LitStr);
    let input = parse_macro_input!(item as ItemStruct);

    let name = &input.ident;
    let path = base_path.value();

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

/*
#[routes]
Reads #[get], #[post], #[put] on methods inside the impl block,
removes those attributes, and generates ControllerDefinition::router().
*/
#[proc_macro_attribute]
pub fn routes(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as ItemImpl);

    let self_ty = input.self_ty.clone();

    let mut route_calls = Vec::new();

    for impl_item in &mut input.items {
        let ImplItem::Fn(method) = impl_item else {
            continue;
        };

        if let Some((http_method, path)) = extract_route_meta(method) {
            let method_name = &method.sig.ident;
            let path_lit = LitStr::new(&path, method.sig.ident.span());

            let call = match http_method.as_str() {
                "get" => quote! { builder = builder.get(#path_lit, Self::#method_name); },
                "post" => quote! { builder = builder.post(#path_lit, Self::#method_name); },
                "put" => quote! { builder = builder.put(#path_lit, Self::#method_name); },
                "delete" => quote! { builder = builder.delete(#path_lit, Self::#method_name); },
                _ => continue,
            };

            route_calls.push(call);
        }
    }

    let expanded = quote! {
        #input

        impl nestforge::ControllerDefinition for #self_ty {
            fn router() -> axum::Router<nestforge::Container> {
                let mut builder = nestforge::RouteBuilder::<#self_ty>::new();
                #(#route_calls)*
                builder.build()
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

    let provider_regs = args.providers.iter().map(build_provider_registration);

    let import_refs = args.imports.iter().map(|ty| {
        quote! { nestforge::ModuleRef::of::<#ty>() }
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
pub fn use_guard(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn use_interceptor(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn dto(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as ItemStruct);

    input.attrs.push(parse_quote!(
        #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, nestforge::Validate)]
    ));

    TokenStream::from(quote! { #input })
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
        return syn::Error::new(id_field_ty.span(), "identifiable id field must be of type `u64`")
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

    input
        .attrs
        .push(parse_quote!(#[derive(Debug, Clone, serde::Serialize)]));

    TokenStream::from(quote! { #input })
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
        return syn::Error::new(id_field_ty.span(), "entity_dto id field must be of type `u64`")
            .to_compile_error()
            .into();
    }

    input.attrs.push(parse_quote!(
        #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, nestforge::Validate)]
    ));

    let name = &input.ident;

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
        let (required, email) = parse_validate_rules(&field.attrs);
        if !(required || email) {
            continue;
        }

        let is_string = is_type_named(&field.ty, "String");
        let is_option_string = is_option_of(&field.ty, "String");
        let is_option_any = is_option_any(&field.ty);

        if required {
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

        if email {
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

/* -------- module parser -------- */

struct ModuleArgs {
    imports: Vec<Type>,
    controllers: Vec<Type>,
    providers: Vec<Expr>,
    exports: Vec<Type>,
    global: bool,
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

impl Parse for ModuleArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut imports: Vec<Type> = Vec::new();
        let mut controllers: Vec<Type> = Vec::new();
        let mut providers: Vec<Expr> = Vec::new();
        let mut exports: Vec<Type> = Vec::new();
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
            } else if key == "global" {
                let lit: syn::LitBool = input.parse()?;
                global = lit.value;
            } else {
                return Err(syn::Error::new(
                    key.span(),
                    "Unsupported module key. Use `imports`, `controllers`, `providers`, `exports`, or `global`.",
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
            global,
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

fn parse_validate_rules(attrs: &[Attribute]) -> (bool, bool) {
    let mut required = false;
    let mut email = false;

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
                required = true;
            } else if meta.path.is_ident("email") {
                email = true;
            }
            Ok(())
        });
    }

    (required, email)
}

fn is_type_named(ty: &Type, name: &str) -> bool {
    matches!(ty, Type::Path(tp) if tp.path.is_ident(name))
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
    let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first() else {
        return false;
    };
    is_type_named(inner_ty, inner_name)
}
