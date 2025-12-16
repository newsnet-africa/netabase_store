impl < __Context > :: bincode :: Decode < __Context > for
DefinitionSubscriptions
{
    fn decode < __D : :: bincode :: de :: Decoder < Context = __Context > >
    (decoder : & mut __D) ->core :: result :: Result < Self, :: bincode ::
    error :: DecodeError >
    {
        let variant_index = < u32 as :: bincode :: Decode ::< __D :: Context
        >>:: decode(decoder) ?; match variant_index
        {
            0u32 =>core :: result :: Result :: Ok(Self ::Topic1 {}), 1u32
            =>core :: result :: Result :: Ok(Self ::Topic2 {}), 2u32 =>core ::
            result :: Result :: Ok(Self ::Topic3 {}), 3u32 =>core :: result ::
            Result :: Ok(Self ::Topic4 {}), variant =>core :: result :: Result
            ::
            Err(:: bincode :: error :: DecodeError :: UnexpectedVariant
            {
                found : variant, type_name : "DefinitionSubscriptions",
                allowed : &:: bincode :: error :: AllowedEnumVariants :: Range
                { min: 0, max: 3 }
            })
        }
    }
} impl < '__de, __Context > :: bincode :: BorrowDecode < '__de, __Context >
for DefinitionSubscriptions
{
    fn borrow_decode < __D : :: bincode :: de :: BorrowDecoder < '__de,
    Context = __Context > > (decoder : & mut __D) ->core :: result :: Result <
    Self, :: bincode :: error :: DecodeError >
    {
        let variant_index = < u32 as :: bincode :: Decode ::< __D :: Context
        >>:: decode(decoder) ?; match variant_index
        {
            0u32 =>core :: result :: Result :: Ok(Self ::Topic1 {}), 1u32
            =>core :: result :: Result :: Ok(Self ::Topic2 {}), 2u32 =>core ::
            result :: Result :: Ok(Self ::Topic3 {}), 3u32 =>core :: result ::
            Result :: Ok(Self ::Topic4 {}), variant =>core :: result :: Result
            ::
            Err(:: bincode :: error :: DecodeError :: UnexpectedVariant
            {
                found : variant, type_name : "DefinitionSubscriptions",
                allowed : &:: bincode :: error :: AllowedEnumVariants :: Range
                { min: 0, max: 3 }
            })
        }
    }
}