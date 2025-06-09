use std::sync::Arc;

use crate::{DocumentUseCases, InMemoryDocumentRepository};

/// 依赖注入容器
/// 遵循DDD架构，管理所有层的依赖关系
pub struct Container {
    // 应用层
    pub document_use_cases: Arc<DocumentUseCases<InMemoryDocumentRepository>>,
}

impl Container {
    /// 创建并配置所有依赖
    pub fn new() -> Self {
        // 基础设施层 - 创建仓库
        let document_repository = InMemoryDocumentRepository::new();

        // 应用层 - 创建用例服务
        let document_use_cases = Arc::new(DocumentUseCases::new(document_repository));

        Self { document_use_cases }
    }

    /// 获取文档用例服务
    pub fn get_document_use_cases(&self) -> Arc<DocumentUseCases<InMemoryDocumentRepository>> {
        self.document_use_cases.clone()
    }
}

impl Default for Container {
    fn default() -> Self {
        Self::new()
    }
}
