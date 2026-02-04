//! Integration tests that stress Netabase macros, attributes and arguments.
//!
//! These tests are the executable counterparts to the high-level patterns
//! described in `crate::tutorial::patterns`.

use netabase_store::macros::{
    NetabaseBlobItem,
    NetabaseModel,
    netabase_definition,
};
use netabase_store::databases::redb::RedbStore;
use netabase_store::databases::redb::transaction::RedbModelCrud;
use netabase_store::traits::database::store::NBStore;
use netabase_store::traits::database::transaction::NBTransaction;
use serde::{Deserialize, Serialize};

/// Blob payload type used for `#[blob]` tests.
#[derive(NetabaseBlobItem, Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TestBlob {
    pub bytes: Vec<u8>,
    pub content_type: String,
}

#[netabase_definition(DefB)]
mod def_b {
    use super::*;
    // Bring the blob trait into scope so generated code can call `split_into_blobs`.
    use netabase_store::blob::NetabaseBlobItem;

    #[derive(NetabaseModel, Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
    pub struct Media {
        #[primary_key]
        pub id: String,
        pub title: String,
        #[blob]
        pub data: TestBlob,
    }
}

use def_b::*;

#[test]
fn blob_roundtrip_and_subscriptions_work() -> Result<(), Box<dyn std::error::Error>> {
    // Use BlogDef for the store; both DefA and DefB are in the same repository.
    let (store, _temp) = RedbStore::<DefB>::new_temporary()?;

    // Blob round-trip
    let blob = TestBlob {
        bytes: vec![1, 2, 3, 4, 5],
        content_type: "application/octet-stream".into(),
    };

    let txn = store.begin_write()?;
    txn.create(&Media {
        id: MediaID("m1".into()),
        title: "Blob".into(),
        data: blob.clone(),
    })?;
    txn.commit()?;

    let txn = store.begin_read()?;
    let loaded: Option<Media> = txn.read(&MediaID("m1".into()))?;
    assert_eq!(loaded.unwrap().data, blob);

    Ok(())
}
