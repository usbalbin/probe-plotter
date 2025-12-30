use probe_plotter_common::symbol::Symbol;
use proc_macro::TokenStream;
use quote::quote;
use syn::{
    RangeLimits, Token,
    parse::{self, Parse, ParseStream},
    parse_macro_input,
};

use crate::expr_to_float_lit;

pub(crate) fn make_setting(args: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as SettingArgs);

    let sym_name = serde_json::to_string(&Symbol::Setting {
        ty: args.ty.to_string().as_str().try_into().unwrap(),
        name: args.name.to_string(),
        range: args.range_start.base10_parse().unwrap()..=args.range_end.base10_parse().unwrap(),
        step_size: args.step_size.base10_parse().unwrap(),
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
                    Some(::probe_plotter::Setting::new(&mut #name.0))
                }
            }
        })
    )
    .into()
}

// root.child.leaf: i8 @ 0x1234, setting

// FOO: i32 = 0, 0..=10, 2
// FOO: i32 = 0, 0..=10, // Step size defaults to 1
// FOO: i32 = 0 // range defaults to the types full range
// TODO Implement the defaults
pub(crate) struct SettingArgs {
    pub(crate) name: syn::Ident,
    pub(crate) ty: syn::Ident,
    pub(crate) initial_val: syn::Expr,
    pub(crate) range_start: syn::LitFloat,
    pub(crate) range_end: syn::LitFloat,
    pub(crate) step_size: syn::LitFloat,
}

impl Parse for SettingArgs {
    fn parse(input: ParseStream) -> parse::Result<Self> {
        let name = input.parse()?;
        let _colon: Token![:] = input.parse()?;
        let ty = input.parse()?;
        let _eq: Token![=] = input.parse()?;
        let initial_val = input.parse()?;

        let _comma: parse::Result<Token![,]> = input.parse();
        let range: syn::Expr = input.parse()?;

        let syn::Expr::Range(range) = range else {
            panic!("Invalid range")
        };

        let range_start = range
            .start
            .expect("Only inclusive ranges with both a start and end are supported");
        let range_end = range
            .end
            .expect("Only inclusive ranges with both a start and end are supported");
        assert!(
            matches!(range.limits, RangeLimits::Closed(_)),
            "Only inclusive ranges with both a start and end are supported"
        );

        let _comma: parse::Result<Token![,]> = input.parse();
        let step_size: syn::Lit = input.parse()?;

        let step_size = match step_size {
            syn::Lit::Int(i) => syn::LitFloat::new(&format!("{}.0", i.base10_digits()), i.span()),
            syn::Lit::Float(f) => f,
            x => return Err(syn::Error::new(x.span(), "expected float or int literal")),
        };

        Ok(Self {
            name,
            ty,
            initial_val,
            range_start: expr_to_float_lit(*range_start)?,
            range_end: expr_to_float_lit(*range_end)?,
            step_size,
        })
    }
}
