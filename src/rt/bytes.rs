use std::{alloc::{alloc_zeroed, Layout}, ops::{Deref, DerefMut}, ptr, slice};

#[derive(Debug)]
#[repr(transparent)]
pub struct Bytes32Aligned {
    slice: [u32],
}
impl Bytes32Aligned {
    pub(crate) fn new_zeroed(len: usize) -> Box<Self> {
        assert!(len & 3 == 0);
        unsafe {
            let layout = Layout::from_size_align_unchecked(len, align_of::<u32>());
            let ptr = alloc_zeroed(layout);
            Box::from_raw(ptr::slice_from_raw_parts_mut(ptr as *mut u32, len / 4) as *mut Bytes32Aligned)
        }
    }
    pub const fn from_boxed_u32_slice(b: Box<[u32]>) -> Box<Self> {
        unsafe {
            std::mem::transmute(b)
        }
    }
    pub const fn len(&self) -> usize {
        self.slice.len() * size_of::<u32>()
    }
    pub fn as_u32_slice(&self) -> &[u32] {
        &self.slice
    }
    pub fn as_u32_slice_mut(&mut self) -> &mut [u32] {
        &mut self.slice
    }
    pub const fn into_boxed_u32_slice(self: Box<Self>) -> Box<[u32]> {
        unsafe {
            std::mem::transmute(self)
        }
    }
}
impl Deref for Bytes32Aligned {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        unsafe {
            slice::from_raw_parts(self.slice.as_ptr() as *const u8, self.len())
        }
    }
}
impl DerefMut for Bytes32Aligned {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            slice::from_raw_parts_mut(self.slice.as_mut_ptr() as *mut u8, self.len())
        }
    }
}
impl Clone for Box<Bytes32Aligned> {
    fn clone(&self) -> Self {
        let u32_slice_box = self.as_u32_slice().to_vec().into_boxed_slice();
        Bytes32Aligned::from_boxed_u32_slice(u32_slice_box)
    }
}
pub trait AsBytes32Aligned {
    fn as_bytes_32aligned(&self) -> &Bytes32Aligned;
    fn as_bytes_32aligned_mut(&mut self) -> &mut Bytes32Aligned;
}

impl AsBytes32Aligned for [u32] {
    fn as_bytes_32aligned(&self) -> &Bytes32Aligned {
        unsafe {
            std::mem::transmute(self)
        }
    }
    fn as_bytes_32aligned_mut(&mut self) -> &mut Bytes32Aligned {
        unsafe {
            std::mem::transmute(self)
        }
    }
}

pub trait VecU32AsBytes {
    /// Extend container from byte slice
    fn extend_from_bytes(&mut self, slice: &[u8]);
}

impl VecU32AsBytes for Vec<u32> {
    fn extend_from_bytes(&mut self, slice: &[u8]) {
        if slice.as_ptr() as usize & 3 == 0 {
            self.extend_from_slice(unsafe { slice::from_raw_parts(slice.as_ptr() as *const u32, slice.len() / 4) });
            if slice.len() & 3 > 0 {
                let mut last = [0u8; 4];
                for (a, b) in last.iter_mut().zip(slice[slice.len() & !3..].iter().copied()) {
                    *a = b;
                }
                self.push(u32::from_ne_bytes(last))
            }
        } else {
            for chunk in slice.chunks(4) {
                let mut chunk_as_u32 = [0u8; 4];
                chunk_as_u32[..chunk.len()].copy_from_slice(chunk);
                self.push(u32::from_ne_bytes(chunk_as_u32))
            }
        }
    }
}
