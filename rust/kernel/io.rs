use bindings::iounmap;
use core::{ffi::c_void, mem::size_of};

/// A wrapper around `ioremap` and `iounmap`.
/// 
pub struct IoReMapBox<T: Sized> {
    ptr: *mut T,
}

impl<T: Sized> IoReMapBox<T> {
    pub fn new(phys_addr: usize) -> Self {
        unsafe {
            let ptr = bindings::ioremap(
                phys_addr.try_into().unwrap(),
                size_of::<T>().try_into().unwrap(),
            ) as *mut T;
            Self { ptr }
        }
    }
}

impl<T: Sized> Drop for IoReMapBox<T> {
    fn drop(&mut self) {
        unsafe {
            iounmap(self.ptr as *mut c_void);
        }
    }
}

impl<T: Sized> core::ops::Deref for IoReMapBox<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.ptr }
    }
}
