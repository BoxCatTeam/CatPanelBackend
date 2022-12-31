use std::sync::Arc;

use async_trait::async_trait;
use axum_sessions::async_session::{Session, SessionStore};
use bincode::config::Configuration;
use bincode::serde::{decode_from_slice, encode_into_slice};
use rocksdb::{
    DBCompressionType, DBWithThreadMode, IteratorMode, Options, SingleThreaded, WriteBatch, DB,
};
use smallvec::SmallVec;

const BINCODE_CONFIG: Configuration = bincode::config::standard()
    .with_little_endian()
    .with_variable_int_encoding()
    .write_fixed_array_length()
    .with_no_limit();

#[derive(Debug, Clone)]
pub struct RocksdbStore(Arc<DBWithThreadMode<SingleThreaded>>);

impl RocksdbStore {
    pub fn new() -> anyhow::Result<Self> {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.create_missing_column_families(true);
        opts.set_compression_type(DBCompressionType::Zstd);
        opts.set_bottommost_compression_type(DBCompressionType::Zstd);
        opts.set_level_compaction_dynamic_level_bytes(true);

        let db = DB::open(&opts, "sessions")?;

        Ok(RocksdbStore(Arc::new(db)))
    }
}

#[async_trait]
impl SessionStore for RocksdbStore {
    async fn load_session(&self, cookie_value: String) -> anyhow::Result<Option<Session>> {
        let id = Session::id_from_cookie_value(&cookie_value)?;
        Ok(Some(
            decode_from_slice(
                &if let Some(val) = self.0.get(id.as_bytes())? {
                    val
                } else {
                    return Ok(None);
                },
                BINCODE_CONFIG,
            )?
            .0,
        ))
    }

    async fn store_session(&self, session: Session) -> anyhow::Result<Option<String>> {
        let mut data = SmallVec::<[u8; 256]>::new();
        encode_into_slice(&session, &mut data, BINCODE_CONFIG)?;
        self.0.put(session.id().as_bytes(), &data)?;

        session.reset_data_changed();
        Ok(session.into_cookie_value())
    }

    async fn destroy_session(&self, session: Session) -> anyhow::Result<()> {
        self.0.delete(session.id().as_bytes()).map_err(Into::into)
    }

    async fn clear_store(&self) -> anyhow::Result<()> {
        let mut batch = WriteBatch::default();
        self.0
            .iterator(IteratorMode::Start)
            .try_for_each::<_, anyhow::Result<()>>(|res| {
                let (key, _) = res?;
                batch.delete(key);
                Ok(())
            })?;
        self.0.write(batch)?;
        Ok(())
    }
}
