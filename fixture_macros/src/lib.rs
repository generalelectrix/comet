use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Field, Fields};

/// Derive the EmitState trait on a fixture struct.
///
/// Fields that do not have an emit_state method can be skipped with #[skip_emit].
/// Fields that implement OscControl as well as EmitState can be forced to emit
/// with the OscControl method with the #[osc_control_emit] attribute.
#[proc_macro_derive(EmitState, attributes(skip_emit, osc_control_emit))]
pub fn derive_emit_state(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, data, .. } = parse_macro_input!(input as DeriveInput);

    let Data::Struct(struct_data) = data else {
        panic!("Can only derive EmitState for structs.");
    };
    let Fields::Named(fields) = struct_data.fields else {
        panic!("Can only derive EmitState for named structs.");
    };
    let mut lines = quote! {};
    for field in fields.named.iter() {
        if field_has_attr(field, "skip_emit") {
            continue;
        }
        let Some(ident) = &field.ident else {
            continue;
        };
        if field_has_attr(field, "osc_control_emit") {
            lines = quote! {
                #lines
                OscControl::emit_state(&self.#ident, emitter);
            }
        } else {
            lines = quote! {
                #lines
                self.#ident.emit_state(emitter);
            }
        }
    }
    quote! {
        use crate::fixture::control::OscControl;
        use crate::fixture::EmitState;
        use crate::osc::FixtureStateEmitter;
        impl EmitState for #ident {
            fn emit_state(&self, emitter: &FixtureStateEmitter) {
                #lines
            }
        }
    }
    .into()
}

fn field_has_attr(field: &Field, ident: &str) -> bool {
    field
        .attrs
        .iter()
        .any(|attr| attr.meta.path().is_ident(ident))
}
