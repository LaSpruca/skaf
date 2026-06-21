use proc_macro::TokenStream;
use quote::{format_ident, quote, quote_spanned};
use syn::{
    Data, DataStruct, DeriveInput, Expr, ExprAssign, ExprLit, Fields, FieldsNamed, Ident, Lit,
    LitStr, Token, punctuated::Punctuated, spanned::Spanned,
};

#[proc_macro_derive(StructureType, attributes(skaf))]
pub fn structure_type(stream: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(stream as DeriveInput);

    let mut name = None;

    for attr in input.attrs.iter() {
        match attr.meta {
            syn::Meta::List(ref meta_list) if meta_list.path.is_ident("skaf") => {
                let args = match meta_list
                    .parse_args_with(Punctuated::<ExprAssign, Token![,]>::parse_terminated)
                {
                    Ok(val) => val,
                    Err(ex) => return ex.into_compile_error().into(),
                };

                for item in args.iter() {
                    let arg_name = match *item.left {
                        Expr::Path(ref p) => p,
                        _ => {
                            return quote_spanned! { item.right.span() =>
                                compile_error!("Arg names must be an identifier", )
                            }
                            .into();
                        }
                    };

                    if arg_name.path.is_ident("name") {
                        match *item.right {
                            Expr::Lit(ExprLit {
                                lit: Lit::Str(ref s),
                                ..
                            }) => name = Some(s.value()),
                            _ => {
                                return quote_spanned! { item.right.span() =>
                                    compile_error!("`name` must be a string")
                                }
                                .into();
                            }
                        }
                    } else {
                        let name = arg_name.path.get_ident().map(|x| x.to_string());
                        return quote_spanned! {arg_name.span()=>
                            compile_error!("Unrecoginised argument {}", stringify!(#name))
                        }
                        .into();
                    }
                }
            }
            _ => {}
        }
    }

    let name = name.unwrap_or_else(|| input.ident.to_string());

    let input_span = input.span();
    let fields = match input.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(ref fields),
            ..
        }) => fields.clone(),
        _ => {
            return quote_spanned! {input_span=>compile_error!("Only named structs are supported")}
                .into();
        }
    };

    let proxy = create_proxy(&input.ident, &fields);
    let imp = create_structure_type(&input.ident, &fields, name.as_str());

    quote! {
        #imp
        #proxy
    }
    .into()
}

fn create_structure_type(
    struct_name: &Ident,
    input: &FieldsNamed,
    name: &str,
) -> proc_macro2::TokenStream {
    let proxy_type = format_ident!("{}Proxy", struct_name);
    let name = Lit::Str(LitStr::new(name, input.span()));
    let make_proxy = input.named.iter().map(|x| {
        let ident = &x.ident;
        quote_spanned! {x.span()=>#ident: engine.make_value(object.get_field(stringify!(#ident)).expect("Unreachable"))}
    });
    let get_structure = input.named.iter().map(|x| {
        let ident = &x.ident;
        let ty = &x.ty;
        quote_spanned! {x.span()=>.field::<#ty>(stringify!(#ident))}
    });
    let make = input.named.iter().map(|x| {
        let ident = &x.ident;
        quote_spanned! {x.span()=>#ident: proxy.#ident.get_value(engine),}
    });

    quote! {
        impl ::skaf::StructureType for #struct_name {
            type ProxyType = #proxy_type;

            fn tag() -> &'static str {
                #name
            }

            fn make_proxy(object: ::skaf::parser::Object, engine: &::skaf::Engine) -> Self::ProxyType {
                Self::ProxyType {
                    #(#make_proxy),*
                }
            }

            fn get_structure() -> ::skaf::Structure {
                ::skaf::Structure::builder()
                    #(#get_structure)*
                    .build()
            }
            fn make(proxy: &Self::ProxyType, engine: &::skaf::Engine) -> Self {
                Self {
                    #(#make)*
                }
            }
        }
    }
}

fn create_proxy(struct_name: &Ident, input: &FieldsNamed) -> proc_macro2::TokenStream {
    let struct_fields = input.named.iter().map(|x| {
        let ident = &x.ident;
        let ty = &x.ty;
        quote_spanned! {x.span()=>#ident: ::skaf::Proxy<#ty>}
    });

    let proxy_attrs = input.named.iter().map(|x| {
        let ident = &x.ident;
        quote_spanned! {x.span()=>stringify!(#ident) => {Some(Box::new(self.#ident.get_value(engine)))}}
    });

    let new_name = format_ident!("{}Proxy", struct_name);

    quote! {
        #[allow(dead_code)]
        struct #new_name {
            #(#struct_fields),*
        }

        impl ::skaf::StructureProxy for #new_name {
            fn get(&self, path: &str, engine: &::skaf::Engine) -> ::core::option::Option<Box<dyn ::core::any::Any>> {
                match path {
                    #(#proxy_attrs)*
                    _ => None
                }
            }
        }
    }
}
