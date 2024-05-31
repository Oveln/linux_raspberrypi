// SPDX-License-Identifier: GPL-2.0

//! Unified device property interface.
//!
//! C header: [`include/linux/property.h`](../../../../include/linux/property.h)

use crate::{bindings, 
            device, 
            types::{ARef, AlwaysRefCounted, Opaque},
            str::CStr,
            error::Result,
};
use core::ptr;

/// Represents a `struct fwnode_handle *` part of a device's fwnode graph
///
/// # Invariants
///
/// The pointer is valid.
#[repr(transparent)]
pub struct FwNodeHandle(Opaque<bindings::fwnode_handle>);

// SAFETY: The type invariants guarantee that `FwNodeHandle` is always ref-counted.
unsafe impl AlwaysRefCounted for FwNodeHandle {
    fn inc_ref(&self) {
        // SAFETY: The existence of a shared reference means that the refcount is nonzero.
        unsafe { bindings::fwnode_handle_get(self.as_ptr()) };
    }

    unsafe fn dec_ref(obj: ptr::NonNull<Self>) {
        // SAFETY: The safety requirements guarantee that the refcount is nonzero.
        unsafe { bindings::fwnode_handle_put(obj.cast().as_ptr()) }
    }
}

impl FwNodeHandle {
    /// Creates a new FwNodeHandle instance from an existing [`device::Device`] instance.
    pub fn from(dev: &device::Device) -> Option<ARef<Self>> {
        // SAFETY: By the type invariants, `dev` owns a reference, so it safe to access `ptr`.
        // `fwnode_handle_get` increments the refcount of the `fwnode_handle` if it is not
        // NULL or returns NULL.
        let node = unsafe { bindings::fwnode_handle_get(bindings::__dev_fwnode(dev.ptr)) };
        let ptr = ptr::NonNull::new(node)?;
        // SAFETY: `fwnode_handle_get` increments the refcount
        Some(unsafe { ARef::from_raw(ptr.cast()) })
    }

    /// Returns a raw pointer to the inner C struct.
    #[inline]
    pub fn as_ptr(&self) -> *mut bindings::fwnode_handle {
        self.0.get()
    }

    /// Creates an instance of `NodeIterator`
    ///
    /// This provides an `Iterator` wrapping the internal C functionality of invoking
    /// `fwnode_graph_get_next_endpoint`
    pub fn endpoints(node: ARef<Self>) -> NodeIterator {
        NodeIterator {
            handle: node,
            curr_node: None,
            next_fn: next_endpoint,
        }
    }
    /// Gets the parent to the current `FwNodeHandle` of the device tree
    pub fn parent(node: ARef<Self>) -> Option<ARef<Self>> {
        // SAFETY: By the type invariants, `node` owns a reference, so it safe to the underlying
        // ptr.
        let node = unsafe { bindings::fwnode_get_parent(node.as_ptr()) };
        let ptr = ptr::NonNull::new(node)?;
        // SAFETY: `fwnode_get_parent` increments the refcount
        Some(unsafe { ARef::from_raw(ptr.cast()) })
    }

    /// Creates an instance of `NodeIterator`
    ///
    /// This provides an `Iterator` wrapping the internal C functionality of invoking
    /// `fwnode_graph_get_next_child_node`
    pub fn children(node: ARef<Self>) -> NodeIterator {
        NodeIterator {
            handle: node,
            curr_node: None,
            next_fn: next_child,
        }
    }

    /// Get Device 
    pub fn child_count(node: ARef<Self>) -> Result<usize> {
        let mut count:usize = 0;
        let node_iter = FwNodeHandle::children(node.clone());
        for _ in node_iter {
            count += 1
        }
        Ok(count)
    }

    /// Used to check if a device tree node is a software node.
    pub fn is_softnode(node: ARef<Self>) -> bool {
        // SAFETY: By the type invariants, `node` owns a reference, so it safe to the underlying
        // ptr.
        unsafe { bindings::is_software_node(node.as_ptr()) }
    }

    /// Reads the value of a property named u32 type from the device tree.
    pub fn fwnode_property_read_u32(node: ARef<Self>, name: &'static CStr) -> Result<u32> {
        let mut val: u32 = 0;
        // SAFETY: `node` owns a reference, so it safe to the underlying ptr.
        unsafe { bindings::fwnode_property_read_u32_array(node.as_ptr(), name.as_char_ptr(), &mut val, 1); }
        Ok(val)
    }
}

type NodeIteratorFn = 
    fn(root: ARef<FwNodeHandle>, curr: &Option<ARef<FwNodeHandle>>) -> Option<ARef<FwNodeHandle>>;

/// Implements the Iterator trait to iterate the device's endpoints given the `FwNodeHandle`
pub struct NodeIterator {
    handle: ARef<FwNodeHandle>,
    curr_node: Option<ARef<FwNodeHandle>>,
    next_fn: NodeIteratorFn,
}
impl Iterator for NodeIterator {
    type Item = ARef<FwNodeHandle>;

    fn next(&mut self) -> Option<Self::Item> {
        self.curr_node = (self.next_fn)(self.handle.clone(), &self.curr_node);
        self.curr_node.clone()
    }
}

fn next_endpoint(node: ARef<FwNodeHandle>, curr: &Option<ARef<FwNodeHandle>>) -> Option<ARef<FwNodeHandle>> {
    let res_ptr = match curr {
        // SAFETY: By the type invariants, `node` has a refcount > 1, so it is safe to access the
        // underlying ptr
        None => unsafe {
            bindings::fwnode_graph_get_next_endpoint(
                node.as_ptr(),
                ptr::null_mut::<bindings::fwnode_handle>(),
            )
        },
        // SAFETY: By the type invariants, `node` has a refcount > 1, so it is safe to access the
        // underlying ptr. `curr`, by the type invariants has a refcount > 1, hence its safe to access the
        // it's underlying ptr
        Some(curr) => unsafe {
            bindings::fwnode_graph_get_next_endpoint(node.as_ptr(), curr.as_ptr())
        },
    };
    let ptr = ptr::NonNull::new(res_ptr)?;
    // SAFETY: `fwnode_graph_get_next_endpoint` increments the refcount before returning
    Some(unsafe { ARef::from_raw(ptr.cast()) })
}

fn next_child(node: ARef<FwNodeHandle>, curr: &Option<ARef<FwNodeHandle>>) -> Option<ARef<FwNodeHandle>> {
    let res_ptr = match curr {
        // SAFETY: By the type invariants, `node` has a refcount > 1, so it is safe to access the
        // underlying ptr
        None => unsafe {
            bindings::fwnode_get_next_child_node(
                node.as_ptr(),
                ptr::null_mut::<bindings::fwnode_handle>(),
            )
        },
        // SAFETY: By the type invariants, `node` has a refcount > 1, so it is safe to access the
        // underlying ptr. `curr`, by the type invariants has a refcount > 1, hence its safe to access the
        // it's underlying ptr
        Some(curr) => unsafe { bindings::fwnode_get_next_child_node(node.as_ptr(), curr.as_ptr()) },
    };
    let ptr = ptr::NonNull::new(res_ptr)?;
    // SAFETY: `fwnode_graph_get_next_child_node` increments the refcount before returning
    Some(unsafe { ARef::from_raw(ptr.cast()) })
}

