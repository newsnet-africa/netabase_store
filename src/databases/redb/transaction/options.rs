use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CrudOptions {
    pub list: ListConfig,
    pub hydration: HydrationConfig,
    pub blob: BlobConfig,
    pub subscription: SubscriptionConfig,
}

impl CrudOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_limit(mut self, limit: usize) -> Self {
        self.list.limit = Some(limit);
        self
    }

    pub fn with_offset(mut self, offset: usize) -> Self {
        self.list.offset = Some(offset);
        self
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ListConfig {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HydrationConfig {
    pub depth: usize,
    pub fetch_relations: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BlobConfig {
    pub strip_blobs: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SubscriptionConfig {
    pub notify: bool,
}
