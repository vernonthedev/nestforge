use proc_macro::TokenStream;
use quote::quote;
use syn::{
    bracketed,
    parse::{Parse, ParseStream},
    parse_macro_input, punctuated::Punctuated,
    Attribute, Expr, Ident, ImplItem, ImplItemFn, ItemImpl, ItemStruct, LitStr, Meta, Token, Type,
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
    controllers = [AppController, UsersController],
    providers = [AppConfig { ... }, UsersService::new()]
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

    let provider_regs = args.providers.iter().map(|expr| {
        quote! { container.register(#expr)?; }
    });

    let expanded = quote! {
        #input

        impl nestforge::ModuleDefinition for #name {
            fn register(container: &nestforge::Container) -> anyhow::Result<()> {
                #(#provider_regs)*
                Ok(())
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
    controllers: Vec<Type>,
    providers: Vec<Expr>,
}

impl Parse for ModuleArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut controllers: Vec<Type> = Vec::new();
        let mut providers: Vec<Expr> = Vec::new();

        while !input.is_empty() {
            let key: Ident = input.parse()?;
            input.parse::<Token![=]>()?;

            if key == "controllers" {
                controllers = parse_bracket_list::<Type>(input)?;
            } else if key == "providers" {
                providers = parse_bracket_list::<Expr>(input)?;
            } else {
                return Err(syn::Error::new(
                    key.span(),
                    "Unsupported module key. Use `controllers` or `providers`.",
                ));
            }

            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(Self {
            controllers,
            providers,
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