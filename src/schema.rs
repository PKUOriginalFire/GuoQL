use juniper::{
    graphql_object, graphql_value, EmptySubscription, FieldError, FieldResult, Nullable, RootNode,
};

use crate::context::Context;
use crate::storage::{Eater, EaterStats, Pot};

/// GraphQL Schema。
/// 
/// ```graphql
/// schema {
///   query: Query
///   mutation: Mutations
/// }
/// ```
pub type Schema = RootNode<'static, Query, Mutations, EmptySubscription<Context>>;

/// 创建 GraphQL Schema。
pub fn schema() -> Schema {
    Schema::new(Query, Mutations, EmptySubscription::new())
}

/// GraphQL 查询。
/// 
/// ```graphql
/// type Query {
///   # 有锅吗？
///   pots: [Pot!]!
/// 
///   # 查询一个锅。
///   pot(id: Int, index: Int): Pot!
/// 
///   # 查询统计。
///   stats(top: Int): [EaterStats!]!
/// }
/// 
/// type Pot {
///   # 锅的 ID。
///   id: Int!
/// 
///   # 锅哪啊？
///   position: String!
/// 
///   # 啥时候锅啊？
///   time: String!
/// 
///   # 啥口味啊？
///   taste: String!
/// 
///   # 谁锅啊？
///   eaters: [Eater!]!
/// 
///   # 备注。
///   note: String
/// 
///   # 一共几面？
///   mian: Int!
/// 
///   # 一共几饭？
///   fan: Int!
/// }
/// 
/// type Eater {
///   # 你谁啊？
///   name: String!
/// 
///   # 几面？
///   mian: Int!
/// 
///   # 几饭？
///   fan: Int!
/// }
/// 
/// type EaterStats {
///   # 你谁啊？
///   name: String!
/// 
///   # 总面数。
///   mian: Int!
/// 
///   # 总饭数。
///   fan: Int!
/// 
///   # 吃锅次数。
///   eatCount: Int!
/// 
///   # 约锅次数。
///   potCount: Int!
/// 
///   # 平均面数。
///   avgMian: Float!
/// 
///   # 平均饭数。
///   avgFan: Float!
/// }
/// ```
pub struct Query;

#[graphql_object(context = Context)]
impl Query {
    /// 有锅吗？
    pub async fn pots(&self, ctx: &Context) -> FieldResult<Vec<Pot>> {
        log::info!("/有锅吗");
        Ok(ctx.read().await.pots.clone())
    }

    /// 查询一个锅。
    pub async fn pot(
        &self,
        ctx: &Context,
        #[graphql(description = "锅的 ID")] id: Option<i32>,
        #[graphql(description = "锅的索引")] index: Option<i32>,
    ) -> FieldResult<Pot> {
        log::info!("/查询锅 {id:?} {index:?}");
        ctx.read()
            .await
            .pot(id, index)
            .cloned()
            .ok_or_else(|| err("真有这锅吗？"))
    }

    /// 查询统计。
    pub async fn stats(
        &self,
        ctx: &Context,
        #[graphql(description = "截取前几名")] top: Option<i32>,
    ) -> FieldResult<Vec<EaterStats>> {
        log::info!("/统计 {top:?}");
        let mut stats = ctx.read().await.stats.clone();

        stats.sort_by_key(|stat| stat.eat_count);
        if let Some(top) = top {
            stats.truncate(top as usize);
        }
        stats.reverse();
        Ok(stats)
    }
}

/// GraphQL 变更。
/// 
/// ```graphql
/// type Mutations {
///   # 约锅。
///   newPot(
///     position: String!
///     time: String!
///     taste: String!
///     name: String!
///     mian: Int!
///     fan: Int!
///     note: String
///   ): Pot!
/// 
///   # 吃锅。
///   eat(id: Int, index: Int, name: String!, mian: Int!, fan: Int!): Pot!
/// 
///   # 吃完了。
///   finish(id: Int, index: Int): Pot!
/// 
///   # 改锅。
///   edit(
///     id: Int
///     index: Int
///     position: String
///     time: String
///     taste: String
///     note: String
///   ): Pot!
/// 
///   # 下车。
///   leave(id: Int, index: Int, name: String!): Pot!
/// 
///   # 改需求。
///   editDemand(id: Int, index: Int, name: String!, mian: Int, fan: Int): Pot!
/// 
///   # 清空锅。
///   clear: [Pot!]!
/// }
/// ```
pub struct Mutations;

#[graphql_object(context = Context)]
impl Mutations {
    /// 约锅。
    pub async fn new_pot(
        ctx: &Context,
        #[graphql(description = "锅哪啊？")] position: String,
        #[graphql(description = "啥时候锅啊？")] time: String,
        #[graphql(description = "啥口味啊？")] taste: String,
        #[graphql(description = "你谁啊")] name: String,
        #[graphql(description = "几面？")] mian: i32,
        #[graphql(description = "几饭？")] fan: i32,
        #[graphql(description = "备注")] note: Option<String>,
    ) -> FieldResult<Pot> {
        log::info!("/约锅 {name} {position} {time} {taste} {mian} {fan} {note:?}");
        let pot = ctx
            .modify(|storage| {
                let id = storage.counter();
                let pot = Pot {
                    id,
                    position,
                    time,
                    taste,
                    eaters: vec![Eater {
                        name: name.clone(),
                        mian,
                        fan,
                    }],
                    note,
                };
                storage.pots.push(pot.clone());

                storage.stats_mut(&name).pot_count += 1;
                Ok(pot)
            })
            .await
            .unwrap();
        ctx.flush().await;
        Ok(pot)
    }

    /// 吃锅。
    pub async fn eat(
        ctx: &Context,
        #[graphql(description = "锅的 ID")] id: Option<i32>,
        #[graphql(description = "锅的索引")] index: Option<i32>,
        #[graphql(description = "你谁啊")] name: String,
        #[graphql(description = "几面？")] mian: i32,
        #[graphql(description = "几饭？")] fan: i32,
    ) -> FieldResult<Pot> {
        log::info!("/吃锅 {name} {id:?} {index:?} {mian} {fan}");
        let pot = ctx
            .modify(|storage| {
                let pot = storage
                    .pot_mut(id, index)
                    .ok_or_else(|| err("真有这锅吗？"))?;
                if pot.eaters.iter().any(|e| e.name == name) {
                    return Err(err("你已经在锅里了！"));
                }

                pot.eaters.push(Eater { name, mian, fan });
                Ok(pot.clone())
            })
            .await?;
        ctx.flush().await;
        Ok(pot)
    }

    /// 吃完了。
    pub async fn finish(
        ctx: &Context,
        #[graphql(description = "锅的 ID")] id: Option<i32>,
        #[graphql(description = "锅的索引")] index: Option<i32>,
    ) -> FieldResult<Pot> {
        log::info!("/吃完了 {id:?} {index:?}");
        let pot = ctx
            .modify(|storage| {
                let pot = storage
                    .pot_mut(id, index)
                    .ok_or_else(|| err("真有这锅吗？"))?
                    .clone();

                let id = pot.id;
                storage.pots.retain(|p| p.id != id);

                for eater in pot.eaters.iter() {
                    let stats = storage.stats_mut(&eater.name);
                    stats.eat_count += 1;
                    stats.mian += eater.mian;
                    stats.fan += eater.fan;
                }
                Ok(pot)
            })
            .await?;
        ctx.flush().await;
        Ok(pot)
    }

    /// 改锅。
    pub async fn edit(
        ctx: &Context,
        #[graphql(description = "锅的 ID")] id: Option<i32>,
        #[graphql(description = "锅的索引")] index: Option<i32>,
        #[graphql(description = "锅哪啊？")] position: Option<String>,
        #[graphql(description = "啥时候锅啊？")] time: Option<String>,
        #[graphql(description = "啥口味啊？")] taste: Option<String>,
        #[graphql(description = "备注")] note: Nullable<String>,
    ) -> FieldResult<Pot> {
        log::info!("/改锅 {id:?} {index:?} {position:?} {time:?} {taste:?} {note:?}");
        let pot = ctx
            .modify(|storage| {
                let pot = storage
                    .pot_mut(id, index)
                    .ok_or_else(|| err("真有这锅吗？"))?;

                if let Some(position) = position {
                    pot.position = position;
                }
                if let Some(time) = time {
                    pot.time = time;
                }
                if let Some(taste) = taste {
                    pot.taste = taste;
                }
                if let Some(note) = note.explicit() {
                    pot.note = note;
                }

                Ok(pot.clone())
            })
            .await?;
        ctx.flush().await;
        Ok(pot)
    }

    /// 下车。
    pub async fn leave(
        ctx: &Context,
        #[graphql(description = "锅的 ID")] id: Option<i32>,
        #[graphql(description = "锅的索引")] index: Option<i32>,
        #[graphql(description = "你谁啊")] name: String,
    ) -> FieldResult<Pot> {
        log::info!("/下车 {id:?} {index:?} {name}");
        let pot = ctx
            .modify(|storage| {
                let pot = storage
                    .pot_mut(id, index)
                    .ok_or_else(|| err("真有这锅吗？"))?;

                pot.eaters.retain(|e| e.name != name);
                Ok(pot.clone())
            })
            .await?;
        ctx.flush().await;
        Ok(pot)
    }

    /// 改需求。
    pub async fn edit_demand(
        ctx: &Context,
        #[graphql(description = "锅的 ID")] id: Option<i32>,
        #[graphql(description = "锅的索引")] index: Option<i32>,
        #[graphql(description = "你谁啊")] name: String,
        #[graphql(description = "几面？")] mian: Option<i32>,
        #[graphql(description = "几饭？")] fan: Option<i32>,
    ) -> FieldResult<Pot> {
        log::info!("/改需求 {id:?} {index:?} {name} {mian:?} {fan:?}");
        let pot = ctx
            .modify(|storage| {
                let pot = storage
                    .pot_mut(id, index)
                    .ok_or_else(|| err("真有这锅吗？"))?;
                let eater = pot
                    .eaters
                    .iter_mut()
                    .find(|e| e.name == name)
                    .ok_or(err("你不在锅里！"))?;

                if let Some(mian) = mian {
                    eater.mian = mian;
                }
                if let Some(fan) = fan {
                    eater.fan = fan;
                }

                Ok(pot.clone())
            })
            .await?;
        ctx.flush().await;
        Ok(pot)
    }

    /// 清空锅。
    pub async fn clear(ctx: &Context) -> Vec<Pot> {
        log::info!("/清空锅");
        ctx.modify(|storage| {
            for pot in std::mem::take(&mut storage.pots) {
                for eater in pot.eaters.iter() {
                    let stats = storage.stats_mut(&eater.name);
                    stats.eat_count += 1;
                    stats.mian += eater.mian;
                    stats.fan += eater.fan;
                }
            }

            Ok(())
        })
        .await
        .unwrap();
        ctx.flush().await;
        vec![]
    }
}

#[graphql_object]
impl Pot {
    /// 锅的 ID。
    pub fn id(&self) -> i32 {
        self.id
    }

    /// 锅哪啊？
    pub fn position(&self) -> &str {
        &self.position
    }

    /// 啥时候锅啊？
    pub fn time(&self) -> &str {
        &self.time
    }

    /// 啥口味啊？
    pub fn taste(&self) -> &str {
        &self.taste
    }

    /// 谁锅啊？
    pub fn eaters(&self) -> &Vec<Eater> {
        &self.eaters
    }

    /// 备注。
    pub fn note(&self) -> Option<&str> {
        self.note.as_deref()
    }

    /// 一共几面？
    pub fn mian(&self) -> i32 {
        self.eaters.iter().map(|e| e.mian).sum()
    }

    /// 一共几饭？
    pub fn fan(&self) -> i32 {
        self.eaters.iter().map(|e| e.fan).sum()
    }
}

#[graphql_object]
impl Eater {
    /// 你谁啊？
    pub fn name(&self) -> &str {
        &self.name
    }

    /// 几面？
    pub fn mian(&self) -> i32 {
        self.mian
    }

    /// 几饭？
    pub fn fan(&self) -> i32 {
        self.fan
    }
}

#[graphql_object]
impl EaterStats {
    /// 你谁啊？
    pub fn name(&self) -> &str {
        &self.name
    }

    /// 总面数。
    pub fn mian(&self) -> i32 {
        self.mian
    }

    /// 总饭数。
    pub fn fan(&self) -> i32 {
        self.fan
    }

    /// 吃锅次数。
    pub fn eat_count(&self) -> i32 {
        self.eat_count
    }

    /// 约锅次数。
    pub fn pot_count(&self) -> i32 {
        self.pot_count
    }

    /// 平均面数。
    pub fn avg_mian(&self) -> f64 {
        if self.eat_count == 0 {
            0.0
        } else {
            self.mian as f64 / self.eat_count as f64
        }
    }

    /// 平均饭数。
    pub fn avg_fan(&self) -> f64 {
        if self.eat_count == 0 {
            0.0
        } else {
            self.fan as f64 / self.eat_count as f64
        }
    }
}

fn err(msg: impl Into<String>) -> FieldError<juniper::DefaultScalarValue> {
    FieldError::new(
        msg.into(),
        graphql_value!({
            "type": "USER_ERROR"
        }),
    )
}
