use proc_macro::TokenStream;
use quote::{format_ident, quote, quote_spanned};
use syn::{
    Data, DataStruct, DeriveInput, Expr, ExprAssign, ExprLit, Fields, FieldsNamed, Ident, ItemFn,
    Lit, LitStr, Token, punctuated::Punctuated, spanned::Spanned,
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

#[proc_macro_attribute]
pub fn function(_attr: TokenStream, token_stream: TokenStream) -> TokenStream {
    let fun = syn::parse_macro_input!(token_stream as ItemFn);
    let name = &fun.sig.ident;
    let vis = &fun.vis;

    let args = &fun.sig.inputs;
    let output = &fun.sig.output;
    let body = &fun.block.stmts;

    let sig_return = match output {
        syn::ReturnType::Default => quote! { ::core::any::TypeId::of::<()>() },
        syn::ReturnType::Type(_, rtype) => quote! { ::core::any::TypeId::of::<#rtype>() },
    };

    let sig_args = args.iter().filter_map(|elm| match elm {
        syn::FnArg::Receiver(_) => None,
        syn::FnArg::Typed(pat_type) => {
            let ty = &pat_type.ty;
            Some(quote_spanned! {pat_type.span()=>::core::any::TypeId::of::<#ty>()})
        }
    });
    let call_args = args
        .iter()
        .filter_map(|elm| match elm {
            syn::FnArg::Receiver(_) => None,
            syn::FnArg::Typed(pat_type) => Some(pat_type),
        })
        .enumerate()
        .map(|(i, pat_type)| {
            let name = format_ident!("arg_{i}");
            let ty = &pat_type.ty;
            Some(quote_spanned! {pat_type.span()=>let #name: #ty = args[#i].get_value_as(engine);})
        });

    let mut next = 0;
    let invoke_args = args.iter().map(|elm| match elm {
        syn::FnArg::Receiver(_) => quote! {self},
        syn::FnArg::Typed(pat_type) => {
            let name = format_ident!("arg_{next}");
            next += 1;
            quote_spanned! {pat_type.span()=>#name}
        }
    });
    let make_args = args
        .iter()
        .filter_map(|elm| match elm {
            syn::FnArg::Receiver(_) => None,
            syn::FnArg::Typed(pat_type) => Some(pat_type),
        })
        .enumerate()
        .map(|(i, pat_type)| {
            let ty = &pat_type.ty;
            Some(quote_spanned! {pat_type.span()=>engine.make_value::<#ty>(&values[#i]).to_any()})
        });

    quote_spanned! {fun.span() =>
        #[allow(non_camel_case_types)]
        #vis struct #name;
        #[allow(non_camel_case_types, dead_code, redundant_semicolons)]
        impl #name {
            fn call(#args) #output {
                #(#body)*
            }
        }
        #[allow(non_camel_case_types, dead_code, redundant_semicolons)]
        impl ::skaf::Function for #name {
            fn name() -> &'static str
            where
                Self: Sized
            {
                stringify!(#name)
            }

            fn sig(&self) -> (Vec<::core::any::TypeId>, ::core::any::TypeId) {
                (vec![#(#sig_args),*], #sig_return)
            }

            fn call(&self, engine: &::skaf::Engine, args: &::std::vec::Vec<::skaf::Proxy<Box<dyn ::core::any::Any>>>) -> Box<dyn ::core::any::Any> {
                #(#call_args)*;
                Box::new(Self::call(#(#invoke_args),*))
            }

            fn make_args(
                &self,
                values: &::std::vec::Vec<skaf::parser::Value>,
                engine: &::skaf::Engine
            ) -> ::std::vec::Vec<::skaf::Proxy<Box<dyn ::core::any::Any>>>
            {
                vec![#(#make_args),*]
            }
        }
    }
    .into()
}
