use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
    bracketed,
    parse::{Parse, ParseStream},
    parse_macro_input, punctuated::Punctuated,
    Attribute, Expr, Field, Fields, Ident, ImplItem, ImplItemFn, ItemImpl, ItemStruct, LitStr, Meta, Token, Type,
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
pub fn use_guard(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn use_interceptor(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
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

    if ident != "get" && ident != "post" && ident != "put" {
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
