impl < __Context > :: bincode :: Decode < __Context > for Comment
{
    fn decode < __D : :: bincode :: de :: Decoder < Context = __Context > >
    (decoder : & mut __D) ->core :: result :: Result < Self, :: bincode ::
    error :: DecodeError >
    {
        core :: result :: Result ::
        Ok(Self
        {
            id : :: bincode :: Decode :: decode(decoder) ?, post_id : ::
            bincode :: Decode :: decode(decoder) ?, author : :: bincode ::
            Decode :: decode(decoder) ?, content : :: bincode :: Decode ::
            decode(decoder) ?,
        })
    }
} impl < '__de, __Context > :: bincode :: BorrowDecode < '__de, __Context >
for Comment
{
    fn borrow_decode < __D : :: bincode :: de :: BorrowDecoder < '__de,
    Context = __Context > > (decoder : & mut __D) ->core :: result :: Result <
    Self, :: bincode :: error :: DecodeError >
    {
        core :: result :: Result ::
        Ok(Self
        {
            id : :: bincode :: BorrowDecode ::< '_, __Context >::
            borrow_decode(decoder) ?, post_id : :: bincode :: BorrowDecode ::<
            '_, __Context >:: borrow_decode(decoder) ?, author : :: bincode ::
            BorrowDecode ::< '_, __Context >:: borrow_decode(decoder) ?,
            content : :: bincode :: BorrowDecode ::< '_, __Context >::
            borrow_decode(decoder) ?,
        })
    }
}