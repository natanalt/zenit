use std::{
    io::{self, Read, Seek, Take},
    ops::RangeInclusive,
};

// TODO: cleanup SeekableTake (remove unwraps and all that)

/// A very basic implementation of a [`Read`] + [`Seek`] interface that works
/// like [`Take`] does.
#[derive(Debug)]
pub struct SeekableTake<T: Read + Seek> {
    inner: Take<T>,
    range: RangeInclusive<u64>,
}

impl<T: Read + Seek> Read for SeekableTake<T> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.read(buf)
    }
}

impl<T: Read + Seek> Seek for SeekableTake<T> {
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        use io::SeekFrom::*;

        let current_pos: i64 = self.stream_position()?.try_into().unwrap();

        match pos {
            Current(n) => {
                // i'm just so tired, just cast it as i64
                // if this ever comes back to haunt me i'll cry
                let n = n as i64;
                let current_limit = self.inner.limit() as i64;

                let new_position = current_pos + n;
                let new_limit = current_limit - n;

                if new_position < 0 || new_limit < 0 || !self.range.contains(&(new_position as u64))
                {
                    Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "seeking outside the SeekableTake range",
                    ))
                } else {
                    self.inner.set_limit(new_limit as u64);
                    self.inner.get_mut().seek(pos)
                }
            }
            Start(n) => {
                let new_position = n as i64;
                let _current_limit = self.inner.limit() as i64;

                let new_limit = *self.range.end() as i64 - new_position;
                if new_position < 0 || new_limit < 0 || !self.range.contains(&(new_position as u64))
                {
                    Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "seeking outside the SeekableTake range",
                    ))
                } else {
                    self.inner.set_limit(new_limit as u64);
                    self.inner.get_mut().seek(pos)
                }
            }
            End(_) => todo!("seeking from the end not yet supported"),
        }
    }

    fn stream_position(&mut self) -> io::Result<u64> {
        self.inner.get_mut().stream_position()
    }
}

pub trait SeekableTakeExt: Read + Seek {
    fn seekable_take(&mut self, n: u64) -> SeekableTake<&mut Self>;
}

impl<T: Read + Seek> SeekableTakeExt for T {
    fn seekable_take(&mut self, n: u64) -> SeekableTake<&mut Self> {
        // fuck me if this fails
        let start = self.stream_position().unwrap();

        SeekableTake {
            range: start..=start + n,
            inner: self.take(n),
        }
    }
}
