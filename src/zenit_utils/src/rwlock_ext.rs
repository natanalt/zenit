use std::sync::RwLock;

pub trait RwLockExt<T: ?Sized> {
    fn write_with<R>(&self, f: impl FnOnce(&mut T) -> R) -> Result<R, ()>;
    fn read_with<R>(&self, f: impl FnOnce(&T) -> R) -> Result<R, ()>;
}

impl<T: ?Sized> RwLockExt<T> for RwLock<T> {
    fn write_with<R>(&self, f: impl FnOnce(&mut T) -> R) -> Result<R, ()> {
        let mut guard = self.write().map_err(|_| ())?;
        let value = f(&mut guard);
        Ok(value)
    }

    fn read_with<R>(&self, f: impl FnOnce(&T) -> R) -> Result<R, ()> {
        let guard = self.read().map_err(|_| ())?;
        let value = f(&guard);
        Ok(value)
    }
}
