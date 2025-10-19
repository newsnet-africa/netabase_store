impl < __Context > :: bincode :: Decode < __Context > for SerializableRecord
{
    fn decode < __D : :: bincode :: de :: Decoder < Context = __Context > >
    (decoder : & mut __D) ->core :: result :: Result < Self, :: bincode ::
    error :: DecodeError >
    {
        core :: result :: Result ::
        Ok(Self
        {
            key : :: bincode :: Decode :: decode(decoder) ?, value : ::
            bincode :: Decode :: decode(decoder) ?, publisher : :: bincode ::
            Decode :: decode(decoder) ?,
        })
    }
} impl < '__de, __Context > :: bincode :: BorrowDecode < '__de, __Context >
for SerializableRecord
{
    fn borrow_decode < __D : :: bincode :: de :: BorrowDecoder < '__de,
    Context = __Context > > (decoder : & mut __D) ->core :: result :: Result <
    Self, :: bincode :: error :: DecodeError >
    {
        core :: result :: Result ::
        Ok(Self
        {
            key : :: bincode :: BorrowDecode ::< '_, __Context >::
            borrow_decode(decoder) ?, value : :: bincode :: BorrowDecode ::<
            '_, __Context >:: borrow_decode(decoder) ?, publisher : :: bincode
            :: BorrowDecode ::< '_, __Context >:: borrow_decode(decoder) ?,
        })
    }
}