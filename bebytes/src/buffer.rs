//! Internal buffer management for `BeBytes`
//!
//! This module provides efficient buffer types for serialization without external dependencies.
//! The types are designed to be API-compatible with the previous bytes crate implementation
//! while being simpler and more focused on `BeBytes`' actual needs.

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
#[cfg(feature = "std")]
use std::vec::Vec;

/// Trait for types that can write bytes efficiently.
///
/// This trait provides methods for writing primitive types and byte slices
/// to a buffer in an efficient manner, avoiding unnecessary allocations.
pub trait BufMut {
    fn put_u8(&mut self, val: u8);
    fn put_u16(&mut self, val: u16);
    fn put_u16_le(&mut self, val: u16);
    fn put_u32(&mut self, val: u32);
    fn put_u32_le(&mut self, val: u32);
    fn put_u64(&mut self, val: u64);
    fn put_u64_le(&mut self, val: u64);
    fn put_u128(&mut self, val: u128);
    fn put_u128_le(&mut self, val: u128);
    fn put_slice(&mut self, src: &[u8]);
    fn extend_from_slice(&mut self, src: &[u8]);
    fn reserve(&mut self, additional: usize);
    fn remaining_mut(&self) -> usize;
    fn chunk_mut(&mut self) -> &mut [u8];
    fn advance_mut(&mut self, n: usize);
}

/// A growable byte buffer optimized for writing.
///
/// `BytesMut` is a thin wrapper around `Vec<u8>` that provides
/// buffer-oriented methods for efficient serialization.
pub struct BytesMut {
    inner: Vec<u8>,
}

impl BytesMut {
    #[inline]
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: Vec::with_capacity(capacity),
        }
    }

    #[inline]
    #[must_use]
    pub fn capacity(&self) -> usize {
        self.inner.capacity()
    }

    #[inline]
    pub fn clear(&mut self) {
        self.inner.clear();
    }

    #[inline]
    pub fn resize(&mut self, new_len: usize, value: u8) {
        self.inner.resize(new_len, value);
    }

    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.inner.reserve(additional);
    }

    #[inline]
    pub fn extend_from_slice(&mut self, src: &[u8]) {
        self.inner.extend_from_slice(src);
    }

    #[inline]
    #[must_use]
    pub fn to_vec(self) -> Vec<u8> {
        self.inner
    }

    /// Convert to an immutable Bytes buffer.
    ///
    /// This moves the internal `Vec<u8>` without copying, providing
    /// an efficient way to finalize the buffer after writing.
    #[inline]
    #[must_use]
    pub fn freeze(self) -> Bytes {
        Bytes { inner: self.inner }
    }
}

impl BufMut for BytesMut {
    #[inline]
    fn put_u8(&mut self, val: u8) {
        self.inner.push(val);
    }

    #[inline]
    fn put_u16(&mut self, val: u16) {
        self.inner.extend_from_slice(&val.to_be_bytes());
    }

    #[inline]
    fn put_u16_le(&mut self, val: u16) {
        self.inner.extend_from_slice(&val.to_le_bytes());
    }

    #[inline]
    fn put_u32(&mut self, val: u32) {
        self.inner.extend_from_slice(&val.to_be_bytes());
    }

    #[inline]
    fn put_u32_le(&mut self, val: u32) {
        self.inner.extend_from_slice(&val.to_le_bytes());
    }

    #[inline]
    fn put_u64(&mut self, val: u64) {
        self.inner.extend_from_slice(&val.to_be_bytes());
    }

    #[inline]
    fn put_u64_le(&mut self, val: u64) {
        self.inner.extend_from_slice(&val.to_le_bytes());
    }

    #[inline]
    fn put_u128(&mut self, val: u128) {
        self.inner.extend_from_slice(&val.to_be_bytes());
    }

    #[inline]
    fn put_u128_le(&mut self, val: u128) {
        self.inner.extend_from_slice(&val.to_le_bytes());
    }

    #[inline]
    fn put_slice(&mut self, src: &[u8]) {
        self.inner.extend_from_slice(src);
    }

    #[inline]
    fn extend_from_slice(&mut self, src: &[u8]) {
        self.inner.extend_from_slice(src);
    }

    #[inline]
    fn reserve(&mut self, additional: usize) {
        self.inner.reserve(additional);
    }

    #[inline]
    fn remaining_mut(&self) -> usize {
        usize::MAX - self.inner.len()
    }

    #[inline]
    fn chunk_mut(&mut self) -> &mut [u8] {
        // Return a mutable slice to the uninitialized part of the buffer
        // We need to ensure there's enough capacity
        let len = self.inner.len();
        let cap = self.inner.capacity();
        if len < cap {
            unsafe { core::slice::from_raw_parts_mut(self.inner.as_mut_ptr().add(len), cap - len) }
        } else {
            &mut []
        }
    }

    #[inline]
    fn advance_mut(&mut self, n: usize) {
        let new_len = self.inner.len() + n;
        assert!(new_len <= self.inner.capacity());
        unsafe {
            self.inner.set_len(new_len);
        }
    }
}

impl core::ops::Deref for BytesMut {
    type Target = [u8];

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl core::ops::DerefMut for BytesMut {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

/// An immutable byte buffer.
///
/// `Bytes` wraps a `Vec<u8>` and provides an immutable view of the data.
/// This type maintains API compatibility with the previous bytes crate implementation
/// while being simpler and avoiding unnecessary reference counting.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bytes {
    inner: Vec<u8>,
}

impl Bytes {
    /// Create a new `Bytes` instance from a slice by copying the data.
    #[inline]
    #[must_use]
    pub fn copy_from_slice(data: &[u8]) -> Self {
        Self {
            inner: data.to_vec(),
        }
    }
}

impl From<Vec<u8>> for Bytes {
    #[inline]
    fn from(vec: Vec<u8>) -> Self {
        Self { inner: vec }
    }
}

impl From<Bytes> for Vec<u8> {
    #[inline]
    fn from(bytes: Bytes) -> Self {
        bytes.inner
    }
}

impl AsRef<[u8]> for Bytes {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        &self.inner
    }
}

impl core::ops::Deref for Bytes {
    type Target = [u8];

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

// Also implement BufMut for Vec<u8> directly for compatibility
impl BufMut for Vec<u8> {
    #[inline]
    fn put_u8(&mut self, val: u8) {
        self.push(val);
    }

    #[inline]
    fn put_u16(&mut self, val: u16) {
        self.extend_from_slice(&val.to_be_bytes());
    }

    #[inline]
    fn put_u16_le(&mut self, val: u16) {
        self.extend_from_slice(&val.to_le_bytes());
    }

    #[inline]
    fn put_u32(&mut self, val: u32) {
        self.extend_from_slice(&val.to_be_bytes());
    }

    #[inline]
    fn put_u32_le(&mut self, val: u32) {
        self.extend_from_slice(&val.to_le_bytes());
    }

    #[inline]
    fn put_u64(&mut self, val: u64) {
        self.extend_from_slice(&val.to_be_bytes());
    }

    #[inline]
    fn put_u64_le(&mut self, val: u64) {
        self.extend_from_slice(&val.to_le_bytes());
    }

    #[inline]
    fn put_u128(&mut self, val: u128) {
        self.extend_from_slice(&val.to_be_bytes());
    }

    #[inline]
    fn put_u128_le(&mut self, val: u128) {
        self.extend_from_slice(&val.to_le_bytes());
    }

    #[inline]
    fn put_slice(&mut self, src: &[u8]) {
        self.extend_from_slice(src);
    }

    #[inline]
    fn extend_from_slice(&mut self, src: &[u8]) {
        Vec::extend_from_slice(self, src);
    }

    #[inline]
    fn reserve(&mut self, additional: usize) {
        Vec::reserve(self, additional);
    }

    #[inline]
    fn remaining_mut(&self) -> usize {
        usize::MAX - self.len()
    }

    #[inline]
    fn chunk_mut(&mut self) -> &mut [u8] {
        let len = self.len();
        let cap = self.capacity();
        if len < cap {
            unsafe { core::slice::from_raw_parts_mut(self.as_mut_ptr().add(len), cap - len) }
        } else {
            &mut []
        }
    }

    #[inline]
    fn advance_mut(&mut self, n: usize) {
        let new_len = self.len() + n;
        assert!(new_len <= self.capacity());
        unsafe {
            self.set_len(new_len);
        }
    }
}
