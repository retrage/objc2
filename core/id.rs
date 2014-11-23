use std::fmt;
use std::hash;
use std::mem;

use {Message, ToMessage};

pub struct Owned;
pub struct Shared;

pub trait Ownership { }
impl Ownership for Owned { }
impl Ownership for Shared { }

/// A pointer type for Objective-C's reference counted objects. The object of
/// an `Id` is retained and sent a `release` message when the `Id` is dropped.
///
/// An `Id` may be either `Owned` or `Shared`, represented by the types `Id`
/// and `ShareId`, respectively. If owned, there are no other references to the
/// object and the `Id` can be mutably dereferenced. `ShareId`, however, can
/// only be immutably dereferenced because there may be other references to the
/// object, but a `ShareId` can be cloned to provide more references to the
/// object. An owned `Id` can be "downgraded" freely to a `ShareId`, but there
/// is no way to safely upgrade back.
#[unsafe_no_drop_flag]
pub struct Id<T: Message, O: Ownership = Owned> {
	ptr: *mut T,
}

impl<T: Message, O: Ownership> Id<T, O> {
	/// Constructs an `Id` from a pointer to an unretained object and
	/// retains it. Panics if the pointer is null.
	/// Unsafe because the pointer must be to a valid object and
	/// the caller must ensure the ownership is correct.
	pub unsafe fn from_ptr(ptr: *mut T) -> Id<T, O> {
		match Id::maybe_from_ptr(ptr) {
			Some(id) => id,
			None => panic!("Attempted to construct an Id from a null pointer"),
		}
	}

	/// Constructs an `Id` from a pointer to a retained object; this won't
	/// retain the pointer, so the caller must ensure the object has a +1
	/// retain count. Panics if the pointer is null.
	/// Unsafe because the pointer must be to a valid object and
	/// the caller must ensure the ownership is correct.
	pub unsafe fn from_retained_ptr(ptr: *mut T) -> Id<T, O> {
		match Id::maybe_from_retained_ptr(ptr) {
			Some(id) => id,
			None => panic!("Attempted to construct an Id from a null pointer"),
		}
	}

	/// Constructs an `Id` from a pointer to an unretained object and
	/// retains it if the pointer isn't null, otherwise returns None.
	/// Unsafe because the pointer must be to a valid object and
	/// the caller must ensure the ownership is correct.
	pub unsafe fn maybe_from_ptr(ptr: *mut T) -> Option<Id<T, O>> {
		// objc_msgSend is a no-op on null pointers
		msg_send![ptr retain];
		Id::maybe_from_retained_ptr(ptr)
	}

	/// Constructs an `Id` from a pointer to a retained object if the pointer
	/// isn't null, otherwise returns None. This won't retain the pointer,
	/// so the caller must ensure the object has a +1 retain count.
	/// Unsafe because the pointer must be to a valid object and
	/// the caller must ensure the ownership is correct.
	pub unsafe fn maybe_from_retained_ptr(ptr: *mut T) -> Option<Id<T, O>> {
		if ptr.is_null() {
			None
		} else {
			Some(Id { ptr: ptr })
		}
	}
}

impl<T: Message> Id<T, Owned> {
	/// "Downgrade" an owned `Id` to a `ShareId`, allowing it to be cloned.
	pub fn share(self) -> ShareId<T> {
		unsafe {
			mem::transmute(self)
		}
	}
}

impl<T: Message, O: Ownership> ToMessage<T> for Id<T, O> {
	fn as_ptr(&self) -> *mut T {
		self.ptr
	}
}

impl<T: Message> Clone for Id<T, Shared> {
	fn clone(&self) -> ShareId<T> {
		unsafe { Id::from_ptr(self.ptr) }
	}
}

#[unsafe_destructor]
impl<T: Message, O: Ownership> Drop for Id<T, O> {
	fn drop(&mut self) {
		if !self.ptr.is_null() {
			let ptr = mem::replace(&mut self.ptr, RawPtr::null());
			unsafe {
				msg_send![ptr release];
			}
		}
	}
}

impl<T: Message, O: Ownership> Deref<T> for Id<T, O> {
	fn deref(&self) -> &T {
		unsafe { &*self.ptr }
	}
}

impl<T: Message> DerefMut<T> for Id<T, Owned> {
	fn deref_mut(&mut self) -> &mut T {
		unsafe { &mut *self.ptr }
	}
}

impl<T: Message + PartialEq, O: Ownership> PartialEq for Id<T, O> {
	fn eq(&self, other: &Id<T, O>) -> bool {
		self.deref() == other.deref()
	}

	fn ne(&self, other: &Id<T, O>) -> bool {
		self.deref() != other.deref()
	}
}

impl<T: Message + Eq, O: Ownership> Eq for Id<T, O> { }

impl<T: Message + hash::Hash, O: Ownership> hash::Hash for Id<T, O> {
	fn hash(&self, state: &mut hash::sip::SipState) {
		self.deref().hash(state)
	}
}

impl<T: Message + fmt::Show, O: Ownership> fmt::Show for Id<T, O> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.deref().fmt(f)
	}
}

/// A convenient alias for a shared `Id`.
pub type ShareId<T> = Id<T, Shared>;

/// Extension methods for slices containing `Id`s.
pub trait IdVector<T> for Sized? {
	/// Convert a slice of `Id`s into a slice of references
	fn as_refs_slice(&self) -> &[&T];
}

impl<T: Message, O: Ownership> IdVector<T> for [Id<T, O>] {
	fn as_refs_slice(&self) -> &[&T] {
		unsafe {
			mem::transmute(self)
		}
	}
}

/// Trait to convert to a vector of `Id`s by consuming self.
pub trait IntoIdVector<T> {
	/// Converts to a vector of `Id`s by consuming self, retaining each object
	/// contained in self.
	/// Unsafe because the caller must ensure the `Id`s are constructed from
	/// valid objects and the ownership of the resulting `Id`s is correct.
	unsafe fn into_id_vec<O: Ownership>(self) -> Vec<Id<T, O>>;
}

impl<T: Message, R: ToMessage<T>> IntoIdVector<T> for Vec<R> {
	unsafe fn into_id_vec<O: Ownership>(self) -> Vec<Id<T, O>> {
		self.map_in_place(|obj| Id::from_ptr(obj.as_ptr()))
	}
}