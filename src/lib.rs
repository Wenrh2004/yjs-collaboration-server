//! # Volo HTTP Yjs服务器
//!
//! 这个库提供了一个基于Volo HTTP和Yrs (Yjs的Rust实现) 的协作编辑服务器。
//! 使用领域驱动设计(DDD)和整洁架构(Clean Architecture)的原则组织。

// 导出所有模块
pub mod adapter;
pub mod application;
pub mod domain;
pub mod infrastructure;

// 重导出主要类型供外部使用
use std::sync::Arc;

pub use adapter::http::router::HttpRouter;
pub use application::use_cases::document_use_cases::DocumentUseCases;
pub use domain::repositories::document_repository::DocumentRepository;
pub use infrastructure::adapters::in_memory_document_repository::InMemoryDocumentRepository;
use volo_http::Router;

/// 创建默认的路由器，使用内存文档存储库
pub fn create_router() -> Router {
    // 创建仓库
    let repository = InMemoryDocumentRepository::new();

    // 创建用例服务
    let document_use_cases = Arc::new(DocumentUseCases::new(repository));

    // 创建HTTP路由器
    let http_router = HttpRouter::new(document_use_cases);

    // 构建并返回路由器
    http_router.build_router()
}
