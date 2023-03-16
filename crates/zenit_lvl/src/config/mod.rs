use std::ffi::CString;

pub mod read;

#[derive(Debug, Clone)]
pub struct LevelConfig {
    pub name_hash: u32,
    pub root: ConfigScope,
}

#[derive(Debug, Clone)]
pub enum ConfigObject {
    Data(ConfigData),
    Scope(ConfigScope),
}

#[derive(Debug, Clone, Default)]
pub struct ConfigScope {
    pub children: Vec<ConfigObject>,
}

#[derive(Debug, Clone)]
pub struct ConfigData {
    pub name_hash: u32,
    /// Tuple-like values stored in this data object. Use [`ConfigData::get`] for actual reading.
    pub values: Vec<u32>,
    pub tail: Vec<u8>,
}

impl ConfigData {
    /// Attempts to read a value from the object. The generic type parameter has implementations
    /// for `f32` and `CString` which is all data chunks can store anyway.
    pub fn get<T: FromConfigValue>(&self, idx: u32) -> Option<T> {
        T::get(self, idx)
    }
}

/// Magic implementation stuff for [`ConfigData::get`]
pub trait FromConfigValue
where
    Self: Sized,
{
    fn get(data: &ConfigData, idx: u32) -> Option<Self>;
}

impl FromConfigValue for CString {
    // TODO: cleanup impl FromConfigValue for CString
    fn get(data: &ConfigData, idx: u32) -> Option<Self> {
        // To quote myself:
        // > After everything, I came to the conclusion, that:
        // > for each string parameter, a u32 value is stored, that defines
        // > the offset to its contents in the DATA's tail at exactly
        // >     offset = buffer_start + the_value + 9
        // > I have absolutely no idea what that 9 does there, but it seems
        // > like a fairly common pattern among all DATA chunks I checked so far
        // This may be incredibly incorrect.
        data.values
            .get(idx as usize)
            .map(|&v| v as usize)
            .and_then(|offset| {
                // ConfigData doesn't emulate the in-file format of config data,
                // so we have to do a bit of math trickery in here

                // No idea how I figured out this magic, I already forgot
                let idx = idx as usize;
                let in_tail_offset = offset - idx * 4;

                // Check if there's a terminator right before us
                if in_tail_offset != 0 {
                    // If not, we should be able to safely assume that the value is invalid...
                    // Hopefully no one depends on completely wacky strings?
                    if data.tail[in_tail_offset - 1] != 0 {
                        return None;
                    }
                }

                // Verify if there's a terminator, and if so, make a CString
                let potential_string = &data.tail[in_tail_offset..];
                potential_string
                    .iter()
                    .zip(0usize..)
                    .find(|(&byte, _)| byte == 0)
                    .and_then(|(_, len)| CString::new(&potential_string[0..len]).ok())
            })
    }
}

impl FromConfigValue for f32 {
    fn get(data: &ConfigData, idx: u32) -> Option<Self> {
        data.values.get(idx as usize).map(|raw| {
            // Reinterpret the value as a float
            f32::from_bits(*raw)
        })
    }
}
