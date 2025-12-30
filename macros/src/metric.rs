use probe_plotter_common::symbol::Symbol;
use proc_macro::TokenStream;
use quote::quote;
use syn::{
    LitStr, Token,
    parse::{self, Parse, ParseStream},
    parse_macro_input,
};

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

    let name = args.name;
    let ty = args.ty;
    let initial_value = args.initial_val;

    quote!(
        cortex_m::interrupt::free(|_| {
            #[used]
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

//FOO: i32 = 0, "x * 3.0"
//FOO: i32 = 0 // defaults to "x"
pub(crate) struct Args {
    pub(crate) name: syn::Ident,
    pub(crate) ty: syn::Ident,
    pub(crate) initial_val: syn::Expr,
    pub(crate) expression_string: Option<syn::LitStr>,
}

impl Parse for Args {
    fn parse(input: ParseStream) -> parse::Result<Self> {
        let name: syn::Ident = input.parse()?;
        let _comma: Token![:] = input.parse()?;
        let ty = input.parse()?;
        let _comma: Token![=] = input.parse()?;
        let initial_val = input.parse()?;

        let comma: parse::Result<Token![,]> = input.parse();
        let expression_string = input.parse();

        let expression_string = match (comma, expression_string) {
            (Ok(_), Ok(expr)) => expr,
            (Ok(_), Err(e)) => return Err(e),
            (Err(_), _) => LitStr::new(&name.to_string(), name.span()),
        };

        Ok(Self {
            name,
            ty,
            initial_val,
            expression_string: Some(expression_string),
        })
    }
}
