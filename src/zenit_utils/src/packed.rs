use byteorder::{ReadBytesExt, LE};
use std::io::Read;

pub trait PackedParser
where
    Self: Sized,
{
    fn parse_packed<R: Read>(r: &mut R) -> anyhow::Result<Self>;
}

macro_rules! impl_parser {
    ($type:ty, $r:ident, $($content:tt)*) => {
        impl PackedParser for $type {
            fn parse_packed<R: Read>($r: &mut R) -> anyhow::Result<Self> {
                Ok($($content)*)
            }
        }
    }
}

impl_parser!(u8, r, r.read_u8()?);
impl_parser!(i8, r, r.read_i8()?);
impl_parser!(u16, r, r.read_u16::<LE>()?);
impl_parser!(i16, r, r.read_i16::<LE>()?);
impl_parser!(u32, r, r.read_u32::<LE>()?);
impl_parser!(i32, r, r.read_i32::<LE>()?);
impl_parser!(f32, r, r.read_f32::<LE>()?);
