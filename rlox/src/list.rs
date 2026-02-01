use std::alloc::{Layout, alloc, dealloc, handle_alloc_error, realloc};
use std::fmt::{Display, Error, Formatter};
use std::iter::{DoubleEndedIterator, FromIterator, IntoIterator, Iterator};
use std::mem;
use std::mem::ManuallyDrop;
use std::ops::{Deref, DerefMut, Drop};
use std::ptr::NonNull;
use std::slice;

// A handwritten Vec impl minus Send/Sync traits equipped
// https://doc.rust-lang.org/nomicon/vec/vec-layout.html

pub struct List<T> {
    // Note that if we had used `*mut T` instead we would
    // be invariant over T.
    // This is the main reason behind using `NonNull<T>`
    // We additionally get null pointer optimisation with NonNull
    arr: NonNull<T>,
    cap: usize,
    len: usize,
}

impl<T> List<T> {
    pub fn new() -> Self {
        assert!(mem::size_of::<T>() != 0, "no support for ZSTs yet!");
        List {
            arr: NonNull::dangling(),
            cap: 0,
            len: 0,
        }
    }

    fn grow(&mut self) {
        // compute new cap and corresponding layout to describe
        // the allocation
        let (new_cap, new_layout_result) = if self.cap == 0 {
            (1, Layout::array::<T>(1))
        } else {
            let new_cap = self.cap * 2;
            (new_cap, Layout::array::<T>(new_cap))
        };

        let new_layout = new_layout_result.expect("capacity always fits in 1..isize::MAX");

        // create new raw arr ptr
        let new_arr_ptr = if self.cap == 0 {
            unsafe { alloc(new_layout) }
        } else {
            let old_layout = Layout::array::<T>(self.cap).unwrap();
            unsafe { realloc(self.arr.as_ptr() as *mut u8, old_layout, new_layout.size()) }
        } as *mut T;

        // create new NonNull arr
        self.arr = match NonNull::new(new_arr_ptr) {
            Some(new_arr) => new_arr,
            None => handle_alloc_error(new_layout),
        };

        self.cap = new_cap;
    }

    pub fn push(&mut self, elem: T) {
        if self.len == self.cap {
            self.grow();
        }

        unsafe {
            // Pointer arithmetic is more verbose in rust
            // Writes to uninitialised memory (even if allocated) cannot use C-style
            // dereferencing

            // *self.arr.add(self.len).as_ptr() = elem;
            // The above will not work for this reason

            self.arr.add(self.len).write(elem);
        }

        self.len += 1;
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            None
        } else {
            self.len = self.len.checked_sub(1)?;
            Some(unsafe {
                // *self.arr.add(self.len).as_ptr()
                // Even though the above expression is *logically* equivalent
                // to the below code, the compiler is unaware of the semantics
                // between self.arr and self.len

                // It may be possible that there is a hole by virtue of moving,
                // so use-after-free or double-free issues are implied

                // Hence we use `read` which performs a bitwise copy AND
                // acknowledges that arr[len] is logically uninitialised

                self.arr.add(self.len).read()
            })
        }
    }
}

pub struct ListIter<T> {
    arr: NonNull<T>,
    cap: usize,
    start: *const T,
    end: *const T,
}

impl<T> Iterator for ListIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        if self.start == self.end {
            None
        } else {
            unsafe {
                let item = self.start.read();
                self.start = self.start.add(1);
                Some(item)
            }
        }
    }
}

impl<T> DoubleEndedIterator for ListIter<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.end == self.start {
            None
        } else {
            unsafe {
                self.end = self.end.sub(1);
                Some(self.end.read())
            }
        }
    }
}

impl<T> FromIterator<T> for List<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut list = List::new();

        for it in iter {
            list.push(it);
        }

        list
    }
}

impl<T> IntoIterator for List<T> {
    type Item = T;
    type IntoIter = ListIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        let list = ManuallyDrop::new(self);
        let arr = list.arr;
        let cap = list.cap;
        let len = list.len;

        ListIter {
            arr,
            cap,
            start: arr.as_ptr(),
            end: if cap == 0 {
                arr.as_ptr()
            } else {
                unsafe { arr.as_ptr().add(len) }
            },
        }
    }
}

impl<T> Drop for ListIter<T> {
    fn drop(&mut self) {
        if self.cap > 0 {
            // Free the allocation of each element (if of type: impl Drop)
            for _ in &mut *self {}

            // Free the container array allocation
            let layout = Layout::array::<T>(self.cap).expect("Should always succeed");
            unsafe {
                dealloc(self.arr.as_ptr() as *mut u8, layout);
            }
        }
    }
}

impl<T> Drop for List<T> {
    fn drop(&mut self) {
        while self.pop().is_some() {}

        // NonNull::dangling() yields a pointer, albeit invalid.
        // This avoids freeing a dangling pointer.
        if self.cap == 0 {
            return;
        }

        let layout = Layout::array::<T>(self.cap).unwrap();
        unsafe {
            dealloc(self.arr.as_ptr() as *mut u8, layout);
        }
    }
}

impl<T> Deref for List<T> {
    type Target = [T];
    fn deref(&self) -> &Self::Target {
        unsafe {
            /* Introducing provenance: a pointer is more than (just) a number...

             tl;dr
             - It is a contract on a known set of addresses (currently contigous) relative to an "Original Pointer"
             returned from a memory allocation
             - The contract permits only accesses within this set, and the access must match the permission "tag"
             i.e., in order to do a read / write from / to a location

             Provenance in this unsafe block rejects the use of NonNull::as_ptr aka ptr::as_ptr(), because
             this function "shrinks" (think narrowing permissions when deriving a new capability) the provenance
             to only access the first element of the backing array. Using this method results in the
             following error (last logged 28/01/26).

             error: Undefined Behavior: trying to retag from <28666> for SharedReadOnly permission at alloc695[0x8],
             but that tag does not exist in the borrow stack for this location
                --> src/containers.rs:117:13
                    |
                    |             slice::from_raw_parts(self.arr.as_ref(), self.len)
                    |             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ this error occurs as part of retag at alloc695[0x0..0x28]
                    |

             Instead, we want to expose read access to all elements via the slice, which is done via ptr::as_ptr
            */
            slice::from_raw_parts(self.arr.as_ptr().cast_const(), self.len)
        }
    }
}

impl<T> DerefMut for List<T> {
    fn deref_mut(&mut self) -> &mut [T] {
        unsafe {
            /* TODO: understand why self.arr.as_mut() causes
                error: Undefined Behavior: trying to retag from <58311> for Unique permission at alloc695[0x8],
                but that tag does not exist in the borrow stack for this location
                        |
                        |             slice::from_raw_parts_mut(self.arr.as_mut(), self.len)
                        |             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ this error occurs as part of retag at alloc695[0x0..0x28]
            */
            slice::from_raw_parts_mut(self.arr.as_ptr(), self.len)
        }
    }
}

impl<T> Display for List<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "[")?;
        if self.len == 0 {
            write!(f, "]")
        } else {
            unsafe {
                // we use references here to avoid invoking
                // any implicit dropping of the T instance
                // at the end of write!()
                write!(f, "{}", self.arr.as_ref())?;
                for i in 1..self.len {
                    write!(f, ", {}", &self.arr.add(i).as_ref())?;
                }
            }
            write!(f, "]")
        }
    }
}
