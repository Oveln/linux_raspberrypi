use crate::types::Opaque;

pub struct Ktermios(Opaque<bindings::ktermios>);

impl Ktermios {
    pub fn from_raw_const<'a>(ptr: *const bindings::ktermios) -> &'a Self {
        let ptr = ptr.cast::<Self>();
        unsafe { &*ptr }
    }
    pub fn from_raw<'a>(ptr: *mut bindings::ktermios) -> &'a mut Self {
        let ptr = ptr.cast::<Self>();
        unsafe { &mut *ptr }
    }

    pub fn as_ptr(&self) -> *mut bindings::ktermios {
        self.0.get()
    }
}
