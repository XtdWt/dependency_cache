use pyo3::prelude::*;
use pyo3::types::PyTuple;
use std::collections::HashMap;

pub type CacheKey = u64;

pub struct MetadataHashManager {
    buckets: HashMap<isize, Vec<CacheKey>>,
    signatures: HashMap<CacheKey, Py<PyTuple>>,
    next_id: CacheKey,
}

impl MetadataHashManager {
    pub fn new() -> Self {
        Self { buckets: HashMap::new(), signatures: HashMap::new(), next_id: 0 }
    }

    pub fn get_cache_key(
        &self,
        py: Python<'_>,
        hash: &isize,
        sig: &Bound<'_, PyTuple>,
    ) -> PyResult<Option<CacheKey>> {
        let Some(bucket) = self.buckets.get(hash) else {
            return Ok(None);
        };
        if bucket.len() == 1 {
            let key = bucket[0];
            let stored = self.signatures[&key].bind(py);
            return Ok(stored.eq(sig)?.then_some(key));
        }
        for &key in bucket {
            let stored = self.signatures[&key].bind(py);
            if stored.eq(sig)? {
                return Ok(Some(key));
            }
        }
        Ok(None)
    }

    pub fn create_cache_key(
        &mut self,
        py: Python<'_>,
        hash: isize,
        sig: Py<PyTuple>,
    ) -> PyResult<CacheKey> {
        if let Some(key) = self.get_cache_key(py, &hash, sig.bind(py))? {
            return Ok(key);
        }
        let key = self.next_id;
        self.next_id += 1;
        self.buckets.entry(hash.clone()).or_default().push(key);
        self.signatures.insert(key, sig);
        Ok(key)
    }

    pub fn get_signature<'py>(&self, py: Python<'py>, id: &CacheKey) -> Option<Bound<'py, PyTuple>> {
        self.signatures.get(id).map(|s| s.bind(py).clone())
    }
}
