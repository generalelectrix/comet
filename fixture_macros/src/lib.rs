use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Field, Fields};

/// Derive the EmitState trait on a fixture struct.
///
/// Fields that do not have an emit_state method can be skipped with #[skip_emit].
/// Fields that implement OscControl as well as EmitState can be forced to emit
/// with the OscControl method with the #[force_osc_control] attribute.
#[proc_macro_derive(EmitState, attributes(skip_emit, force_osc_control))]
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
        if field_has_attr(field, "force_osc_control") {
            lines = quote! {
                #lines
                crate::fixture::control::OscControl::emit_state(&self.#ident, emitter);
            }
        } else {
            lines = quote! {
                #lines
                self.#ident.emit_state(emitter);
            }
        }
    }
    quote! {
        impl crate::fixture::EmitState for #ident {
            fn emit_state(&self, emitter: &crate::osc::FixtureStateEmitter) {
                #lines
            }
        }
    }
    .into()
}

/// Derive the Control trait on a fixture struct.
///
/// Fields that do not have a control method can be skipped with #[skip_control].
/// Fields that implement OscControl as well as Control can be forced to emit
/// with the OscControl method with the #[force_osc_control] attribute.
#[proc_macro_derive(Control, attributes(skip_control, force_osc_control))]
pub fn derive_control(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, data, .. } = parse_macro_input!(input as DeriveInput);

    let Data::Struct(struct_data) = data else {
        panic!("Can only derive Control for structs.");
    };
    let Fields::Named(fields) = struct_data.fields else {
        panic!("Can only derive Control for named structs.");
    };
    let mut lines = quote! {};
    for field in fields.named.iter() {
        if field_has_attr(field, "skip_control") {
            continue;
        }
        let Some(ident) = &field.ident else {
            continue;
        };
        if field_has_attr(field, "force_osc_control") {
            lines = quote! {
                #lines
                if crate::fixture::control::OscControl::control(&mut self.#ident, msg, emitter)? {
                    return Ok(true);
                }
            }
        } else {
            lines = quote! {
                #lines
                if self.#ident.control(msg, emitter)? {
                    return Ok(true);
                }
            }
        }
    }
    quote! {
        impl crate::fixture::Control for #ident {
            fn control(&mut self, msg: &crate::osc::OscControlMessage, emitter: &crate::osc::FixtureStateEmitter) -> anyhow::Result<bool> {
                #lines
                Ok(false)
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
