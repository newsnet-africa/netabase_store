pub mod blob;
pub mod model;

pub mod constants {
    pub mod model {
        pub const PRIMARY_KEY: &str = "primary_key";
        pub const SECONDARY_KEY: &str = "secondary_key";
        pub const FOREIGN_KEY: &str = "foreign_key";
        pub const BLOB: &str = "blob";
        pub const SUBSCRIBE: &str = "subscribe";
        pub const VERSION: &str = "version";

        pub enum Attribute {
            PrimaryKey,
            SecondaryKey,
            ForeignKey,
            Blob,
            Subscribe,
            Version,
            Unknown(String),
        }

        impl From<&str> for Attribute {
            fn from(s: &str) -> Self {
                match s {
                    PRIMARY_KEY => Self::PrimaryKey,
                    SECONDARY_KEY => Self::SecondaryKey,
                    FOREIGN_KEY => Self::ForeignKey,
                    BLOB => Self::Blob,
                    SUBSCRIBE => Self::Subscribe,
                    VERSION => Self::Version,
                    _ => Self::Unknown(s.to_string()),
                }
            }
        }
    }

    pub mod blob {
        pub const BLOB: &str = "blob";
        pub const BLOB_FIELD: &str = "blob_field";
        pub const CHUNK_SCOPE: &str = "chunk_scope";
        pub const CHUNK_SIZE: &str = "chunk_size";
        pub const CHUNK_DERIVES: &str = "chunk_derives";
        pub const CHUNK_SERIALIZE: &str = "chunk_serialize";
        pub const CHUNK_DESERIALIZE: &str = "chunk_deserialize";
        pub const CHUNK_OWNER_ID: &str = "chunk_owner_id";
        pub const CHUNK_CHECKSUM: &str = "chunk_checksum";
        pub const STRATEGY: &str = "strategy";

        pub enum Attribute {
            Blob,
            BlobField,
            ChunkScope,
            ChunkSize,
            ChunkDerives,
            ChunkSerialize,
            ChunkDeserialize,
            ChunkOwnerId,
            ChunkChecksum,
            Strategy,
            Unknown(String),
        }

        impl From<&str> for Attribute {
            fn from(s: &str) -> Self {
                match s {
                    BLOB => Self::Blob,
                    BLOB_FIELD => Self::BlobField,
                    CHUNK_SCOPE => Self::ChunkScope,
                    CHUNK_SIZE => Self::ChunkSize,
                    CHUNK_DERIVES => Self::ChunkDerives,
                    CHUNK_SERIALIZE => Self::ChunkSerialize,
                    CHUNK_DESERIALIZE => Self::ChunkDeserialize,
                    CHUNK_OWNER_ID => Self::ChunkOwnerId,
                    CHUNK_CHECKSUM => Self::ChunkChecksum,
                    STRATEGY => Self::Strategy,
                    _ => Self::Unknown(s.to_string()),
                }
            }
        }
    }
}

// Maintaining some compatibility for now if needed, but better to update callsites
pub const BLOB_ATTR: &str = constants::blob::BLOB;
pub const CHUNK_SIZE_ARG: &str = constants::blob::CHUNK_SIZE;
pub const BLOB_FIELD_ARG: &str = constants::blob::BLOB_FIELD;
pub const CHUNK_SCOPE_ARG: &str = constants::blob::CHUNK_SCOPE;
pub const CHUNK_DERIVES_ARG: &str = constants::blob::CHUNK_DERIVES;
pub const CHUNK_SERIALIZE_ARG: &str = constants::blob::CHUNK_SERIALIZE;
pub const CHUNK_DESERIALIZE_ARG: &str = constants::blob::CHUNK_DESERIALIZE;
pub const CHUNK_OWNER_ID_ARG: &str = constants::blob::CHUNK_OWNER_ID;
pub const CHUNK_CHECKSUM_ARG: &str = constants::blob::CHUNK_CHECKSUM;
pub const STRATEGY_ARG: &str = constants::blob::STRATEGY;
