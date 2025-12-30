pub mod indexedb;
pub mod redb;

pub struct QueryConfig {
    hydrate: Option<u8>,
    load_blobs: bool,
    read_only: bool,
    list_config: ListConfig,
}

pub struct ListConfig {
    limit: Option<usize>,
    offset: Option<usize>,
}
