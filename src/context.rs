use std::{path::PathBuf, sync::Arc};

use juniper::FieldResult;
use tokio::sync::{RwLock, RwLockReadGuard};

use crate::storage::Storage;

/// 上下文。
#[derive(Clone)]
pub struct Context {
    storage: Arc<RwLock<Storage>>,
    location: PathBuf,
}

impl juniper::Context for Context {}

impl Context {
    /// 创建上下文。
    pub async fn new(location: impl Into<PathBuf>) -> Self {
        let location = location.into();
        let storage = Storage::load(&location).await.unwrap_or_default();
        Self {
            storage: Arc::new(RwLock::new(storage)),
            location,
        }
    }

    /// 读取存储。
    pub async fn read(&self) -> RwLockReadGuard<'_, Storage> {
        self.storage.read().await
    }

    /// 修改存储。
    pub async fn modify<T>(
        &self,
        modify: impl FnOnce(&mut Storage) -> FieldResult<T>,
    ) -> FieldResult<T> {
        let mut storage = self.storage.write().await;
        modify(&mut storage)
    }

    /// 保存存储。
    pub async fn flush(&self) {
        let storage = self.storage.read().await;
        storage.save(&self.location).await.unwrap();
    }
}
