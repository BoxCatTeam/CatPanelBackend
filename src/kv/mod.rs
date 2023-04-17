use std::path::Path;

#[cfg(feature = "persy")]
pub use crate::kv::persy::PersyStore;
pub use crate::kv::lmdb::LmdbStore;
pub use crate::kv::redb::RedbStore;

#[cfg(feature = "persy")]
mod persy;
mod lmdb;
mod redb;

// 由于windows/macos不支持 https://en.wikipedia.org/wiki/Sparse_file
// 所以在windows上会导致创建数据库时创建一个 大小=map_size 的本地目录
// 在unix系统上使用lmdb, 在其他系统上使用redb

#[cfg(target_family = "unix")]
pub type DefaultStore = LmdbStore;
#[cfg(not(target_family = "unix"))]
pub type DefaultStore = RedbStore;

pub trait KVStore {
    /// 具体实现应该把本地储存的文件/目录更改为独特的拓展名, 防止后端更换默认储存实现后读取错误的文件
    /// 因此, 由于会更改拓展名, 所以路径名字如果包含`.`请在结尾多加一个占位防止名字被替换更改
    /// 如: xxx.yyy.0可能会被替换为xxx.yyy.{store type}
    fn new<P>(path: P) -> anyhow::Result<Self>
    where
        P: AsRef<Path>,
        Self: Sized;

    fn insert<V>(&self, k: &str, v: V) -> anyhow::Result<()>
    where
        V: AsRef<[u8]>;

    fn get(&self, k: &str) -> anyhow::Result<Option<Vec<u8>>>;

    fn exists(&self, k: &str) -> anyhow::Result<bool> {
        self.get(k).map(|x| x.is_some())
    }

    fn remove(&self, k: &str) -> anyhow::Result<()>;

    fn list(&self) -> anyhow::Result<Vec<(String, Vec<u8>)>>;

    fn keys(&self) -> anyhow::Result<Vec<String>>;

    /*fn insert<'a, K, V>(&self, k: K, v: V) -> anyhow::Result<()>
    where
        K: Into<Key<'a>>,
        V: AsRef<[u8]>;

    fn get<'a, K>(&self, k: K) -> anyhow::Result<Vec<u8>> where K: Into<Key<'a>>;*/
}

/*pub enum Key<'s> {
    Str(&'s str),
    Bytes(&'s [u8]),
}

impl<'s> Key<'s> {
    pub fn to_str(&self) -> anyhow::Result<&'s str> {
        Ok(match self {
            Key::Str(s) => s,
            Key::Bytes(b) => std::str::from_utf8(b)?,
        })
    }

    pub fn to_bytes(&self) -> anyhow::Result<&'s [u8]> {
        Ok(match self {
            Key::Str(s) => s.as_bytes(),
            Key::Bytes(b) => b,
        })
    }
}

impl<'a> From<&'a str> for Key<'a> {
    fn from(value: &'a str) -> Self {
        Key::Str(value)
    }
}

impl<'a> From<&'a [u8]> for Key<'a> {
    fn from(value: &'a [u8]) -> Self {
        Key::Bytes(value)
    }
}
*/
