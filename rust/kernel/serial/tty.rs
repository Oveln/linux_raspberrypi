use crate::types::Opaque;

pub struct SerialStruct(Opaque<bindings::serial_struct>);

impl SerialStruct {
    pub fn from_raw<'a>(ptr: *mut bindings::serial_struct) -> &'a mut Self {
        let ptr = ptr.cast::<Self>();
        unsafe { &mut *ptr }
    }

    pub fn as_ptr(&self) -> *mut bindings::serial_struct {
        self.0.get()
    }
}
