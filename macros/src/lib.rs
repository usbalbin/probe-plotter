// Based on defmt and cortex_m::singleton

extern crate proc_macro;
use std::hash::{DefaultHasher, Hash, Hasher};

use proc_macro::{Span, TokenStream};
use quote::quote;
use syn::parse_macro_input;

use crate::symbol::Symbol;

mod args;
mod cargo;
mod symbol;

/// Create a Metric instance that will be shown in the probe-plotter utility's graph
///
/// ```
/// make_metric!(NAME_AS_SHOWN_IN_GRAPH: DataType = defalt_value, "expression to convert from raw value (x) to the value to plot")
/// ```
///
/// Note that similar to `cortex_m::singleton!`, this should only be called once per metric. The macro will only return Some() the first time, then None.
///
/// ```
/// let mut metric_foo = probe_plotter::make_metric!(FOO: i32 = 0, "x * 3.0").unwrap();
///
/// metric_foo.set(42); // The value 42 will be available for the host after this call. The value will be plotted as x * 3 = 42 * 3 = 126
/// ```
#[proc_macro]
pub fn make_metric(args: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as args::Args);

    let sym_name = Symbol::new(
        args.ty.to_string(),
        args.name.to_string(),
        args.expression_string.value(),
    )
    .mangle();

    let name = args.name;
    let ty = args.ty;
    let initial_value = args.initial_val;

    quote!(
        cortex_m::interrupt::free(|_| {
            #[unsafe(export_name = #sym_name)]
            static mut #name: (#ty, bool) =
                (0, false);

            #[allow(unsafe_code)]
            let used = unsafe { #name.1 };
            if used {
                None
            } else {
                #[allow(unsafe_code)]
                unsafe {
                    #name.1 = true;
                    #name.0 = #initial_value;
                    Some(::probe_plotter::Metric::new(&mut #name.0))
                }
            }
        })
    )
    .into()
}

pub(crate) fn crate_local_disambiguator() -> u64 {
    // We want a deterministic, but unique-per-macro-invocation identifier. For that we
    // hash the call site `Span`'s debug representation, which contains a counter that
    // should disambiguate macro invocations within a crate.
    hash(&format!("{:?}", Span::call_site()))
}

fn hash(string: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    string.hash(&mut hasher);
    hasher.finish()
}
