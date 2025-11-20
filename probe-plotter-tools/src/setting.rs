use std::ops::RangeInclusive;

use probe_plotter_common::PrimitiveType;
use probe_rs::MemoryInterface;

use crate::read_value;

#[derive(Clone, Debug)]
pub struct Setting {
    pub name: String,
    pub ty: PrimitiveType,
    pub address: u64,
    pub value: f64,
    pub range: RangeInclusive<f64>,
    pub step_size: f64,
}

impl Setting {
    pub fn read(&mut self, core: &mut probe_rs::Core) -> Result<(), probe_rs::Error> {
        self.value = read_value(core, self.address, self.ty)?;
        Ok(())
    }

    pub fn write(&mut self, x: f64, core: &mut probe_rs::Core) -> Result<(), probe_rs::Error> {
        match self.ty {
            PrimitiveType::u8 => core.write_word_8(
                self.address,
                x.round().clamp(u8::MIN as _, u8::MAX as _) as u8,
            )?,
            PrimitiveType::u16 => core.write_word_16(
                self.address,
                x.round().clamp(u16::MIN as _, u16::MAX as _) as u16,
            )?,
            PrimitiveType::u32 => core.write_word_32(
                self.address,
                x.round().clamp(u32::MIN as _, u32::MAX as _) as u32,
            )?,

            PrimitiveType::i8 => core.write_word_8(
                self.address,
                x.round().clamp(i8::MIN as _, i8::MAX as _) as u8,
            )?,
            PrimitiveType::i16 => core.write_word_16(
                self.address,
                x.round().clamp(i16::MIN as _, i16::MAX as _) as u16,
            )?,
            PrimitiveType::i32 => core.write_word_32(
                self.address,
                x.round().clamp(i32::MIN as _, i32::MAX as _) as u32,
            )?,

            PrimitiveType::f32 => core.write_word_32(self.address, (x as f32).to_bits())?,
        };

        Ok(())
    }
}
