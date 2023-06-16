use crate::{ok, AnyResult};
use anyhow::bail;
use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use std::{
    ffi::CString,
    io::{Read, Write},
    mem::MaybeUninit,
};

/// Special trait for reading packed data, always assumed to be little endian.
pub trait PackedData: Sized + Clone {
    fn read_packed<R: Read>(r: &mut R) -> AnyResult<Self>;
    fn write_packed<W: Write>(&self, w: &mut W) -> AnyResult;
}

/// Important note: `read_packed` uses `read_to_end`. No problem.
impl PackedData for Vec<u8> {
    fn read_packed<R: Read>(r: &mut R) -> AnyResult<Self> {
        let mut result = Vec::new();
        r.read_to_end(&mut result)?;
        Ok(result)
    }

    fn write_packed<W: Write>(&self, w: &mut W) -> AnyResult {
        w.write_all(self.as_ref())?;
        ok()
    }
}

impl<T: PackedData, const N: usize> PackedData for [T; N] {
    fn read_packed<R: Read>(r: &mut R) -> AnyResult<Self> {
        // This whole function sucks.
        // It relies on unsafe, and requires weird workarounds due to several useful MaybeUninit
        // functions not yet being stabilized. Basically, this is a hacky mess with workarounds
        // around workarounds
        //
        // TODO: please make PackedData impl for generic [T; N] be nice

        unsafe {
            let mut array: [MaybeUninit<T>; N] = MaybeUninit::uninit().assume_init();

            for i in 0..N {
                match T::read_packed(r) {
                    Ok(value) => array[i].write(value),

                    // In case of an error, we need to manually drop initialized elements
                    Err(e) => {
                        for v in &mut array[0..i] {
                            v.assume_init_drop();
                        }
                        return Err(e);
                    }
                };
            }

            // Using this instead of transmute, as transmute has issues with generic arrays
            Ok(array.as_ptr().cast::<[T; N]>().read())
        }
    }

    fn write_packed<W: Write>(&self, w: &mut W) -> AnyResult {
        for value in self {
            value.write_packed(w)?;
        }
        ok()
    }
}

macro_rules! impl_data {
    ($type:ty, $r:ident, $reader:expr, $w:ident, $self:ident, $writer:expr) => {
        impl PackedData for $type {
            fn read_packed<R: Read>($r: &mut R) -> AnyResult<Self> {
                Ok($reader)
            }

            fn write_packed<W: Write>(&self, $w: &mut W) -> AnyResult {
                let $self = self;
                $writer;
                Ok(())
            }
        }
    };
}

impl_data!((), _r, (), _w, _value, ());
impl_data!(u8, r, r.read_u8()?, w, value, w.write_u8(*value)?);
impl_data!(i8, r, r.read_i8()?, w, value, w.write_i8(*value)?);
impl_data!(
    u16,
    r,
    r.read_u16::<LE>()?,
    w,
    value,
    w.write_u16::<LE>(*value)?
);
impl_data!(
    i16,
    r,
    r.read_i16::<LE>()?,
    w,
    value,
    w.write_i16::<LE>(*value)?
);
impl_data!(
    u32,
    r,
    r.read_u32::<LE>()?,
    w,
    value,
    w.write_u32::<LE>(*value)?
);
impl_data!(
    i32,
    r,
    r.read_i32::<LE>()?,
    w,
    value,
    w.write_i32::<LE>(*value)?
);
impl_data!(
    f32,
    r,
    r.read_f32::<LE>()?,
    w,
    value,
    w.write_f32::<LE>(*value)?
);

impl_data!(
    CString,
    r,
    {
        const SIZE_LIMIT: usize = 8192;
        let mut result = Vec::with_capacity(100);

        loop {
            let next = r.read_u8()?;
            if next == 0 {
                result.push(0);
                break;
            } else if result.len() == SIZE_LIMIT {
                bail!("max C string size ({SIZE_LIMIT} bytes) reached");
            } else {
                result.push(next);
            }
        }

        CString::from_vec_with_nul(result)?
    },
    w,
    value,
    {
        w.write_all(value.as_bytes())?;
        w.write_all(&[0])?;
    }
);

/// Trait with a `write_packed` wrapper method for any [`Write`] type, purely for clarity.
pub trait PackedWriteExt {
    /// Writes the specified [`PackedData`] object into this stream.
    fn write_packed(&mut self, t: impl PackedData) -> AnyResult;
}

impl<T: Write> PackedWriteExt for T {
    fn write_packed(&mut self, t: impl PackedData) -> AnyResult {
        t.write_packed(self)
    }
}

/// Trait with a `read_packed` wrapper method for any [`Read`] type, purely for clarity.
pub trait PackedReadExt {
    /// Reads the specified [`PackedData`] type from this stream.
    fn read_packed<T: PackedData>(&mut self) -> AnyResult<T>;
}

impl<T: Read> PackedReadExt for T {
    fn read_packed<R: PackedData>(&mut self) -> AnyResult<R> {
        R::read_packed(self)
    }
}
