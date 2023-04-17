use std::path::Path;

use redb::{Database, ReadableTable, TableDefinition};

use crate::kv::KVStore;

const TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("default");

pub struct RedbStore(Database);

impl KVStore for RedbStore {
    fn new<P>(path: P) -> anyhow::Result<Self>
    where
        P: AsRef<Path>,
        Self: Sized,
    {
        let path = path.as_ref().with_extension("redb");
        let db = Database::create(path)?;

        let mut tx = db.begin_write()?;
        tx.open_table(TABLE)?;
        tx.commit()?;

        Ok(RedbStore(db))
    }

    fn insert<V>(&self, k: &str, v: V) -> anyhow::Result<()>
    where
        V: AsRef<[u8]>,
    {
        let mut tx = self.0.begin_write()?;
        {
            let mut table = tx.open_table(TABLE)?;
            table.insert(k, v.as_ref())?;
        }
        tx.commit()?;
        Ok(())
    }

    fn get(&self, k: &str) -> anyhow::Result<Option<Vec<u8>>> {
        let tx = self.0.begin_read()?;
        let table = tx.open_table(TABLE)?;
        table
            .get(k)
            .map(|x| x.map(|x| x.value().to_owned()))
            .map_err(Into::into)
    }

    fn remove(&self, k: &str) -> anyhow::Result<()> {
        let mut tx = self.0.begin_write()?;
        {
            let mut table = tx.open_table(TABLE)?;
            table.remove(k)?;
        }
        tx.commit()?;
        Ok(())
    }

    fn list(&self) -> anyhow::Result<Vec<(String, Vec<u8>)>> {
        let tx = self.0.begin_read()?;
        let table = tx.open_table(TABLE)?;
        table
            .iter()
            .map(|x| {
                x.into_iter()
                    .map(|x| x.map(|(k, v)| (k.value().to_owned(), v.value().to_owned())))
                    .collect::<Result<_, _>>()
            })
            .and_then(|x| x)
            .map_err(Into::into)
    }

    fn keys(&self) -> anyhow::Result<Vec<String>> {
        todo!()
    }
}
