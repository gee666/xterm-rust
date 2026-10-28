// Copyright (c) 2026 The xterm.js authors. All rights reserved.
// Licensed under MIT. Blob-like immutable data without a browser dependency.

use crate::image::storage::StorageError;
use std::any::Any;
use std::collections::TryReserveError;
use std::fmt;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;

pub type ByteRead = Pin<Box<dyn Future<Output = Result<Vec<u8>, StorageError>>>>;

/// Browser hosts retain real Blob data outside WASM linear memory. Reads own
/// their result, like Blob.arrayBuffer, without mutable protocol/map borrows.
pub trait EncodedSource: Any {
    fn size(&self) -> usize;
    fn mime(&self) -> &str;
    fn read(self: Rc<Self>) -> ByteRead;
    fn as_any(&self) -> &dyn Any;
}

/// Capture synchronously before the caller releases/reuses its decoder. Owned
/// input permits host callbacks without a decoder/map borrow across the call.
/// Browser implementations create an actual Blob, not another WASM Vec store.
pub trait EncodedSourceFactory {
    fn capture_owned(&self, data: Vec<u8>, mime: &str) -> Result<EncodedBytes, StorageError>;
}

#[derive(Clone)]
pub struct EncodedBytes(Rc<dyn EncodedSource>);
impl EncodedBytes {
    pub fn from_source(source: Rc<dyn EncodedSource>) -> Self {
        Self(source)
    }
    pub fn from_owned(data: Vec<u8>) -> Self {
        Self::from_owned_with_mime(data, "")
    }
    pub fn from_owned_with_mime(data: Vec<u8>, mime: &str) -> Self {
        Self(Rc::new(MemorySource {
            data,
            mime: blob_mime(mime),
        }))
    }
    pub fn copy_from(data: &[u8]) -> Result<Self, TryReserveError> {
        let mut owned = Vec::new();
        owned.try_reserve_exact(data.len())?;
        owned.extend_from_slice(data);
        Ok(Self::from_owned(owned))
    }
    pub fn len(&self) -> usize {
        self.0.size()
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub fn mime(&self) -> &str {
        self.0.mime()
    }
    pub fn read(&self) -> ByteRead {
        Rc::clone(&self.0).read()
    }
    pub fn source<T: Any>(&self) -> Option<&T> {
        self.0.as_any().downcast_ref()
    }
    pub fn as_memory(&self) -> Option<&[u8]> {
        self.source::<MemorySource>()
            .map(|source| source.data.as_slice())
    }
}
impl fmt::Debug for EncodedBytes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Formatting an unrelated failure must not invoke host property getters.
        f.debug_struct("EncodedBytes").finish_non_exhaustive()
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct MemorySourceFactory;
impl EncodedSourceFactory for MemorySourceFactory {
    fn capture_owned(&self, data: Vec<u8>, mime: &str) -> Result<EncodedBytes, StorageError> {
        Ok(EncodedBytes::from_owned_with_mime(data, mime))
    }
}

struct MemorySource {
    data: Vec<u8>,
    mime: String,
}
impl EncodedSource for MemorySource {
    fn size(&self) -> usize {
        self.data.len()
    }
    fn mime(&self) -> &str {
        &self.mime
    }
    fn read(self: Rc<Self>) -> ByteRead {
        Box::pin(async move {
            let mut owned = Vec::new();
            owned
                .try_reserve_exact(self.data.len())
                .map_err(StorageError::host)?;
            owned.extend_from_slice(&self.data);
            Ok(owned)
        })
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

fn blob_mime(mime: &str) -> String {
    if mime.bytes().any(|byte| !(0x20..=0x7e).contains(&byte)) {
        String::new()
    } else {
        mime.to_ascii_lowercase()
    }
}
