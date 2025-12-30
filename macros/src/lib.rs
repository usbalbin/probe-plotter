// Based on defmt and cortex_m::singleton

extern crate proc_macro;

use proc_macro::{TokenStream};
use syn::{spanned::Spanned};

mod cargo;
mod ptr;
mod setting;
mod metric;
mod metric_from_address;
mod metric_from_base_with_offset;

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
    metric::make_metric(args)
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
    setting::make_setting(args)
}

/// See [make_metric_from_base_with_offset] for more info
#[proc_macro]
pub fn make_ptr(args: TokenStream) -> TokenStream {
    ptr::make_ptr(args)
}

/// Tell probe-plotter-tools about an existing value at the provided address
/// which should be shown as a metric.
/// 
/// # NOTE:
/// unlike the regular [make_metric], this macro will not return a `Metric` object. The host
/// will see the value as is in memory with no need to manually set it as with `Metric::set`.
/// 
/// Due to this, the optimizer may in some cases, especially if non-volatile stores are used,
/// decide to remove stores to the address specified. The host will then not see those writes.
///
/// ```rust
/// probe_plotter::make_metric_from_address(root.path.child: u8 @ 0x1234, "3 * root.path.child");
/// ```
#[proc_macro]
pub fn make_metric_from_address(args: TokenStream) -> TokenStream {
    metric_from_address::make_metric_from_address(args)
}

/// Tell probe-plotter-tools about an existing value at a relative offset with an other metrics value as base
///
/// ```rust
/// let some_address: *const u8 = *const 0x1234;
/// let mut my_ptr_metric = probe_plotter::make_ptr(MY_PTR_METRIC);
/// my_ptr_metric.set(some_address);
/// probe_plotter::make_metric_from_address_with_offset(root.path.child: u8 @ MY_PTR_METRIC + 42, "3 * root.path.child");
/// // The address of the metric `root.path.child` will be 0x1234 + 42 
/// ```
#[proc_macro]
pub fn make_metric_from_base_with_offset(args: TokenStream) -> TokenStream {
    metric_from_base_with_offset::make_metric_from_base_with_offset(args)
}

// TODO: Clean up this mess
fn expr_to_float_lit(e: syn::Expr) -> Result<syn::LitFloat, syn::Error> {
    let error_msg = "expected float or int literal";
    Ok(match e {
        syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Float(f),
            ..
        }) => f,
        syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Int(i),
            ..
        }) => syn::LitFloat::new(&format!("{}.0", i.base10_digits()), i.span()),
        syn::Expr::Unary(syn::ExprUnary {
            op: syn::UnOp::Neg(_),
            expr,
            ..
        }) => match *expr {
            syn::Expr::Lit(syn::ExprLit { lit, .. }) => match lit {
                // TODO: Is there a better way to handle the minus sign?
                syn::Lit::Int(i) => {
                    syn::LitFloat::new(&format!("-{}.0", i.base10_digits()), i.span())
                }
                syn::Lit::Float(f) => {
                    syn::LitFloat::new(&format!("-{}", f.base10_digits()), f.span())
                }
                x => return Err(syn::Error::new(x.span(), error_msg)),
            },
            x => return Err(syn::Error::new(x.span(), error_msg)),
        },
        x => return Err(syn::Error::new(x.span(), error_msg)),
    })
}
