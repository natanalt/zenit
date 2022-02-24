use anyhow::bail;
use byteorder::{ReadBytesExt, LE};
use std::{ffi::CString, io::Read};

use crate::AnyResult;

/// Special trait for reading packed data, always assumed to be little endian.
/// Can be implemented using the [`zenit_proc::PackedParser`] derive macro.
pub trait PackedParser
where
    Self: Sized,
{
    fn parse_packed<R: Read>(r: &mut R) -> AnyResult<Self>;
}

impl PackedParser for Vec<u8> {
    fn parse_packed<R: Read>(r: &mut R) -> AnyResult<Self> {
        let mut result = Vec::new();
        r.read_to_end(&mut result)?;
        Ok(result)
    }
}

macro_rules! impl_parser {
    ($type:ty, $r:ident, $($content:tt)*) => {
        impl PackedParser for $type {
            fn parse_packed<R: Read>($r: &mut R) -> AnyResult<Self> {
                Ok($($content)*)
            }
        }
    }
}

impl_parser!((), _r, ());
impl_parser!(u8, r, r.read_u8()?);
impl_parser!(i8, r, r.read_i8()?);
impl_parser!(u16, r, r.read_u16::<LE>()?);
impl_parser!(i16, r, r.read_i16::<LE>()?);
impl_parser!(u32, r, r.read_u32::<LE>()?);
impl_parser!(i32, r, r.read_i32::<LE>()?);
impl_parser!(f32, r, r.read_f32::<LE>()?);

impl_parser!(CString, r, {
    const SIZE_LIMIT: usize = 8192;
    let mut result = Vec::with_capacity(100);

    loop {
        let next = r.read_u8()?;
        if next == 0 {
            break;
        } else if result.len() == SIZE_LIMIT {
            bail!("max c string size ({} bytes) reached", SIZE_LIMIT);
        } else {
            result.push(next);
        }
    }

    CString::from_vec_with_nul(result)?
});
