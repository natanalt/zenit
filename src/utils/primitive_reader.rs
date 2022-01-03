use byteorder::ByteOrder;
use std::{cmp::min, marker::PhantomData, mem::size_of};

/// Data parser allowing reading multibyte primitives in a stream-like fashion.
#[derive(Debug, Clone)]
pub struct PrimitiveReader<'s, Endian: ByteOrder> {
    data: &'s [u8],
    offset: usize,
    _phantom: PhantomData<Endian>,
}

macro_rules! reader_template {
    ($fn_name:ident, $ty_name:ty) => {
        pub fn $fn_name(&mut self) -> Option<$ty_name> {
            if self.remaining_bytes() >= size_of::<$ty_name>() {
                let result =
                    Endian::$fn_name(&self.data[self.offset..self.offset + size_of::<$ty_name>()]);
                self.offset += size_of::<$ty_name>();
                Some(result)
            } else {
                None
            }
        }
    };
}

macro_rules! peeker_template {
    ($fn_name:ident, $reader_fn_name:ident, $ty_name:ty) => {
        pub fn $fn_name(&mut self) -> Option<$ty_name> {
            if self.remaining_bytes() >= size_of::<$ty_name>() {
                let result = Endian::$reader_fn_name(
                    &self.data[self.offset..self.offset + size_of::<$ty_name>()],
                );
                Some(result)
            } else {
                None
            }
        }
    };
}

impl<'s, Endian: ByteOrder> PrimitiveReader<'s, Endian> {
    pub fn new(data: &'s [u8]) -> Self {
        Self {
            data,
            offset: 0,
            _phantom: Default::default(),
        }
    }

    pub fn remaining_bytes(&self) -> usize {
        self.data.len() - self.offset
    }

    pub fn skip_specific_bytes(&mut self, value: u8) {
        while self.peek_u8() == Some(value) {
            self.offset += 1;
        }
    }

    pub fn slice_from_current(&self, bytes: usize) -> &'s [u8] {
        &self.data[self.offset..self.offset + bytes]
    }

    pub fn skip_bytes(&mut self, count: usize) {
        self.offset += min(count, self.remaining_bytes());
    }

    pub fn read_bytes(&mut self, target: &mut [u8]) -> usize {
        let bytes_to_write = min(target.len(), self.remaining_bytes());
        target.copy_from_slice(self.slice_from_current(bytes_to_write));
        self.offset += bytes_to_write;
        bytes_to_write
    }

    pub fn peek_bytes(&mut self, target: &mut [u8]) -> usize {
        let bytes_to_write = min(target.len(), self.remaining_bytes());
        target.copy_from_slice(self.slice_from_current(bytes_to_write));
        bytes_to_write
    }

    pub fn read_u8(&mut self) -> Option<u8> {
        if self.remaining_bytes() >= 1 {
            let result = self.data[self.offset];
            self.offset += 1;
            Some(result)
        } else {
            None
        }
    }

    pub fn peek_u8(&mut self) -> Option<u8> {
        if self.remaining_bytes() >= 1 {
            let result = self.data[self.offset];
            Some(result)
        } else {
            None
        }
    }

    reader_template!(read_u16, u16);
    reader_template!(read_i16, i16);
    reader_template!(read_u32, u32);
    reader_template!(read_i32, i32);
    reader_template!(read_f32, f32);
    peeker_template!(peek_u16, read_u16, u16);
    peeker_template!(peek_i16, read_i16, i16);
    peeker_template!(peek_u32, read_u32, u32);
    peeker_template!(peek_i32, read_i32, i32);
    peeker_template!(peek_f32, read_f32, f32);

    pub fn data(&self) -> &'s [u8] {
        self.data
    }

    pub fn offset(&self) -> usize {
        self.offset
    }
}
