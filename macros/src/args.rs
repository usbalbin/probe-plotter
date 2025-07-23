use proc_macro2::Span;
use syn::{
    LitStr, Token,
    parse::{self, Parse, ParseStream},
};

//FOO: i32 = 0, "x * 3.0"
//FOO: i32 = 0 // defaults to "x"
pub(crate) struct Args {
    pub(crate) name: syn::Ident,
    pub(crate) ty: syn::Ident,
    pub(crate) initial_val: syn::Expr,
    pub(crate) expression_string: syn::LitStr,
}

impl Parse for Args {
    fn parse(input: ParseStream) -> parse::Result<Self> {
        let name = input.parse()?;
        let _comma: Token![:] = input.parse()?;
        let ty = input.parse()?;
        let _comma: Token![=] = input.parse()?;
        let initial_val = input.parse()?;

        let comma: parse::Result<Token![,]> = input.parse();
        let expression_string = input.parse();

        let expression_string = match (comma, expression_string) {
            (Ok(_), Ok(expr)) => expr,
            (Ok(_), Err(e)) => return Err(e),
            (Err(_), _) => LitStr::new("x", Span::mixed_site()),
        };

        Ok(Self {
            name,
            ty,
            initial_val,
            expression_string,
        })
    }
}
