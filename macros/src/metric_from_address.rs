use probe_plotter_common::{
    PrimitiveType,
    symbol::{Address, Symbol},
};
use proc_macro::TokenStream;
use quote::quote;
use syn::{
    Token,
    parse::{self, Parse, ParseStream},
    parse_macro_input,
};

use crate::{parse_expr_str, parse_name};

pub fn make_metric_from_address(args: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as Args);

    let sym_name = serde_json::to_string(&Symbol::Metric {
        ty: args.ty,
        name: args.name.clone(),
        expr: Some(args.expression_string.value()),
        address: Address::Hardcoded {
            address: args.address,
        },
    })
    .unwrap();
    let static_name = args.static_name;
    quote! {
        #[used]
        #[unsafe(export_name = #sym_name)]
        #[allow(non_upper_case_globals)]
        static #static_name: u8 = 0;
    }
    .into()
}

// root.child.leaf: i8 @ 0x1234, "root.child.leaf"
pub(crate) struct Args {
    pub(crate) name: String,
    pub(crate) ty: PrimitiveType,
    pub(crate) address: u64,
    pub(crate) expression_string: syn::LitStr,
    pub(crate) static_name: syn::Ident,
}

impl Parse for Args {
    fn parse(input: ParseStream) -> parse::Result<Self> {
        let (static_name, name, name_span) = parse_name(&input)?;

        let _colon: Token![:] = input.parse()?;
        let ty = input.parse()?;
        let _at: Token![@] = input.parse()?;
        let address: syn::LitInt = input.parse()?;
        let address = address.base10_parse()?;

        let expression_string = parse_expr_str(&input, &name, name_span)?;

        Ok(Args {
            name,
            ty,
            address,
            expression_string,
            static_name,
        })
    }
}
