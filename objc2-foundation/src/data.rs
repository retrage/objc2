#[cfg(feature = "block")]
use alloc::vec::Vec;
use core::ops::{Index, IndexMut, Range};
use core::slice::{self, SliceIndex};
use core::{ffi::c_void, ptr::NonNull};
use std::io;

use objc2::msg_send;
use objc2::rc::{DefaultId, Id, Owned, Shared};
use objc2::runtime::{Class, Object};

use super::{NSCopying, NSMutableCopying, NSObject, NSRange};

object! {
    /// A static byte buffer in memory.
    ///
    /// This is similar to a [`slice`][`prim@slice`] of [`u8`].
    ///
    /// See [Apple's documentation](https://developer.apple.com/documentation/foundation/nsdata?language=objc).
    unsafe pub struct NSData: NSObject;
}

// TODO: SAFETY
unsafe impl Sync for NSData {}
unsafe impl Send for NSData {}

object! {
    /// A dynamic byte buffer in memory.
    ///
    /// This is the Objective-C equivalent of a [`Vec`] containing [`u8`].
    ///
    /// See [Apple's documentation](https://developer.apple.com/documentation/foundation/nsmutabledata?language=objc).
    ///
    /// [`Vec`]: std::vec::Vec
    unsafe pub struct NSMutableData: NSData;
}

// TODO: SAFETY
unsafe impl Sync for NSMutableData {}
unsafe impl Send for NSMutableData {}

impl NSData {
    unsafe_def_fn!(fn new -> Shared);

    #[doc(alias = "length")]
    pub fn len(&self) -> usize {
        unsafe { msg_send![self, length] }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn bytes(&self) -> &[u8] {
        let ptr: *const c_void = unsafe { msg_send![self, bytes] };
        // The bytes pointer may be null for length zero
        if ptr.is_null() {
            &[]
        } else {
            unsafe { slice::from_raw_parts(ptr as *const u8, self.len()) }
        }
    }

    pub fn with_bytes(bytes: &[u8]) -> Id<Self, Shared> {
        unsafe { Id::new(data_with_bytes(Self::class(), bytes).cast()) }
    }

    #[cfg(feature = "block")]
    pub fn from_vec(bytes: Vec<u8>) -> Id<Self, Shared> {
        // GNUStep's NSData `initWithBytesNoCopy:length:deallocator:` has a
        // bug; it forgets to assign the input buffer and length to the
        // instance before it swizzles to NSDataWithDeallocatorBlock.
        // See https://github.com/gnustep/libs-base/pull/213
        // So we just use NSDataWithDeallocatorBlock directly.
        //
        // NSMutableData does not have this problem.
        #[cfg(gnustep)]
        let cls = objc2::class!(NSDataWithDeallocatorBlock);
        #[cfg(not(gnustep))]
        let cls = Self::class();

        unsafe { Id::new(data_from_vec(cls, bytes).cast()) }
    }
}

unsafe impl NSCopying for NSData {
    type Ownership = Shared;
    type Output = NSData;
}

unsafe impl NSMutableCopying for NSData {
    type Output = NSMutableData;
}

impl AsRef<[u8]> for NSData {
    fn as_ref(&self) -> &[u8] {
        self.bytes()
    }
}

impl<I: SliceIndex<[u8]>> Index<I> for NSData {
    type Output = I::Output;

    #[inline]
    fn index(&self, index: I) -> &Self::Output {
        // Replaces the need for getBytes:range:
        Index::index(self.bytes(), index)
    }
}

impl DefaultId for NSData {
    type Ownership = Shared;

    #[inline]
    fn default_id() -> Id<Self, Self::Ownership> {
        Self::new()
    }
}

/// Creation methods
impl NSMutableData {
    unsafe_def_fn!(fn new -> Owned);

    pub fn with_bytes(bytes: &[u8]) -> Id<Self, Owned> {
        unsafe { Id::new(data_with_bytes(Self::class(), bytes).cast()) }
    }

    #[cfg(feature = "block")]
    pub fn from_vec(bytes: Vec<u8>) -> Id<Self, Owned> {
        unsafe { Id::new(data_from_vec(Self::class(), bytes).cast()) }
    }

    // TODO: Use malloc_buf/mbox and `initWithBytesNoCopy:...`?

    #[doc(alias = "initWithData:")]
    pub fn from_data(data: &NSData) -> Id<Self, Owned> {
        // Not provided on NSData, one should just use NSData::copy or similar
        unsafe {
            let obj: *mut Self = msg_send![Self::class(), alloc];
            let obj: *mut Self = msg_send![obj, initWithData: data];
            Id::new(NonNull::new_unchecked(obj))
        }
    }

    #[doc(alias = "initWithCapacity:")]
    pub fn with_capacity(capacity: usize) -> Id<Self, Owned> {
        unsafe {
            let obj: *mut Self = msg_send![Self::class(), alloc];
            let obj: *mut Self = msg_send![obj, initWithCapacity: capacity];
            Id::new(NonNull::new_unchecked(obj))
        }
    }
}

/// Mutation methods
impl NSMutableData {
    #[doc(alias = "mutableBytes")]
    pub fn bytes_mut(&mut self) -> &mut [u8] {
        let ptr: *mut c_void = unsafe { msg_send![self, mutableBytes] };
        // The bytes pointer may be null for length zero
        if ptr.is_null() {
            &mut []
        } else {
            unsafe { slice::from_raw_parts_mut(ptr as *mut u8, self.len()) }
        }
    }

    /// Expands with zeroes, or truncates the buffer.
    #[doc(alias = "setLength:")]
    pub fn set_len(&mut self, len: usize) {
        unsafe { msg_send![self, setLength: len] }
    }

    #[doc(alias = "appendBytes:length:")]
    pub fn extend_from_slice(&mut self, bytes: &[u8]) {
        let bytes_ptr = bytes.as_ptr() as *const c_void;
        unsafe { msg_send![self, appendBytes: bytes_ptr, length: bytes.len()] }
    }

    pub fn push(&mut self, byte: u8) {
        self.extend_from_slice(&[byte])
    }

    #[doc(alias = "replaceBytesInRange:withBytes:length:")]
    pub fn replace_range(&mut self, range: Range<usize>, bytes: &[u8]) {
        let range = NSRange::from(range);
        let ptr = bytes.as_ptr() as *const c_void;
        unsafe {
            msg_send![
                self,
                replaceBytesInRange: range,
                withBytes: ptr,
                length: bytes.len(),
            ]
        }
    }

    pub fn set_bytes(&mut self, bytes: &[u8]) {
        let len = self.len();
        self.replace_range(0..len, bytes);
    }
}

unsafe impl NSCopying for NSMutableData {
    type Ownership = Shared;
    type Output = NSData;
}

unsafe impl NSMutableCopying for NSMutableData {
    type Output = NSMutableData;
}

impl AsRef<[u8]> for NSMutableData {
    fn as_ref(&self) -> &[u8] {
        self.bytes()
    }
}

impl AsMut<[u8]> for NSMutableData {
    fn as_mut(&mut self) -> &mut [u8] {
        self.bytes_mut()
    }
}

impl<I: SliceIndex<[u8]>> Index<I> for NSMutableData {
    type Output = I::Output;

    #[inline]
    fn index(&self, index: I) -> &Self::Output {
        Index::index(self.bytes(), index)
    }
}

impl<I: SliceIndex<[u8]>> IndexMut<I> for NSMutableData {
    #[inline]
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        IndexMut::index_mut(self.bytes_mut(), index)
    }
}

// impl FromIterator<u8> for Id<NSMutableData, Owned> {
//     fn from_iter<T: IntoIterator<Item = u8>>(iter: T) -> Self {
//         let iter = iter.into_iter();
//         let (lower, _) = iter.size_hint();
//         let data = Self::with_capacity(lower);
//         for item in iter {
//             data.push(item);
//         }
//         data
//     }
// }

impl Extend<u8> for NSMutableData {
    /// You should use [`extend_from_slice`] whenever possible, it is more
    /// performant.
    ///
    /// [`extend_from_slice`]: Self::extend_from_slice
    fn extend<T: IntoIterator<Item = u8>>(&mut self, iter: T) {
        for item in iter {
            self.push(item);
        }
    }
}

impl io::Write for NSMutableData {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        self.extend_from_slice(buf);
        Ok(())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl DefaultId for NSMutableData {
    type Ownership = Owned;

    #[inline]
    fn default_id() -> Id<Self, Self::Ownership> {
        Self::new()
    }
}

unsafe fn data_with_bytes(cls: &Class, bytes: &[u8]) -> NonNull<Object> {
    let bytes_ptr = bytes.as_ptr() as *const c_void;
    unsafe {
        let obj: *mut Object = msg_send![cls, alloc];
        let obj: *mut Object = msg_send![
            obj,
            initWithBytes: bytes_ptr,
            length: bytes.len(),
        ];
        NonNull::new_unchecked(obj)
    }
}

#[cfg(feature = "block")]
unsafe fn data_from_vec(cls: &Class, bytes: Vec<u8>) -> NonNull<Object> {
    use core::mem::ManuallyDrop;

    use block2::{Block, ConcreteBlock};

    let capacity = bytes.capacity();

    let dealloc = ConcreteBlock::new(move |bytes: *mut c_void, len: usize| unsafe {
        // Recreate the Vec and let it drop
        let _ = Vec::from_raw_parts(bytes as *mut u8, len, capacity);
    });
    let dealloc = dealloc.copy();
    let dealloc: &Block<(*mut c_void, usize), ()> = &dealloc;

    let mut bytes = ManuallyDrop::new(bytes);

    unsafe {
        let obj: *mut Object = msg_send![cls, alloc];
        let obj: *mut Object = msg_send![
            obj,
            initWithBytesNoCopy: bytes.as_mut_ptr() as *mut c_void,
            length: bytes.len(),
            deallocator: dealloc,
        ];
        NonNull::new_unchecked(obj)
    }
}

#[cfg(test)]
mod tests {
    use super::{NSData, NSMutableData};
    #[cfg(feature = "block")]
    use alloc::vec;

    #[test]
    fn test_bytes() {
        let bytes = [3, 7, 16, 52, 112, 19];
        let data = NSData::with_bytes(&bytes);
        assert_eq!(data.len(), bytes.len());
        assert_eq!(data.bytes(), bytes);
    }

    #[test]
    fn test_no_bytes() {
        let data = NSData::new();
        assert!(Some(data.bytes()).is_some());
    }

    #[test]
    fn test_bytes_mut() {
        let mut data = NSMutableData::with_bytes(&[7, 16]);
        data.bytes_mut()[0] = 3;
        assert_eq!(data.bytes(), [3, 16]);
    }

    #[test]
    fn test_set_len() {
        let mut data = NSMutableData::with_bytes(&[7, 16]);
        data.set_len(4);
        assert_eq!(data.len(), 4);
        assert_eq!(data.bytes(), [7, 16, 0, 0]);

        data.set_len(1);
        assert_eq!(data.len(), 1);
        assert_eq!(data.bytes(), [7]);
    }

    #[test]
    fn test_append() {
        let mut data = NSMutableData::with_bytes(&[7, 16]);
        data.extend_from_slice(&[3, 52]);
        assert_eq!(data.len(), 4);
        assert_eq!(data.bytes(), [7, 16, 3, 52]);
    }

    #[test]
    fn test_replace() {
        let mut data = NSMutableData::with_bytes(&[7, 16]);
        data.replace_range(0..0, &[3]);
        assert_eq!(data.bytes(), [3, 7, 16]);

        data.replace_range(1..2, &[52, 13]);
        assert_eq!(data.bytes(), [3, 52, 13, 16]);

        data.replace_range(2..4, &[6]);
        assert_eq!(data.bytes(), [3, 52, 6]);

        data.set_bytes(&[8, 17]);
        assert_eq!(data.bytes(), [8, 17]);
    }

    #[cfg(feature = "block")]
    #[test]
    fn test_from_vec() {
        let bytes = vec![3, 7, 16];
        let bytes_ptr = bytes.as_ptr();

        let data = NSData::from_vec(bytes);
        assert_eq!(data.bytes().as_ptr(), bytes_ptr);
    }

    #[test]
    fn test_from_data() {
        let data = NSData::with_bytes(&[1, 2]);
        let mut_data = NSMutableData::from_data(&data);
        assert_eq!(&*data, &**mut_data);
    }

    #[test]
    fn test_with_capacity() {
        let mut data = NSMutableData::with_capacity(5);
        assert_eq!(data.bytes(), &[]);
        data.extend_from_slice(&[1, 2, 3, 4, 5]);
        assert_eq!(data.bytes(), &[1, 2, 3, 4, 5]);
        data.extend_from_slice(&[6, 7]);
        assert_eq!(data.bytes(), &[1, 2, 3, 4, 5, 6, 7]);
    }

    #[test]
    fn test_extend() {
        let iter = (3..=5).into_iter();
        let mut data = NSMutableData::with_bytes(&[1, 2]);
        data.extend(iter);
        assert_eq!(data.bytes(), &[1, 2, 3, 4, 5]);
    }
}
