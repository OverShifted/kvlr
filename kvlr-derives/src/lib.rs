use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Expr, Lit};

#[proc_macro_derive(
    Request,
    attributes(
        kvlr_request_function_id,
        kvlr_request_is_pipelined,
        kvlr_request_response
    )
)]
pub fn request(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    // input.attrs[0].meta.path().segments[0].ident.

    let mut function_id: Option<u32> = None;
    let mut is_pipelined: Option<bool> = None;
    let mut response_type: Option<String> = None;

    for attr in input.attrs {
        let name_value = attr.meta.require_name_value().unwrap();
        let attr_name = name_value.path.segments[0].ident.to_string();

        match attr_name.as_str() {
            "kvlr_request_function_id" => {
                if let Expr::Lit(ref lit, ..) = name_value.value {
                    if let Lit::Int(ref token) = lit.lit {
                        function_id = Some(token.base10_parse().unwrap());
                    } else {
                        panic!("Invalid value for `kvlr_request_function_id`. It must be an integer literal.");
                    }
                } else {
                    panic!("Invalid value for `kvlr_request_function_id`. It must be an integer literal.");
                }
            }
            "kvlr_request_is_pipelined" => {
                if let Expr::Lit(ref lit, ..) = name_value.value {
                    if let Lit::Bool(ref token) = lit.lit {
                        is_pipelined = Some(token.value);
                    } else {
                        panic!("Invalid value for `kvlr_request_is_pipelined`. It must be a boolean literal.");
                    }
                } else {
                    panic!("Invalid value for `kvlr_request_is_pipelined`. It must be a boolean literal.");
                }
            }
            "kvlr_request_response" => {
                if let Expr::Lit(ref lit, ..) = name_value.value {
                    if let Lit::Str(ref token) = lit.lit {
                        response_type = Some(token.value());
                    } else {
                        panic!("Invalid value for `kvlr_request_response`. It must be a string literal.");
                    }
                } else {
                    panic!(
                        "Invalid value for `kvlr_request_response`. It must be a string literal."
                    );
                }
            }
            _ => unreachable!(),
        }
    }

    let function_id = function_id.unwrap();
    let is_pipelined = is_pipelined.unwrap();
    let response_type = response_type.unwrap();

    let response_type: proc_macro2::TokenStream = response_type.parse().unwrap();

    let name = &input.ident;
    let expanded = quote! {
        impl Request for #name {
            const FUNCTION_ID: u32 = #function_id;
            const IS_PIPELINED: bool = #is_pipelined;

            type Response = #response_type;
        }
    };

    TokenStream::from(expanded)
}
