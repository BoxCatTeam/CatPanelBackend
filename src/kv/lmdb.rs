use std::fs::create_dir_all;
use std::path::Path;

use heed::types::{Str, UnalignedSlice};
use heed::{Env, EnvOpenOptions};

use crate::kv::KVStore;

pub struct LmdbStore {
    env: Env,
    db: heed::Database<Str, UnalignedSlice<u8>>,
}

impl KVStore for LmdbStore {
    fn new<P>(path: P) -> anyhow::Result<Self>
    where
        P: AsRef<Path>,
        Self: Sized,
    {
        let path = path.as_ref().with_extension("mdb");
        create_dir_all(&path)?;
        let env = EnvOpenOptions::new().map_size(1024 * 1024 * 1024).open(&path).map_err(heed_err_to_anyhow)?;

        Ok(LmdbStore {
            db: env.create_database(None).map_err(heed_err_to_anyhow)?,
            env,
        })
    }

    fn insert<V>(&self, k: &str, v: V) -> anyhow::Result<()>
    where
        V: AsRef<[u8]>,
    {
        let mut tx = self.env.write_txn().map_err(heed_err_to_anyhow)?;
        self.db.put(&mut tx, k, v.as_ref()).map_err(heed_err_to_anyhow)?;
        tx.commit().map_err(heed_err_to_anyhow)?;
        Ok(())
    }

    fn get(&self, k: &str) -> anyhow::Result<Option<Vec<u8>>> {
        let tx = self.env.read_txn().map_err(heed_err_to_anyhow)?;
        self.db
            .get(&tx, k)
            .map(|x| x.map(ToOwned::to_owned))
            .map_err(heed_err_to_anyhow)
    }

    fn remove(&self, k: &str) -> anyhow::Result<()> {
        let mut tx = self.env.write_txn().map_err(heed_err_to_anyhow)?;
        self.db.delete(&mut tx, k).map_err(heed_err_to_anyhow)?;
        tx.commit().map_err(heed_err_to_anyhow)?;
        Ok(())
    }

    fn list(&self) -> anyhow::Result<Vec<(String, Vec<u8>)>> {
        let tx = self.env.write_txn().map_err(heed_err_to_anyhow)?;
        let iter = self.db.iter(&tx).map_err(heed_err_to_anyhow)?;
        iter.map(|x| x.map(|(k, v)| (k.to_owned(), v.to_owned())))
            .collect::<Result<_, _>>()
            .map_err(heed_err_to_anyhow)
    }

    fn keys(&self) -> anyhow::Result<Vec<String>> {
        todo!()
    }
}

#[inline]
fn heed_err_to_anyhow(err: heed::Error) -> anyhow::Error {
    anyhow::Error::msg(err.to_string())
}
