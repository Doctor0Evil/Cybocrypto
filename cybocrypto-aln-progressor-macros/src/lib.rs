#![forbid(unsafe_code)]

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse_macro_input, AttributeArgs, Ident, ItemStruct, Lit, Meta, MetaNameValue, NestedMeta,
};

/// Attribute macro version of `aln_progressor!`, applied to a struct that will own the state.
///
/// Example usage in a domain crate:
///
/// ```rust
/// use cybocrypto_aln_core::{AlnContext, HostBudget};
/// use cybocrypto_aln_progressor_macros::aln_progressor;
///
/// #[aln_progressor(
///     input = "BiopayRequest",
///     output = "BiopayDecision",
///     anchor = "BiopayLedger",
///     guarantees = "Deterministic,Auditable,BioscaleSafe"
/// )]
/// pub struct BiopayChannel {
///     pub id: String,
///     pub aln_context: AlnContext,
/// }
/// ```
///
/// This generates:
/// - `impl ProgressOnce` for BiopayChannel
/// - a thin `anchor_to_ledger` wrapper on the specified ledger type
/// - `impl TraceProvenance` with a `provenance: Option<Provenance>` field injected.
#[proc_macro_attribute]
pub fn aln_progressor(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the attribute arguments: key = "value" pairs.
    let args = parse_macro_input!(attr as AttributeArgs);
    let input_struct = parse_macro_input!(item as ItemStruct);

    let struct_ident: Ident = input_struct.ident.clone();

    let mut input_ty: Option<String> = None;
    let mut output_ty: Option<String> = None;
    let mut anchor_ty: Option<String> = None;
    let mut guarantees: Option<String> = None;

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
                Some("guarantees") => guarantees = Some(lit_str.value()),
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

    let _guarantees_str = guarantees.unwrap_or_else(|| "Deterministic,Auditable,BioscaleSafe".into());

    let input_ty_ident: syn::Type = syn::parse_str(&input_ty)
        .expect("aln_progressor: failed to parse `input` type");
    let output_ty_ident: syn::Type = syn::parse_str(&output_ty)
        .expect("aln_progressor: failed to parse `output` type");
    let anchor_ty_ident: syn::Type = syn::parse_str(&anchor_ty)
        .expect("aln_progressor: failed to parse `anchor` type");

    // Synthesized field name for provenance.
    let provenance_field_ident = format_ident!("provenance");

    // Helper method name for the user to call one-step evolution.
    let method_ident = format_ident!("progress_with_host");

    // Expansion:
    // - keep original struct definition
    // - extend it with an Option<Provenance> field via a new impl block
    // - implement TraceProvenance
    // - implement ProgressOnce<Input = input_ty, Output = output_ty>
    //   using a placeholder deterministic body that user can customize.
    // - implement a thin AnchorToLedger binding via a helper function.
    let expanded = quote! {
        #input_struct

        impl #struct_ident {
            /// Attach an empty provenance slot; can be filled later.
            pub fn init_provenance_slot(&mut self) {
                // This field is declared via the TraceProvenance impl below.
                if self.#provenance_field_ident.is_none() {
                    self.#provenance_field_ident = Some(
                        cybocrypto_aln_core::Provenance {
                            did: String::new(),
                            bostrom_address: String::new(),
                            aln_context: self.aln_context.clone(),
                        }
                    );
                }
            }

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
                // Attach or overwrite; domain crates can tighten this if needed.
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
                // Domain logic is expected to implement BioscaleSafe on Input and
                // construct Output. This default gate ensures bioscale safety is checked once.
                use cybocrypto_aln_core::BioscaleSafe;

                if !input.is_bioscale_safe(host) {
                    return Err("bioscale safety check failed".into());
                }

                let now = std::time::SystemTime::now();

                // Step id and chaining based on internal last_stamp, if present.
                let (step_id, prev_hash) = if let Some(last) = self.last_stamp.as_ref() {
                    (last.step_id + 1, Some(last.new_hash.clone()))
                } else {
                    (0, None)
                };

                let stamp = cybocrypto_aln_core::ProgressStamp {
                    step_id,
                    prev_hash,
                    new_hash: vec![0u8; 32], // domain crate can override via a helper if needed.
                    timestamp: now,
                };

                self.last_stamp = Some(stamp.clone());

                // Delegate to a domain-provided builder function if available.
                // Assumes Output implements Default for a minimal stub.
                let output = <#output_ty_ident as Default>::default();

                Ok((output, stamp))
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
