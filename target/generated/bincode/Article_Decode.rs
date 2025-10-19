impl < __Context > :: bincode :: Decode < __Context > for Article
{
    fn decode < __D : :: bincode :: de :: Decoder < Context = __Context > >
    (decoder : & mut __D) ->core :: result :: Result < Self, :: bincode ::
    error :: DecodeError >
    {
        core :: result :: Result ::
        Ok(Self
        {
            id : :: bincode :: Decode :: decode(decoder) ?, title : :: bincode
            :: Decode :: decode(decoder) ?, content : :: bincode :: Decode ::
            decode(decoder) ?, author_id : :: bincode :: Decode ::
            decode(decoder) ?,
        })
    }
} impl < '__de, __Context > :: bincode :: BorrowDecode < '__de, __Context >
for Article
{
    fn borrow_decode < __D : :: bincode :: de :: BorrowDecoder < '__de,
    Context = __Context > > (decoder : & mut __D) ->core :: result :: Result <
    Self, :: bincode :: error :: DecodeError >
    {
        core :: result :: Result ::
        Ok(Self
        {
            id : :: bincode :: BorrowDecode ::< '_, __Context >::
            borrow_decode(decoder) ?, title : :: bincode :: BorrowDecode ::<
            '_, __Context >:: borrow_decode(decoder) ?, content : :: bincode
            :: BorrowDecode ::< '_, __Context >:: borrow_decode(decoder) ?,
            author_id : :: bincode :: BorrowDecode ::< '_, __Context >::
            borrow_decode(decoder) ?,
        })
    }
}