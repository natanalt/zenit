use std::sync::Mutex;

pub trait MutexExt<T: ?Sized> {
    fn with<R>(&self, f: impl FnOnce(&mut T) -> R) -> Result<R, ()>;
}

impl<T: ?Sized> MutexExt<T> for Mutex<T> {
    fn with<R>(&self, f: impl FnOnce(&mut T) -> R) -> Result<R, ()> {
        let mut guard = self.lock().map_err(|_| ())?;
        let value = f(&mut guard);
        Ok(value)
    }
}
