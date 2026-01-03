pub mod indexedb;
pub mod redb;

#[allow(dead_code)]
pub struct QueryConfig {
    hydrate: Option<u8>,
    load_blobs: bool,
    read_only: bool,
    list_config: ListConfig,
}

#[allow(dead_code)]
pub struct ListConfig {
    limit: Option<usize>,
    offset: Option<usize>,
}
