use proc_macro::TokenStream;
use quote::{quote, format_ident};
use syn::{parse_macro_input, DeriveInput, Data, Fields, Attribute};

#[proc_macro_derive(AlnPartition, attributes(aln))]
pub fn derive_aln_partition(input: TokenStream) -> TokenStream {
    let input_ast = parse_macro_input!(input as DeriveInput);
    let name = input_ast.ident.clone();

    let mut commit_fields = Vec::new();
    let mut local_fields = Vec::new();
    let mut ephemeral_fields = Vec::new();

    if let Data::Struct(data_struct) = input_ast.data {
        if let Fields::Named(named) = data_struct.fields {
            for field in named.named {
                let field_name = field.ident.unwrap();
                let field_ty = field.ty;
                let attrs: &Vec<Attribute> = &field.attrs;

                let mut is_commit = false;
                let mut is_local = false;
                let mut is_ephemeral = false;

                for attr in attrs {
                    if attr.path().is_ident("aln") {
                        let meta = attr.meta.clone();
                        if let syn::Meta::List(list) = meta {
                            for nested in list.nested {
                                if let syn::NestedMeta::Meta(syn::Meta::Path(path)) = nested {
                                    if path.is_ident("commit") {
                                        is_commit = true;
                                    } else if path.is_ident("local") {
                                        is_local = true;
                                    } else if path.is_ident("ephemeral") {
                                        is_ephemeral = true;
                                    }
                                }
                            }
                        }
                    }
                }

                if is_commit {
                    commit_fields.push(quote! { pub #field_name: #field_ty });
                } else if is_local {
                    local_fields.push(quote! { pub #field_name: #field_ty });
                } else if is_ephemeral {
                    ephemeral_fields.push(quote! { pub #field_name: #field_ty });
                }
            }
        }
    }

    let commit_name = format_ident!("{}CommitView", name);
    let local_name = format_ident!("{}LocalView", name);
    let ephemeral_name = format_ident!("{}EphemeralView", name);

    let expanded = quote! {
        #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
        pub struct #commit_name {
            #(#commit_fields,)*
        }

        #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
        pub struct #local_name {
            #(#local_fields,)*
        }

        #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
        pub struct #ephemeral_name {
            #(#ephemeral_fields,)*
        }

        impl #name {
            pub fn to_commit_view(&self) -> #commit_name {
                #commit_name {
                    // Fill from self.* in a later refinement
                    // For now, this derive focuses on generating the types.
                }
            }
        }
    };

    TokenStream::from(expanded)
}
