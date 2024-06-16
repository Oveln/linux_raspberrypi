use crate::{
    error::{code::EINVAL, from_result, Result},
    str::CString,
    types::Opaque,
};
use bindings::{console, tty_driver};
use core::{
    fmt::{self},
    marker::{self, PhantomData, PhantomPinned},
};
use kernel::error::{code, Error};
use macros::vtable;

pub unsafe trait RawConsole {
    fn raw_console(&self) -> *mut bindings::console;
}

impl<T> Console<T> {
    pub fn from_ptr<'a>(ptr: *mut bindings::console) -> &'a mut Self {
        unsafe { &mut *ptr.cast() }
    }
}

/// T is the type of the data that the console is registered with.
#[repr(transparent)]
pub struct Console<T> {
    console: bindings::console,
    _marker1: PhantomData<T>,
    _pin: PhantomPinned,
}

unsafe impl<T> RawConsole for Console<T> {
    fn raw_console(&self) -> *mut bindings::console {
        &self.console as *const _ as *mut bindings::console
    }
}

impl<T> Console<T> {
    // pub name: [core::ffi::c_char; 16usize],
    pub fn new<A: ConsoleOps>(
        name: fmt::Arguments<'_>,
        flags: bindings::cons_flags,
        index: i16,
        data: *const T,
    ) -> Result<Self, Error> {
        let mut console = bindings::console::default();
        let name = CString::try_from_fmt(name)?;
        if name.len() > 16 {
            return Err(code::EINVAL);
        }
        for i in 0..name.len() {
            console.name[i] = name.as_bytes()[i] as i8;
        }

        console.write = if A::HAS_WRITE {
            Some(OperationsVtable::<A>::write::<T>)
        } else {
            None
        };
        console.read = if A::HAS_READ {
            Some(OperationsVtable::<A>::read::<T>)
        } else {
            None
        };
        console.device = if A::HAS_DEVICE {
            Some(OperationsVtable::<A>::device::<T>)
        } else {
            None
        };
        console.unblank = if A::HAS_UNBLANK {
            Some(OperationsVtable::<A>::unblank::<T>)
        } else {
            None
        };
        console.setup = if A::HAS_SETUP {
            Some(OperationsVtable::<A>::setup::<T>)
        } else {
            None
        };
        console.exit = if A::HAS_EXIT {
            Some(OperationsVtable::<A>::exit::<T>)
        } else {
            None
        };
        console.match_ = if A::HAS_MATCH_ {
            Some(OperationsVtable::<A>::match_::<T>)
        } else {
            None
        };

        console.flags = flags as i16;

        console.index = index;

        console.data = &data as *const _ as *mut core::ffi::c_void;

        Ok(Self {
            console: console,
            _marker1: PhantomData,
            _pin: PhantomPinned,
        })
    }

    pub fn into_inner(self) -> bindings::console {
        self.console
    }

    pub fn set_data(&mut self, data: &T) {
        self.console.data = data as *const _ as *mut core::ffi::c_void;
    }

    pub unsafe fn get_data(&mut self) -> &mut T {
        unsafe { &mut *(self.console.data as *mut T) }
    }
}

//  * @write:		Write callback to output messages (Optional)
//  * @read:		Read callback for console input (Optional)
//  * @device:		The underlying TTY device driver (Optional)
//  * @unblank:		Callback to unblank the console (Optional)
//  * @setup:		Callback for initializing the console (Optional)
//  * @exit:		Callback for teardown of the console (Optional)
//  * @match:		Callback for matching a console (Optional)
#[vtable]
pub trait ConsoleOps {
    type DataType;

    fn write<DataType>(cons: &Console<DataType>, s: &[u8]) {}
    fn read<DataType>(cons: &Console<DataType>, s: &mut [u8], count: u32) -> Result<i32> {
        Err(EINVAL)
    }
    fn device<DataType>(cons: &Console<DataType>, index: &mut i32) -> Result<*mut bindings::tty_driver> {
        Err(EINVAL)
    }
    fn unblank<DataType>() {}
    fn setup<DataType>(cons: &Console<DataType>, options: &[u8]) -> Result<i32> {
        Err(EINVAL)
    }
    fn exit<DataType>(cons: &Console<DataType>) -> Result<i32> {
        Err(EINVAL)
    }
    fn match_<DataType>(cons: &Console<DataType>, name: &[u8], idx: i32, options: &[u8]) -> Result<i32> {
        Err(EINVAL)
    }
}

pub(crate) struct OperationsVtable<T: ConsoleOps>(PhantomData<T>);

impl<T: ConsoleOps> OperationsVtable<T> {
    unsafe extern "C" fn write<A>(
        co: *mut console,
        s: *const core::ffi::c_char,
        count: core::ffi::c_uint,
    ) {
        let cons = Console::from_ptr(co);
        let s = unsafe { core::slice::from_raw_parts(s as *const u8, count as usize) };
        T::write::<A>(cons, s)
    }
    unsafe extern "C" fn read<A>(
        co: *mut console,
        s: *mut core::ffi::c_char,
        count: core::ffi::c_uint,
    ) -> core::ffi::c_int {
        from_result(|| {
            let cons = Console::from_ptr(co);
            let s = unsafe { core::slice::from_raw_parts_mut(s as *mut u8, count as usize) };
            T::read::<A>(cons, s, count)
        })
    }

    unsafe extern "C" fn device<A>(co: *mut console, index: *mut core::ffi::c_int) -> *mut tty_driver {
        let cons = Console::from_ptr(co);
        let index = unsafe { &mut *index };
        match T::device::<A>(cons, index) {
            Ok(ptr) => ptr,
            Err(_) => core::ptr::null_mut(),
        }
    }

    unsafe extern "C" fn unblank<A>() {
        T::unblank::<A>()
    }

    unsafe extern "C" fn setup<A>(
        co: *mut console,
        options: *mut core::ffi::c_char,
    ) -> core::ffi::c_int {
        from_result(|| {
            let cons = Console::from_ptr(co);
            let options = unsafe { core::slice::from_raw_parts(options as *const u8, 16) };
            T::setup::<A>(cons, options)
        })
    }

    unsafe extern "C" fn exit<A>(co: *mut console) -> core::ffi::c_int {
        from_result(|| {
            let cons = Console::from_ptr(co);
            T::exit::<A>(cons)
        })
    }

    unsafe extern "C" fn match_<A>(
        co: *mut bindings::console,
        name: *mut i8,
        idx: i32,
        options: *mut i8,
    ) -> core::ffi::c_int {
        from_result(|| {
            let cons = Console::from_ptr(co);
            let name = unsafe { core::slice::from_raw_parts(name as *const u8, 16) };
            let options = unsafe { core::slice::from_raw_parts(options as *const u8, 16) };
            T::match_::<A>(cons, name, idx, options)
        })
    }
}
