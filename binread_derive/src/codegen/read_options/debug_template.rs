use crate::codegen::sanitization::OPT;
use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

ident_str! {
    WRITE_COMMENT = from_crate!(binary_template::write_comment);
    WRITE_START_STRUCT = from_crate!(binary_template::write_start_struct);
    WRITE_END_STRUCT = from_crate!(binary_template::write_end_struct);
}

pub(super) fn end() -> TokenStream {
    quote! {
        #[cfg(feature = "debug_template")] {
            #WRITE_END_STRUCT(#OPT.variable_name);
        }
    }
}

pub(super) fn handle_error() -> TokenStream {
    let write_end_struct = end();
    quote! {
        .map_err(|e| {
            #[cfg(feature = "debug_template")] {
                #WRITE_COMMENT(&format!("Error: {:?}", e));
                #write_end_struct
            }
            e
        })
    }
}

pub(super) fn start(struct_name: &Ident) -> TokenStream {
    let struct_name = struct_name.to_string();
    quote! {
        #[cfg(feature = "debug_template")] {
            #WRITE_START_STRUCT(#struct_name);
        }
    }
}
