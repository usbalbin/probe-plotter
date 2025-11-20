use probe_plotter_common::{Atype, PrimitiveType, Types};
use probe_rs::MemoryInterface;
use rerun::external::egui::ahash::HashMap;
use shunting::MathContext;

use crate::{read_from_slice, read_value};

/*impl fmt::Debug for Metric {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Metric")
            .field("name", &self.name)
            .field("expr", &self.expr)
            .field("ty", &self.ty)
            .field("address", &self.address)
            .finish()
    }
}*/

pub enum Status {
    SameAsLast,
    New,
}

pub enum Metric {
    Primitive {
        name: String,
        ty: PrimitiveType,
        address: u64,

        last_value: f64,
        expr: shunting::RPNExpr,
    },
    Enum {
        name: String,
        ty: Atype,
        address: u64,

        discriminator_type: PrimitiveType,
        last_discriminator_value: u32,
        data_variants: Vec<EnumVariant>,
        max_size: u64,
    },
}

struct EnumVariant {
    name: String,
    ty: Atype,
    expr: shunting::RPNExpr,
    last_value: f64,
}

impl Metric {
    pub fn read(
        &mut self,
        core: &mut probe_rs::Core,
        math_ctx: &mut MathContext,
        types: &Types
    ) -> Result<(), probe_rs::Error> {
        match self {
            Metric::Primitive {
                name,
                ty,
                address,
                last_value: _,
                expr: _,
            } => {
                let x = read_value(core, *address, *ty)?;
                math_ctx.setvar(&name, shunting::MathOp::Number(x));

                Ok(())
            }
            Metric::Enum {
                name,
                ty,
                address,
                discriminator_type,
                last_discriminator_value,
                data_variants,
                max_size,
            } => {
                let read_first_part =
                    |core: &mut probe_rs::Core| -> Result<Vec<_>, probe_rs::Error> {
                        Ok(match max_size {
                            0 => vec![],
                            1 => core.read_word_8(*address)?.to_le_bytes().to_vec(),
                            2 => core.read_word_16(*address)?.to_le_bytes().to_vec(),
                            3 => core.read_word_32(*address)?.to_le_bytes()[..3].to_vec(),
                            _ => core.read_word_32(*address)?.to_le_bytes().to_vec(),
                        })
                    };

                let mut bytes = read_first_part(core)?;

                assert_eq!(*discriminator_type, PrimitiveType::u8);
                let discriminator = *bytes.get(0).unwrap_or(&0) as usize;
                let size = data_variants[discriminator].ty.size_of(types);
                let align = data_variants[discriminator].ty.align_of(types);
                let variant_name = &data_variants[discriminator].name;

                let start_of_the_rest = *address + 4;

                let start = start_of_the_rest.next_multiple_of(4);

                if size > 4 {
                    let mut dst = vec![0; (size / 4 + 1) as usize];

                    assert!(start_of_the_rest % 4 == 0);
                    assert_eq!(bytes.len(), 4);
                    assert!(align <= 4);

                    core.read_32(start, &mut dst[..(size as usize / 4)])?;
                    bytes.extend(
                        dst[..(size as usize / 4)]
                            .iter()
                            .flat_map(|w| w.to_le_bytes()),
                    );
                }

                let bytes = read_first_part(core)?;
                assert_eq!(*discriminator_type, PrimitiveType::u8);
                let new_discriminator = *bytes.get(0).unwrap_or(&0) as usize;
                if new_discriminator != discriminator {
                    todo!(
                        "The discriminator changed during the read operation. The data is likely invalid"
                    );
                }
                *last_discriminator_value = new_discriminator as u32;

                let x = read_from_slice(&bytes, start, *discriminator_type);
                math_ctx.setvar(
                    &format!("{name}::{variant_name}"),
                    shunting::MathOp::Number(x),
                );

                Ok(())
            }
        }
    }

    pub fn compute(&mut self, math_ctx: &mut MathContext) -> Vec<(String, f64, Status)> {
        match self {
            Metric::Primitive {
                name,
                ty: _,
                address: _,
                expr,
                last_value,
            } => {
                let new = math_ctx.eval(&expr).unwrap();
                let status = if new == *last_value {
                    Status::SameAsLast
                } else {
                    Status::New
                };
                *last_value = new;
                vec![(name.clone(), new, status)]
            }
            Metric::Enum {
                name,
                ty: _,
                address: _,
                discriminator_type: _,
                last_discriminator_value,
                data_variants,
                max_size: _,
            } => data_variants
                .iter_mut()
                .enumerate()
                .map(|(i, x)| {
                    let new = if i as u32 == *last_discriminator_value {
                        math_ctx.eval(&x.expr).unwrap()
                    } else {
                        f64::NAN
                    };
                    let status = if new == x.last_value {
                        Status::SameAsLast
                    } else {
                        Status::New
                    };
                    let variant = &x.name;
                    (format!("{name}::{variant}"), new, status)
                })
                .collect(),
        }
    }
}
