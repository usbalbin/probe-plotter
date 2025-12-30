use proc_macro::TokenStream;
use proc_macro2::Span;
use syn::{ExprLit, LitInt, parse::{self, Parse, ParseStream}, parse_macro_input};

use crate::metric::{MetricArgs, metric_helper};

pub fn make_ptr(args: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as Args);

    let zero = syn::Expr::Lit(ExprLit{
        attrs: vec![],
        lit: syn::Lit::Int(LitInt::new("0", Span::call_site()))
    });
    metric_helper(MetricArgs{ ty: probe_plotter_common::PrimitiveType::u32, name: args.name, initial_val: zero, expression_string: None  })
}

pub(crate) struct Args {
    pub(crate) name: syn::Ident,
}

impl Parse for Args {
    fn parse(input: ParseStream) -> parse::Result<Self> {
        let name: syn::Ident = input.parse()?;

        Ok(Self {
            name,
        })
    }
}