use probe_plotter_common::{
    PrimitiveType,
    symbol::{Address, Symbol},
};
use proc_macro::TokenStream;
use quote::quote;
use syn::{
    LitInt, LitStr, Token,
    parse::{self, Parse, ParseStream},
    parse_macro_input,
};

use crate::parse_name;

pub fn make_metric_from_base_with_offset(args: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as Args);

    let sym_name = serde_json::to_string(&Symbol::Metric {
        ty: args.ty,
        name: args.name.clone(),
        expr: Some(args.expression_string.value()),
        address: Address::RelativeBaseMetricWithOffset {
            base_metric: args.base_symbol.to_string(),
            offset: args.offset,
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

// root.child.leaf: i8 @ BASE_SYMBOL + 3, "root.child.leaf"
pub(crate) struct Args {
    pub(crate) name: String,
    pub(crate) ty: PrimitiveType,
    pub(crate) base_symbol: syn::Ident,
    pub(crate) offset: u64,
    pub(crate) expression_string: syn::LitStr,
    pub(crate) static_name: syn::Ident,
}

impl Parse for Args {
    fn parse(input: ParseStream) -> parse::Result<Self> {
        let (static_name, name, name_span) = parse_name(&input)?;

        let _colon: Token![:] = input.parse()?;
        let ty = input.parse()?;
        let _at: Token![@] = input.parse()?;
        let base_symbol = input.parse()?;
        let _plus: Token![+] = input.parse()?;
        let offset: LitInt = input.parse()?;
        let offset = offset.base10_parse()?;

        let comma: parse::Result<Token![,]> = input.parse();
        let expression_string = input.parse();

        let expression_string = match (comma, expression_string) {
            (Ok(_), Ok(expr)) => expr,
            (Ok(_), Err(e)) => return Err(e),
            (Err(_), _) => LitStr::new(&name, name_span),
        };

        Ok(Args {
            name,
            ty,
            base_symbol,
            offset,
            expression_string,
            static_name,
        })
    }
}
