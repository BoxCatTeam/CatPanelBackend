use std::hash::Hash;
use std::path::Path;

use persy::{Config, Persy};

use crate::kv::KVStore;

#[derive(Clone)]
pub struct PersyStore(Persy);

impl KVStore for PersyStore {
    fn new<P>(path: P) -> anyhow::Result<Self>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref().with_extension("persy");
        if !path.exists() {
            Persy::create(&path)?;
        }
        let persy = Persy::open(&path, Config::new())?;
        Ok(PersyStore(persy))
    }

    fn insert<V>(&self, k: &str, v: V) -> anyhow::Result<()>
    where
        V: AsRef<[u8]>,
    {
        let mut tx = self.0.begin()?;

        /*if !tx.exists_index(DEFAULT_INDEX) {
            tx.create_index(DEFAULT_INDEX, ValueMode::Replace)?;
        }
        tx.put::<ByteVec, ByteVec>(
            DEFAULT_INDEX,
            Vec::from(k.into().to_bytes()?).into(),
            Vec::from(v.as_ref()).into(),
        )?;*/
        if tx.exists_segment(k)? {
            tx.drop_segment(k)?;
        }
        let seg_id = tx.create_segment(k)?;

        tx.insert(seg_id, v.as_ref())?;

        tx.prepare()?.commit()?;
        Ok(())
    }

    fn get(&self, k: &str) -> anyhow::Result<Option<Vec<u8>>> {
        if !self.exists(k)? {
            return Ok(None);
        }
        self.0
            .scan(k)
            .map(|mut i| i.next().map(|(_, data)| data))
            .map_err(Into::into)
    }

    fn exists(&self, k: &str) -> anyhow::Result<bool> {
        self.0.exists_segment(k).map_err(Into::into)
    }

    fn remove(&self, k: &str) -> anyhow::Result<()> {
        let mut tx = self.0.begin()?;
        tx.drop_segment(k)?;
        tx.prepare()?.commit()?;
        Ok(())
    }

    fn list(&self) -> anyhow::Result<Vec<(String, Vec<u8>)>> {
        /*self.0
        .list_segments()
        .map(|v| {
            v.into_iter()
                .map(|(k, id)| Ok((self.get(&k)?.unwrap(), k)))
                .map(|res| res.map(|(v, k)| (k, v)))
                .collect::<anyhow::Result<_>>()
        })?
        .map_err(Into::into)*/
        // 优雅.jpg
        self.0
            .list_segments()
            .map(IntoIterator::into_iter)
            .map(|i| i.map(|(k, _)| Ok((self.get(&k)?.unwrap(), k))))
            .map(|i| i.map(|res| res.map(|(v, k)| (k, v))))
            .map(Iterator::collect::<anyhow::Result<_>>)?
            .map_err(Into::into)
    }

    fn keys(&self) -> anyhow::Result<Vec<String>> {
        self.0
            .list_segments()
            .map(|v| v.into_iter().map(|(k, _)| k).collect())
            .map_err(Into::into)
    }
}
