extern crate proc_macro;
use core::fmt;
use layers_dsl_core::{code_gen, LayerItem};
use proc_macro::TokenStream;

use std::{
    sync::atomic::{AtomicUsize, Ordering},
    *,
};

use syn::__private::{quote::quote, Span};

#[allow(unused_imports)] // typical / pervasive syn imports
use ::syn::{
    parse::{Parse, ParseStream, Parser},
    punctuated::Punctuated,
    spanned::Spanned,
    Result, // explicitly shadow it
    *,
};

fn fn_proc_macro_impl(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let layer: LayerItem = parse_macro_input!(input);
    code_gen(&layer).into()
}

#[proc_macro]
pub fn layers(input: TokenStream) -> TokenStream {
    fn_proc_macro_impl(input)
}
