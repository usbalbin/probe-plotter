use std::fmt::Display;

use syn::parse::Parse;

pub mod symbol;

#[allow(non_camel_case_types)]
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy, PartialEq, Hash, Eq)]
pub enum PrimitiveType {
    u8,
    u16,
    u32,
    i8,
    i16,
    i32,
    f32,
}

impl Display for PrimitiveType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PrimitiveType::u8 => write!(f, "u8"),
            PrimitiveType::u16 => write!(f, "u16"),
            PrimitiveType::u32 => write!(f, "u32"),
            PrimitiveType::i8 => write!(f, "i8"),
            PrimitiveType::i16 => write!(f, "i16"),
            PrimitiveType::i32 => write!(f, "i32"),
            PrimitiveType::f32 => write!(f, "f32"),
        }
    }
}

impl TryFrom<&str> for PrimitiveType {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "u8" => Ok(PrimitiveType::u8),
            "u16" => Ok(PrimitiveType::u16),
            "u32" => Ok(PrimitiveType::u32),
            "i8" => Ok(PrimitiveType::i8),
            "i16" => Ok(PrimitiveType::i16),
            "i32" => Ok(PrimitiveType::i32),
            "f32" => Ok(PrimitiveType::f32),
            _ => Err(()),
        }
    }
}

impl Parse for PrimitiveType {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident: syn::Ident = input.parse()?;
        ident.to_string().as_str().try_into().map_err(|()| {
            syn::Error::new(
                ident.span(),
                "Expected one of u8, u16, u32, i8, i16, i32 or f32",
            )
        })
    }
}

pub fn strip_dots(s: &str) -> String {
    s.replace('.', "__")
}
