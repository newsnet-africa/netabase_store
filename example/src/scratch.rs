use netabase_macros::netabase_definition;

#[netabase_definition(SomeGuy)]
pub mod some_guy {
    use netabase_macros::NetabaseModel;
    use redb::Value;
    use rkyv::util::AlignedVec;

    #[derive(
        NetabaseModel,
        PartialEq,
        PartialOrd,
        Hash,
        Eq,
        Ord,
        Clone,
        Debug,
        serde::Serialize,
        serde::Deserialize,
        rkyv::Serialize,
        rkyv::Deserialize,
        rkyv::Archive,
    )]
    #[rkyv(attr(derive(Debug, PartialEq, Eq, PartialOrd, Ord)))]
    pub struct TheThing {
        #[primary_key]
        pub id: u128,
    }

    #[derive(Debug, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
    #[rkyv(attr(derive(Debug)))]
    pub struct Shit {
        id: String,
    }
}
use rkyv::util::AlignedVec;
pub use some_guy::*;

#[derive(rkyv::Serialize, rkyv::Deserialize, rkyv::Archive, Debug)]
#[rkyv(attr(derive(Debug)))]
pub struct SomeOther {
    pub id: u128,
    pub thing: String,
}

impl redb::Value for SomeOther {
    type SelfType<'a> = SomeOther;

    type AsBytes<'a> = AlignedVec;

    fn fixed_width() -> Option<usize> {
        None
    }

    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
    where
        Self: 'a,
    {
        rkyv::from_bytes::<Self, rkyv::rancor::Error>(data).expect("Fix this later")
    }

    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
    where
        Self: 'b,
    {
        rkyv::to_bytes::<rkyv::rancor::Error>(value).expect("Fix this later")
    }

    fn type_name() -> redb::TypeName {
        todo!()
    }
}

#[cfg(test)]
pub mod scratch_test {
    use super::some_guy::*;

    pub fn test_rkyv() {
        use redb::Value;
        use rkyv::rancor::Error;
        use rkyv::util::AlignedVec;

        let thing = TheThing {
            id: TheThingID(123),
        };

        let thing_bytes = rkyv::to_bytes::<Error>(&thing).unwrap();

        let accessed = rkyv::access::<ArchivedTheThing, Error>(&thing_bytes).unwrap();

        let remade = rkyv::from_bytes::<TheThing, Error>(&thing_bytes).unwrap();
    }
}
