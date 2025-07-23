use syn::{
    Token,
    parse::{self, Parse, ParseStream},
};

//FOO: i32 = 0, "x * 3.0"
pub(crate) struct Args {
    pub(crate) name: syn::Ident,
    pub(crate) ty: syn::Ident,
    pub(crate) initial_val: syn::Expr,
    pub(crate) expression_string: syn::LitStr,
}

impl Parse for Args {
    fn parse(input: ParseStream) -> parse::Result<Self> {
        Ok(Self {
            name: input.parse()?,
            ty: {
                let _comma: Token![:] = input.parse()?;
                input.parse()?
            },
            initial_val: {
                let _comma: Token![=] = input.parse()?;
                input.parse()?
            },
            expression_string: {
                let _comma: Token![,] = input.parse()?;
                input.parse()?
            },
        })
    }
}
