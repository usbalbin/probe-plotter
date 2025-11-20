use std::{collections::HashMap, fmt::Display};

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

#[derive(serde::Serialize, serde::Deserialize, PartialEq, Clone, Hash, Eq)]
pub struct Atype(String);

impl Display for Atype {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

pub type Types = HashMap<Atype, TypeDef>;

pub enum TypeDef {
    Struct {
        name: Atype,
        fields: Vec<symbol::Member>,
    },
    Enum {
        name: Atype,
        discriminator_type: PrimitiveType,
        variants: Vec<EnumVariant>,
    },
}

impl TypeDef {
    fn size_of(&self, types: &Types) -> u64 {
        match self {
            TypeDef::Struct { name, fields } => fields
                .last()
                .map(|x| {
                    x.offset.unwrap().next_multiple_of(x.ty.align_of(&types)) + x.ty.size_of(&types)
                })
                .unwrap_or(0),
            TypeDef::Enum {
                name,
                discriminator_type,
                variants,
            } => todo!(),
        }
    }

    fn align_of(&self, _types: &Types) -> u64 {
        todo!()
    }
}

pub struct EnumVariant {
    pub name: String,
    pub ty: PrimitiveType,
    pub expr: shunting::RPNExpr,
}

impl Atype {
    pub fn is_primitive(&self) -> bool {
        ["u8", "u16", "u32", "i8", "i16", "i32", "f32"].contains(&self.0.as_ref())
    }

    pub fn size_of(&self, types: &Types) -> u64 {
        match PrimitiveType::try_from(self) {
            Ok(p) => p.size_of(),
            Err(()) => types.get(self).unwrap().size_of(types),
        }
    }

    pub fn align_of(&self, types: &Types) -> u64 {
        match PrimitiveType::try_from(self) {
            Ok(p) => p.align_of(),
            Err(()) => types.get(self).unwrap().size_of(types),
        }
    }
}

impl TryFrom<&Atype> for PrimitiveType {
    type Error = ();

    fn try_from(t: &Atype) -> Result<PrimitiveType, Self::Error> {
        match t.0.as_str() {
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

impl PrimitiveType {
    pub fn size_of(&self) -> u64 {
        match self {
            PrimitiveType::u8 | PrimitiveType::i8 => 1,
            PrimitiveType::u16 | PrimitiveType::i16 => 2,
            PrimitiveType::u32 | PrimitiveType::i32 | PrimitiveType::f32 => 4,
        }
    }

    pub fn align_of(&self) -> u64 {
        match self {
            PrimitiveType::u8 | PrimitiveType::i8 => 1,
            PrimitiveType::u16 | PrimitiveType::i16 => 2,
            PrimitiveType::u32 | PrimitiveType::i32 | PrimitiveType::f32 => 4,
        }
    }
}

impl Into<Atype> for &PrimitiveType {
    fn into(self) -> Atype {
        match self {
            PrimitiveType::u8 => Atype("u8".to_string()),
            PrimitiveType::u16 => Atype("u16".to_string()),
            PrimitiveType::u32 => Atype("u32".to_string()),
            PrimitiveType::i8 => Atype("i8".to_string()),
            PrimitiveType::i16 => Atype("i16".to_string()),
            PrimitiveType::i32 => Atype("i32".to_string()),
            PrimitiveType::f32 => Atype("f32".to_string()),
        }
    }
}
