use probe_plotter_common::symbol::Symbol;
use proc_macro::TokenStream;
use quote::quote;
use syn::{
    Token,
    parse::{self, Parse, ParseStream},
    parse_macro_input,
};

use crate::{parse_expr_str, parse_name};

pub fn make_metric(args: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as Args);
    metric_helper(args)
}

pub(crate) fn metric_helper(args: Args) -> TokenStream {
    let sym_name = serde_json::to_string(&Symbol::Metric {
        ty: args.ty.to_string().as_str().try_into().unwrap(),
        name: args.name.to_string(),
        expr: args.expression_string.map(|x| x.value()),
        address: probe_plotter_common::symbol::Address::Symbols,
    })
    .unwrap();

    let ty = args.ty;
    let initial_value = args.initial_val;
    let static_name = args.static_name;

    quote!(
        cortex_m::interrupt::free(|_| {
            #[used]
            #[unsafe(export_name = #sym_name)]
            #[allow(non_upper_case_globals)]
            static mut #static_name: (#ty, bool) =
                (0, false);

            #[allow(unsafe_code)]
            let used = unsafe { #static_name.1 };
            if used {
                None
            } else {
                #[allow(unsafe_code)]
                unsafe {
                    #static_name.1 = true;
                    #static_name.0 = #initial_value;
                    Some(::probe_plotter::Metric::new(&mut #static_name.0))
                }
            }
        })
    )
    .into()
}

//FOO: i32 = 0, "FOO * 3.0"
//FOO: i32 = 0 // defaults to "FOO"
pub(crate) struct Args {
    pub(crate) name: String,
    pub(crate) ty: syn::Ident,
    pub(crate) initial_val: syn::Expr,
    pub(crate) expression_string: Option<syn::LitStr>,
    pub(crate) static_name: syn::Ident,
}

impl Parse for Args {
    fn parse(input: ParseStream) -> parse::Result<Self> {
        let (static_name, name, name_span) = parse_name(&input)?;
        let _comma: Token![:] = input.parse()?;
        let ty = input.parse()?;
        let _comma: Token![=] = input.parse()?;
        let initial_val = input.parse()?;

        let expression_string = parse_expr_str(&input, &name, name_span)?;

        Ok(Self {
            name,
            ty,
            initial_val,
            expression_string: Some(expression_string),
            static_name,
        })
    }
}
