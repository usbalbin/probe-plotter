extern crate proc_macro;
use std::hash::{DefaultHasher, Hash, Hasher};

use proc_macro::{Span, TokenStream};
use quote::quote;
use syn::parse_macro_input;

use crate::symbol::Symbol;

mod args;
mod cargo;
mod symbol;

#[proc_macro]
pub fn make_metric(args: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as args::Args);

    let sym_name = Symbol::new(
        args.ty.to_string(),
        args.name.to_string(),
        args.expression_string.value(),
    )
    .mangle();
    let section = linker_section(false, &sym_name);
    let section_for_macos = linker_section(true, &sym_name);

    let name = args.name;
    let ty = args.ty;
    let initial_value = args.initial_val;

    quote!(
        cortex_m::interrupt::free(|_| {
            //#[cfg_attr(target_os = "macos", unsafe(link_section = #section_for_macos))]
            //#[cfg_attr(not(target_os = "macos"), unsafe(link_section = #section))]
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

/// work around restrictions on length and allowed characters imposed by macos linker
/// returns (note the comma character for macos):
///   under macos: ".defmt," + 16 character hex digest of symbol's hash
///   otherwise:   ".defmt." + prefix + symbol
pub(crate) fn linker_section(for_macos: bool, symbol: &str) -> String {
    let mut sub_section = format!(".{symbol}");

    if for_macos {
        sub_section = format!(",{:x}", hash(&sub_section));
    }

    format!(".defmt{sub_section}")
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
