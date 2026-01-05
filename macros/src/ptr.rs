use proc_macro::TokenStream;
use proc_macro2::Span;
use syn::{
    ExprLit, Ident, LitInt,
    parse::{self, Parse, ParseStream},
    parse_macro_input,
};

use crate::{
    metric::{self, metric_helper},
    parse_name,
};

pub fn make_ptr(args: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as Args);

    let zero = syn::Expr::Lit(ExprLit {
        attrs: vec![],
        lit: syn::Lit::Int(LitInt::new("0", Span::call_site())),
    });
    metric_helper(metric::Args {
        ty: Ident::new("u32", Span::call_site()),
        name: args.name.to_string(),
        initial_val: zero,
        expression_string: None,
        static_name: args.static_name,
    })
}

pub(crate) struct Args {
    pub(crate) name: String,
    pub(crate) static_name: syn::Ident,
}

impl Parse for Args {
    fn parse(input: ParseStream) -> parse::Result<Self> {
        let (static_name, name, _name_span) = parse_name(&input)?;

        Ok(Self { static_name, name })
    }
}
