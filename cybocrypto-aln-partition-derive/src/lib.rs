#![forbid(unsafe_code)]

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse_macro_input, Attribute, Data, DataStruct, DeriveInput, Field, Fields, Ident, Meta,
};

#[proc_macro_derive(AlnPartition, attributes(aln))]
pub fn derive_aln_partition(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident.clone();

    let data_struct = match input.data {
        Data::Struct(ds) => ds,
        _ => {
            return syn::Error::new_spanned(
                input,
                "AlnPartition can only be derived for structs",
            )
            .to_compile_error()
            .into();
        }
    };

    let (commit_fields, commit_types): (Vec<Ident>, Vec<syn::Type>) =
        collect_partitioned_fields(&data_struct, "commit");
    let (local_fields, local_types): (Vec<Ident>, Vec<syn::Type>) =
        collect_partitioned_fields(&data_struct, "local");
    let (ephemeral_fields, ephemeral_types): (Vec<Ident>, Vec<syn::Type>) =
        collect_partitioned_fields(&data_struct, "ephemeral");

    let commit_view_name = format_ident!("{}CommitView", name);
    let local_view_name = format_ident!("{}LocalView", name);
    let ephemeral_view_name = format_ident!("{}EphemeralView", name);

    let commit_view = if commit_fields.is_empty() {
        quote! {}
    } else {
        quote! {
            #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
            pub struct #commit_view_name {
                #( pub #commit_fields: #commit_types, )*
            }
        }
    };

    let local_view = if local_fields.is_empty() {
        quote! {}
    } else {
        quote! {
            #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
            pub struct #local_view_name {
                #( pub #local_fields: #local_types, )*
            }
        }
    };

    let ephemeral_view = if ephemeral_fields.is_empty() {
        quote! {}
    } else {
        quote! {
            #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
            pub struct #ephemeral_view_name {
                #( pub #ephemeral_fields: #ephemeral_types, )*
            }
        }
    };

    let to_commit_view_impl = if commit_fields.is_empty() {
        quote! {
            impl #name {
                pub fn to_commit_view(&self) -> () { () }
            }
        }
    } else {
        quote! {
            impl #name {
                pub fn to_commit_view(&self) -> #commit_view_name {
                    #commit_view_name {
                        #( #commit_fields: self.#commit_fields.clone(), )*
                    }
                }
            }
        }
    };

    let to_local_view_impl = if local_fields.is_empty() {
        quote! {}
    } else {
        quote! {
            impl #name {
                pub fn to_local_view(&self) -> #local_view_name {
                    #local_view_name {
                        #( #local_fields: self.#local_fields.clone(), )*
                    }
                }
            }
        }
    };

    let to_ephemeral_view_impl = if ephemeral_fields.is_empty() {
        quote! {}
    } else {
        quote! {
            impl #name {
                pub fn to_ephemeral_view(&self) -> #ephemeral_view_name {
                    #ephemeral_view_name {
                        #( #ephemeral_fields: self.#ephemeral_fields.clone(), )*
                    }
                }
            }
        }
    };

    let expanded = quote! {
        #commit_view
        #local_view
        #ephemeral_view

        #to_commit_view_impl
        #to_local_view_impl
        #to_ephemeral_view_impl
    };

    TokenStream::from(expanded)
}

fn collect_partitioned_fields(
    data_struct: &DataStruct,
    mode: &str,
) -> (Vec<Ident>, Vec<syn::Type>) {
    let mut idents = Vec::new();
    let mut types = Vec::new();

    let fields_iter = match &data_struct.fields {
        Fields::Named(named) => named.named.iter(),
        _ => return (idents, types),
    };

    for field in fields_iter {
        if has_aln_attr(field, mode) {
            if let Some(ident) = field.ident.clone() {
                idents.push(ident);
                types.push(field.ty.clone());
            }
        }
    }

    (idents, types)
}

fn has_aln_attr(field: &Field, mode: &str) -> bool {
    field.attrs.iter().any(|attr| is_matching_aln_attr(attr, mode))
}

fn is_matching_aln_attr(attr: &Attribute, mode: &str) -> bool {
    if !attr.path().is_ident("aln") {
        return false;
    }

    match attr.parse_meta() {
        Ok(Meta::List(list)) => {
            for nested in &list.nested {
                if let syn::NestedMeta::Meta(Meta::Path(path)) = nested {
                    if path.is_ident(mode) {
                        return true;
                    }
                }
            }
            false
        }
        _ => false,
    }
}
