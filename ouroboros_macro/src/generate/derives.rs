use crate::info_structures::{Derive, StructInfo};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Error, GenericParam, TypeParamBound};

fn add_trait_bound(param: &GenericParam, bound: &TypeParamBound) -> GenericParam {
    let mut new = param.clone();
    match &mut new {
        GenericParam::Type(t) => t.bounds.push(bound.clone()),
        _ => (),
    }
    new
}

fn impl_trait(info: &StructInfo, trait_name: TypeParamBound, body: TokenStream) -> TokenStream {
    let generic_params = info.generic_params();
    let generic_params = generic_params
        .into_iter()
        .map(|i| add_trait_bound(i, &trait_name))
        .collect::<Vec<_>>();
    let generic_args = info.generic_arguments();
    let generic_where = &info.generics.where_clause;
    let struct_name = &info.ident;
    quote! {
        impl <#(#generic_params),*> #trait_name for #struct_name <#(#generic_args),*> #generic_where {
            #body
        }
    }
}

fn impl_debug(info: &StructInfo) -> Result<TokenStream, Error> {
    let fields = info
        .fields
        .iter()
        .filter(|field| !field.is_mutably_borrowed())
        .map(|field| {
            let name = &field.name;
            quote! {
                field(stringify!(#name), &self.#name)
            }
        })
        .collect::<Vec<_>>();
    let trait_name = syn::parse_quote! { ::std::fmt::Debug };
    let struct_name = &info.ident;
    let body = quote! {
        fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
            f.debug_struct(stringify!(#struct_name))
            #(.#fields)*
            .finish()?;
            Ok(())
        }
    };
    Ok(impl_trait(info, trait_name, body))
}

fn impl_partial_eq(info: &StructInfo) -> Result<TokenStream, Error> {
    let fields = info
        .fields
        .iter()
        .filter(|field| !field.is_mutably_borrowed())
        .map(|field| {
            let name = &field.name;
            quote! {
                &*self.#name == &*other.#name
            }
        })
        .collect::<Vec<_>>();
    let trait_name = syn::parse_quote! { ::core::cmp::PartialEq };
    let body = quote! {
        fn eq(&self, other: &Self) -> bool {
            #(#fields)&&*
        }
    };
    Ok(impl_trait(info, trait_name, body))
}

fn impl_eq(info: &StructInfo) -> Result<TokenStream, Error> {
    let trait_name = syn::parse_quote! { ::core::cmp::Eq };
    let body = quote! {};
    Ok(impl_trait(info, trait_name, body))
}

pub fn create_derives(info: &StructInfo) -> Result<TokenStream, Error> {
    let mut impls = Vec::new();
    for derive in &info.derives {
        match derive {
            Derive::Debug => impls.push(impl_debug(info)?),
            Derive::PartialEq => impls.push(impl_partial_eq(info)?),
            Derive::Eq => impls.push(impl_eq(info)?),
        }
    }
    Ok(quote! { #(#impls)* })
}
