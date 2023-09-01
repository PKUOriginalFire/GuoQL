use std::path::Path;

use serde::{Deserialize, Serialize};
use tokio::io;

#[derive(Serialize, Deserialize, Clone, Default)]
/// 存储。
pub struct Storage {
    counter: i32,
    /// 锅。
    pub pots: Vec<Pot>,
    /// 约锅统计。
    pub stats: Vec<EaterStats>,
}

impl Storage {
    /// 单调递增计数器。
    pub fn counter(&mut self) -> i32 {
        self.counter += 1;
        self.counter
    }

    /// 查找锅。
    pub fn pot(&self, id: Option<i32>, index: Option<i32>) -> Option<&Pot> {
        if let Some(id) = id {
            self.pots.iter().find(|p| p.id == id)
        } else if let Some(index) = index {
            self.pots.get(index as usize)
        } else {
            None
        }
    }

    /// 查找锅，可变引用。
    pub fn pot_mut(&mut self, id: Option<i32>, index: Option<i32>) -> Option<&mut Pot> {
        if let Some(id) = id {
            self.pots.iter_mut().find(|p| p.id == id)
        } else if let Some(index) = index {
            self.pots.get_mut(index as usize)
        } else {
            None
        }
    }

    /// 查找统计数据。
    pub fn stats_mut(&mut self, name: &str) -> &mut EaterStats {
        let pos = if let Some(pos) = self.stats.iter_mut().position(|s| s.name == name) {
            pos
        } else {
            let stats = EaterStats {
                name: name.to_owned(),
                ..Default::default()
            };
            self.stats.push(stats);
            self.stats.len() - 1
        };
        &mut self.stats[pos]
    }
}

#[derive(Serialize, Deserialize, Clone)]
/// 锅。
pub struct Pot {
    /// ID。
    pub id: i32,
    /// 锅哪啊？
    pub position: String,
    /// 啥时候锅啊？
    pub time: String,
    /// 啥口味啊？
    pub taste: String,
    /// 吃锅的人。
    pub eaters: Vec<Eater>,
    /// 备注。
    pub note: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
/// 吃锅的人。
pub struct Eater {
    /// 你谁啊？
    pub name: String,
    /// 几面？
    pub mian: i32,
    /// 几饭？
    pub fan: i32,
}

impl Storage {
    /// 从文件加载存储。
    pub async fn load(path: impl AsRef<Path>) -> io::Result<Self> {
        let path = path.as_ref();
        if tokio::fs::try_exists(path).await.unwrap_or(false) {
            let data = tokio::fs::read(path).await?;
            bson::from_slice(&data).map_err(|e| io::Error::new(io::ErrorKind::Other, e))
        } else {
            Ok(Self::default())
        }
    }

    /// 保存存储到文件。
    pub async fn save(&self, path: impl AsRef<Path>) -> io::Result<()> {
        let data = bson::to_vec(self).unwrap();
        tokio::fs::write(path, data).await
    }
}

#[derive(Serialize, Deserialize, Clone, Default)]
/// 约锅统计。
pub struct EaterStats {
    /// 你谁啊？
    pub name: String,
    /// 总面数。
    pub mian: i32,
    /// 总饭数。
    pub fan: i32,
    /// 吃锅次数。
    pub eat_count: i32,
    /// 约锅次数。
    pub pot_count: i32,
}
