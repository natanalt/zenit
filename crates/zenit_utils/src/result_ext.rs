use anyhow::anyhow;
use std::{error::Error, fmt::Display};

use crate::AnyResult;

pub trait AnyhowResultExt<T> {
    fn otherwise(self, s: impl Display) -> AnyResult<T>;
}

impl<T, E: Error + Send + Sync + 'static> AnyhowResultExt<T> for Result<T, E> {
    fn otherwise(self, s: impl Display) -> AnyResult<T> {
        match self {
            Ok(v) => Ok(v),
            Err(e) => Err(anyhow::Error::from(e).context(s.to_string())),
        }
    }
}

impl<T> AnyhowResultExt<T> for Option<T> {
    fn otherwise(self, s: impl Display) -> AnyResult<T> {
        match self {
            Some(v) => Ok(v),
            None => Err(anyhow!("{s}")),
        }
    }
}
