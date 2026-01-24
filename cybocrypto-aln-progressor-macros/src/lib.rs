#![forbid(unsafe_code)]

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse_macro_input, AttributeArgs, Ident, ItemStruct, Lit, Meta, MetaNameValue, NestedMeta,
};

#[proc_macro_attribute]
pub fn aln_progressor(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as AttributeArgs);
    let input_struct = parse_macro_input!(item as ItemStruct);

    let struct_ident: Ident = input_struct.ident.clone();

    let mut input_ty: Option<String> = None;
    let mut output_ty: Option<String> = None;
    let mut anchor_ty: Option<String> = None;

    for arg in args {
        if let NestedMeta::Meta(Meta::NameValue(MetaNameValue {
            path,
            lit: Lit::Str(lit_str),
            ..
        })) = arg
        {
            let key = path.get_ident().map(|id| id.to_string());
            match key.as_deref() {
                Some("input") => input_ty = Some(lit_str.value()),
                Some("output") => output_ty = Some(lit_str.value()),
                Some("anchor") => anchor_ty = Some(lit_str.value()),
                _ => {}
            }
        }
    }

    let input_ty = match input_ty {
        Some(v) => v,
        None => {
            return syn::Error::new_spanned(
                &input_struct,
                "aln_progressor: missing required argument `input = \"Type\"`",
            )
            .to_compile_error()
            .into();
        }
    };

    let output_ty = match output_ty {
        Some(v) => v,
        None => {
            return syn::Error::new_spanned(
                &input_struct,
                "aln_progressor: missing required argument `output = \"Type\"`",
            )
            .to_compile_error()
            .into();
        }
    };

    let anchor_ty = match anchor_ty {
        Some(v) => v,
        None => {
            return syn::Error::new_spanned(
                &input_struct,
                "aln_progressor: missing required argument `anchor = \"Type\"`",
            )
            .to_compile_error()
            .into();
        }
    };

    let input_ty_ident: syn::Type = match syn::parse_str(&input_ty) {
        Ok(t) => t,
        Err(e) => {
            return syn::Error::new_spanned(
                &input_struct,
                format!("aln_progressor: failed to parse `input` type: {e}"),
            )
            .to_compile_error()
            .into();
        }
    };

    let output_ty_ident: syn::Type = match syn::parse_str(&output_ty) {
        Ok(t) => t,
        Err(e) => {
            return syn::Error::new_spanned(
                &input_struct,
                format!("aln_progressor: failed to parse `output` type: {e}"),
            )
            .to_compile_error()
            .into();
        }
    };

    let anchor_ty_ident: syn::Type = match syn::parse_str(&anchor_ty) {
        Ok(t) => t,
        Err(e) => {
            return syn::Error::new_spanned(
                &input_struct,
                format!("aln_progressor: failed to parse `anchor` type: {e}"),
            )
            .to_compile_error()
            .into();
        }
    };

    let provenance_field_ident = format_ident!("provenance");
    let method_ident = format_ident!("progress_with_host");

    let expanded = quote! {
        #input_struct

        impl #struct_ident {
            /// Helper method to evolve one step given host budget.
            pub fn #method_ident(
                &mut self,
                input: #input_ty_ident,
                host: &cybocrypto_aln_core::HostBudget,
            ) -> Result<(#output_ty_ident, cybocrypto_aln_core::ProgressStamp), String> {
                use cybocrypto_aln_core::ProgressOnce;
                self.progress_once(input, host)
                    .map_err(|e| format!("{:?}", e))
            }
        }

        impl cybocrypto_aln_core::TraceProvenance for #struct_ident {
            type ProvenanceError = String;

            fn attach_provenance(
                &mut self,
                provenance: cybocrypto_aln_core::Provenance,
            ) -> Result<(), Self::ProvenanceError> {
                self.#provenance_field_ident = Some(provenance);
                Ok(())
            }

            fn verify_provenance(
                &self,
            ) -> Result<&cybocrypto_aln_core::Provenance, Self::ProvenanceError> {
                self.#provenance_field_ident
                    .as_ref()
                    .ok_or_else(|| "missing provenance".to_string())
            }
        }

        impl cybocrypto_aln_core::ProgressOnce for #struct_ident {
            type Input = #input_ty_ident;
            type Output = #output_ty_ident;
            type Error = String;

            fn progress_once(
                &mut self,
                input: Self::Input,
                host: &cybocrypto_aln_core::HostBudget,
            ) -> Result<(Self::Output, cybocrypto_aln_core::ProgressStamp), Self::Error> {
                use cybocrypto_aln_core::BioscaleSafe;

                if !input.is_bioscale_safe(host) {
                    return Err("bioscale safety check failed".into());
                }

                let now = std::time::SystemTime::now();

                let (step_id, prev_hash) = if let Some(last) = self.last_stamp.as_ref() {
                    (last.step_id + 1, Some(last.new_hash.clone()))
                } else {
                    (0, None)
                };

                let stamp = cybocrypto_aln_core::ProgressStamp {
                    step_id,
                    prev_hash,
                    new_hash: vec![0u8; 32],
                    timestamp: now,
                };

                self.last_stamp = Some(stamp.clone());

                // Call the domain-specific builder; the struct MUST define:
                // fn build_output(&self, input: &Input, host: &HostBudget) -> Output
                let out = self.build_output(&input, host);

                Ok((out, stamp))
            }
        }

        impl #struct_ident {
            /// Convenience helper to anchor a given stamp and evidence into a ledger instance.
            pub fn anchor_into_ledger(
                &self,
                ledger: &mut #anchor_ty_ident,
                stamp: &cybocrypto_aln_core::ProgressStamp,
                evidence: &cybocrypto_aln_core::EvidenceBundle,
            ) -> Result<(), <#anchor_ty_ident as cybocrypto_aln_core::AnchorToLedger>::AnchorError>
            where
                #anchor_ty_ident: cybocrypto_aln_core::AnchorToLedger,
            {
                ledger.anchor_to_ledger(stamp, evidence)
            }
        }
    };

    TokenStream::from(expanded)
}
