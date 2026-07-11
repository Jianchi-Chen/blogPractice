//! models/ 数据访问层。里的代码通常只跟数据库打交道（定义表结构、CRUD 函数），不关心 HTTP、权限、日志等。只服务 routes

use serde::Serialize;

pub mod article;
pub mod comment;
pub mod search;
pub mod user;

#[derive(Serialize)]
pub struct ResponseMessage {
    pub message: String,
}
