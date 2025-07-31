// Based on defmt

use syn::{
    LitStr, RangeLimits, Token,
    parse::{self, Parse, ParseStream},
    spanned::Spanned,
};

//FOO: i32 = 0, "x * 3.0"
//FOO: i32 = 0 // defaults to "x"
pub(crate) struct MetricArgs {
    pub(crate) name: syn::Ident,
    pub(crate) ty: syn::Ident,
    pub(crate) initial_val: syn::Expr,
    pub(crate) expression_string: syn::LitStr,
}

impl Parse for MetricArgs {
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
            expression_string,
        })
    }
}

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
