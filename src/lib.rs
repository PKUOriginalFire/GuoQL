#![deny(missing_docs)]
//! # GuoQL
//!
//! 锅 bot，但是 GraphQL。

/// 对象定义。
/// 
/// 具体信息见 [`Query`] 和 [`Mutations`]。
/// 
/// [`Query`]: schema::Query
/// [`Mutations`]: schema::Mutations
pub mod schema;

/// 上下文定义。
pub mod context;

/// 存储定义。
pub mod storage;
