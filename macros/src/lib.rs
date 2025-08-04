// Based on defmt and cortex_m::singleton

extern crate proc_macro;
use std::hash::{DefaultHasher, Hash, Hasher};

use proc_macro::{Span, TokenStream};
use quote::quote;
use syn::parse_macro_input;

use crate::symbol::{GraphSymbol, MetricsSymbol, SettingSymbol};

mod args;
mod cargo;
mod symbol;

/// Create a Metric instance that will be shown in the probe-plotter utility's graph
///
/// ```
/// make_metric!(NAME_AS_SHOWN_IN_GRAPH: DataType = defalt_value, "expression to convert from raw value (NAME_AS_SHOWN_IN_GRAPH) to the value to plot")
/// ```
///
/// Note that similar to `cortex_m::singleton!`, this should only be called once per metric. The macro will only return Some() the first time, then None.
///
/// ```
/// let mut metric_foo = probe_plotter::make_metric!(FOO: i32 = 0, "FOO * 3.0").unwrap();
///
/// metric_foo.set(42); // The value 42 will be available for the host after this call. The value will be plotted as FOO * 3 = 42 * 3 = 126
/// ```
#[proc_macro]
pub fn make_metric(args: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as args::MetricArgs);

    let sym_name = MetricsSymbol::new(args.ty.to_string(), args.name.to_string()).mangle();

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

/// Create a Setting instance that will be shown as a slider in the probe-plotter utility
///
/// ```
/// make_setting!(NAME_AS_SHOWN_NEXT_TO_SLIDER: DataType = defalt_value, min_value..=max_value, step_size)
/// ```
///
/// Note that similar to `cortex_m::singleton!`, this should only be called once per setting. The macro will only return Some() the first time, then None.
///
/// ```
/// let mut setting_foo = probe_plotter::make_setting!(FOO: i32 = 0, 0..=10, 1.0).unwrap();
///
/// let value = setting_foo.get();
/// ```
#[proc_macro]
pub fn make_setting(args: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as args::SettingArgs);

    let sym_name = SettingSymbol::new(
        args.ty.to_string(),
        args.name.to_string(),
        args.range_start.base10_parse().unwrap()..=args.range_end.base10_parse().unwrap(),
        args.step_size.base10_parse().unwrap(),
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
                    Some(::probe_plotter::Setting::new(&mut #name.0))
                }
            }
        })
    )
    .into()
}

/// Register a graph for the probe-plotter utility's to plot.
///
/// This has access to all metrics and settings which may be used in the expression
///
/// ```
/// make_graph!(GRAPH_TITLE = "expression to convert from raw values to the value to plot, may refer to any metrics or settings")
/// ```
///
/// ```
/// probe_plotter::make_graph!(FOO_GRAPH = "FOO * 3.0");
///
///
/// let mut metric_foo = probe_plotter::make_metric!(FOO: i32 = 0).unwrap();
///
/// metric_foo.set(42); // The value 42 will be available for the host after this call. The value will be plotted as FOO * 3 = 42 * 3 = 126
/// ```
#[proc_macro]
pub fn make_graph(args: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as args::GraphArgs);

    let sym_name = GraphSymbol::new(args.name.to_string(), args.expression_string.value()).mangle();
    let section = linker_section(false, &sym_name);
    let section_for_macos = linker_section(true, &sym_name);

    let name = args.name;
    quote!(
        #[cfg_attr(target_os = "macos", unsafe(link_section = #section_for_macos))]
        #[cfg_attr(not(target_os = "macos"), unsafe(link_section = #section))]
        #[unsafe(export_name = #sym_name)]
        static mut #name: u8 = 42;

        #[allow(unsafe_code)]
        unsafe {
            // TODO: Find better way to ensure the compiler considers this static used
            // the #[used] attribute does not seem enough
            let mut x = &raw const #name as usize;
            core::arch::asm!("mov {0}, {0}", inout(reg) x);
        }
    )
    .into()
}
/*
quote!(
        #[cfg_attr(target_os = "macos", unsafe(link_section = #section_for_macos))]
        #[cfg_attr(not(target_os = "macos"), unsafe(link_section = #section))]
        #[used]
        #[unsafe(export_name = #sym_name)]
        static #name: u8 = 0;
    )
    .into()


let name = args.name;
    quote!(
        #[used]
        #[unsafe(export_name = #sym_name)]
        static mut #name: (i8, bool) =
            (0, false);
    )
    .into()
    */

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
