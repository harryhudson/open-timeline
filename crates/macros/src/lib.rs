// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! *Part of the wider OpenTimeline project*
//!
//! This crate contains the OpenTimeline procedural macros
//!

extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{LitInt, parse_macro_input};

// TODO: these are copied from elsewhere (so are not synced)
const MIN_YEAR: i64 = -50000;
const MAX_YEAR: i64 = 10000;

/// Generate the type with compile time bounds checking
fn generate_const_checked_integer_macro(
    input: TokenStream,
    type_name: &str,
    min: i64,
    max: i64,
) -> TokenStream {
    let lit = parse_macro_input!(input as LitInt);

    let value = match lit.base10_parse::<i64>() {
        Ok(v) => v,
        Err(_) => {
            return syn::Error::new_spanned(lit, "Expected a valid i64 integer literal")
                .to_compile_error()
                .into();
        }
    };

    if value < min || value > max {
        return syn::Error::new_spanned(
            lit,
            format!("{type_name} must be between {min} and {max}"),
        )
        .to_compile_error()
        .into();
    }

    let ident = syn::Ident::new(type_name, proc_macro2::Span::call_site());
    quote! {
        #ident::try_from(#value).unwrap()
    }
    .into()
}

/// Create a `Day`, using `day!(x)`, with compile time checking of the value.
#[proc_macro]
pub fn day(input: TokenStream) -> TokenStream {
    generate_const_checked_integer_macro(input, "Day", 1, 31)
}

/// Create a `Month`, using `month!(x)`, with compile time checking of the value.
#[proc_macro]
pub fn month(input: TokenStream) -> TokenStream {
    generate_const_checked_integer_macro(input, "Month", 1, 12)
}

/// Create a `Year`, using `year!(x)`, with compile time checking of the value.
#[proc_macro]
pub fn year(input: TokenStream) -> TokenStream {
    generate_const_checked_integer_macro(input, "Year", MIN_YEAR, MAX_YEAR)
}
