// Implementation borrowed from d3d12-rs as part of the gfx project.
// https://github.com/gfx-rs/d3d12-rs/blob/master/src/com.rs
//
// LICENSE MIT - https://github.com/gfx-rs/gfx/blob/master/LICENSE-MIT
//
// Permission is hereby granted, free of charge, to any
// person obtaining a copy of this software and associated
// documentation files (the "Software"), to deal in the
// Software without restriction, including without
// limitation the rights to use, copy, modify, merge,
// publish, distribute, sublicense, and/or sell copies of
// the Software, and to permit persons to whom the Software
// is furnished to do so, subject to the following
// conditions:
//
// The above copyright notice and this permission notice
// shall be included in all copies or substantial portions
// of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF
// ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED
// TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A
// PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT
// SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
// CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
// OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR
// IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
// DEALINGS IN THE SOFTWARE.
//

pub use winapi::shared::winerror::HRESULT;
pub type D3DResult<T> = (T, HRESULT);

use std::{
	fmt,
	hash::{
		Hash,
		Hasher,
	},
	ops::Deref,
	ptr,
};
use winapi::{
	ctypes::c_void,
	um::unknwnbase::IUnknown,
	Interface,
};

#[repr(transparent)]
pub struct WeakPtr<T>(*mut T);

impl<T> WeakPtr<T>
{
	pub fn null() -> Self
	{
		WeakPtr(ptr::null_mut())
	}

	pub unsafe fn from_raw(raw : *mut T) -> Self
	{
		WeakPtr(raw)
	}

	pub fn is_null(&self) -> bool
	{
		self.0.is_null()
	}

	pub fn as_ptr(&self) -> *const T
	{
		self.0
	}

	pub fn as_mut_ptr(&self) -> *mut T
	{
		self.0
	}

	pub unsafe fn mut_void(&mut self) -> *mut *mut c_void
	{
		&mut self.0 as *mut *mut _ as *mut *mut _
	}
}

impl<T : Interface> WeakPtr<T>
{
	pub unsafe fn as_unknown(&self) -> &IUnknown
	{
		debug_assert!(!self.is_null());
		&*(self.0 as *mut IUnknown)
	}

	// Cast creates a new WeakPtr requiring explicit destroy call.
	pub unsafe fn cast<U>(&self) -> D3DResult<WeakPtr<U>>
	where
		U : Interface,
	{
		let mut obj = WeakPtr::<U>::null();
		let hr = self.as_unknown().QueryInterface(&U::uuidof(), obj.mut_void());
		(obj, hr)
	}

	// Destroying one instance of the WeakPtr will invalidate all
	// copies and clones.
	pub unsafe fn destroy(&self)
	{
		self.as_unknown().Release();
	}
}

impl<T> Clone for WeakPtr<T>
{
	fn clone(&self) -> Self
	{
		WeakPtr(self.0)
	}
}

impl<T> Copy for WeakPtr<T> {}

impl<T> Deref for WeakPtr<T>
{
	type Target = T;

	fn deref(&self) -> &T
	{
		debug_assert!(!self.is_null());
		unsafe { &*self.0 }
	}
}

impl<T> fmt::Debug for WeakPtr<T>
{
	fn fmt(&self, f : &mut fmt::Formatter) -> fmt::Result
	{
		write!(f, "WeakPtr( ptr: {:?} )", self.0)
	}
}

impl<T> PartialEq<*mut T> for WeakPtr<T>
{
	fn eq(&self, other : &*mut T) -> bool
	{
		self.0 == *other
	}
}

impl<T> PartialEq for WeakPtr<T>
{
	fn eq(&self, other : &Self) -> bool
	{
		self.0 == other.0
	}
}

impl<T> Hash for WeakPtr<T>
{
	fn hash<H : Hasher>(&self, state : &mut H)
	{
		self.0.hash(state);
	}
}
