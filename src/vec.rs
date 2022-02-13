/*
 * Vec implementation
 *
 * Credit: The Rustonomicon
 */

use std::alloc::{self, Layout};
use std::marker::PhantomData;
use std::mem;
use std::ops::{Deref, DerefMut};
use std::ptr::{self, NonNull};

pub struct Vec<T> {
    // Memory location of this structure's array of `T`s
    ptr: NonNull<T>,
    // The maximum number `T`s this Vec can hold without having to reallocate
    // Allocations are restricted to `isize::MAX` elements, hence we manually
    // ensure, for now, that `cap <= isize::MAX`.
    cap: usize,
    // The actual number of `T`s currently being stored
    len: usize,
    // Rust nonsense to indicate satisfy the drop checker
    _marker: PhantomData<T>,
}

/* We must ensure the automatic derivation of Send/Sync is well-defined */
unsafe impl<T: Send> Send for Vec<T> {}
unsafe impl<T: Sync> Sync for Vec<T> {}

impl<T> Vec<T> {
    /*
     * Create an empty Vec.
     */
    fn new() -> Self {
        assert!(mem::size_of::<T>() != 0, "Zero-size types unsupported.");

        /* Create an empty Vec */
        return Vec {
            // This pointer should never be dereferenced. This is a workaround
            // of using NULL. We shall always check for that cap,len != 0 before
            // dereferencing.
            ptr: NonNull::dangling(),
            cap: 0,
            len: 0,
            _marker: PhantomData,
        };
    }

    /*
     * Allocate more memory for the Vec. Just allocates, so `self.len` is
     * not changed by this function.
     */
    fn grow(&mut self) {
        let (new_cap, new_layout): (usize, Layout);
        if self.cap == 0 {
            // empty Vec
            // Initial size of (initialized) Vec
            new_cap = 1;
            new_layout = Layout::array::<T>(new_cap).unwrap();
        } else {
            new_cap = 2 * self.cap;
            // Safe to unwrap based on our restriction `self.cap <= isize::MAX`
            new_layout = Layout::array::<T>(new_cap).unwrap();
        }

        // Manual verification of maximum allocation size
        assert!(
            new_cap <= (isize::MAX as usize),
            "Tried to allocate too much memory."
        );

        /* Allocate memory, check if successful */
        let new_ptr: *mut u8;
        if self.cap == 0 {
            unsafe {
                new_ptr = alloc::alloc(new_layout);
            }
        } else {
            let old_layout = Layout::array::<T>(self.cap).unwrap();
            let old_ptr = self.ptr.as_ptr() as *mut u8;
            unsafe {
                new_ptr = alloc::realloc(old_ptr, old_layout, new_layout.size());
            }
        }

        match NonNull::new(new_ptr as *mut T) {
            None => {
                alloc::handle_alloc_error(new_layout);
            }
            Some(p) => {
                self.ptr = p;
            }
        }

        self.cap = new_cap;
    }

    /*
     * Append an element to the Vec
     */
    pub fn push(&mut self, elem: T) {
        // Allocate more memory if necessary
        if self.len == self.cap {
            self.grow();
        }

        // Write the new element to memory
        unsafe {
            // We use this function to write as it avoides implicitly
            // reading unitialized memory. Indeed, something like
            // `ptr[idx] = x` would tell Rust to call Drop on the
            // previous values of `ptr[idx]`, even though this memory
            // may not be initialized.
            ptr::write(self.ptr.as_ptr().add(self.len), elem);
        }

        self.len += 1;
    }

    /*
     * Remove the last element of the Vec. This function returns the new
     * last element of the Vec.
     */
    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }

        self.len -= 1;
        unsafe {
            return Some(ptr::read(self.ptr.as_ptr().add(self.len)));
        }
    }
}

impl<T> Drop for Vec<T> {
    fn drop(&mut self) {
        if self.cap == 0 {
            return;
        }

        /* Pop elements until none left */
        loop {
            match self.pop() {
                None => {
                    break;
                }
                Some(_) => {}
            }
        }

        /* Deallocate memory */
        let layout = Layout::array::<T>(self.cap).unwrap();
        unsafe {
            alloc::dealloc(self.ptr.as_ptr() as *mut u8, layout);
        }
    }
}

impl<T> Deref for Vec<T> {
    type Target = [T];
    fn deref(&self) -> &[T] {
        unsafe {
            return std::slice::from_raw_parts(self.ptr.as_ptr(), self.len);
        }
    }
}

impl<T> DerefMut for Vec<T> {
    fn deref_mut(&mut self) -> &mut [T] {
        unsafe {
            return std::slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len);
        }
    }
}
