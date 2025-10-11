impl < __Context > :: bincode :: Decode < __Context > for AnotherNewThing
{
    fn decode < __D : :: bincode :: de :: Decoder < Context = __Context > >
    (decoder : & mut __D) ->core :: result :: Result < Self, :: bincode ::
    error :: DecodeError >
    {
        core :: result :: Result ::
        Ok(Self
        {
            second_hi : :: bincode :: Decode :: decode(decoder) ?, you_key :
            :: bincode :: Decode :: decode(decoder) ?, there : :: bincode ::
            Decode :: decode(decoder) ?, just_cause : :: bincode :: Decode ::
            decode(decoder) ?,
        })
    }
} impl < '__de, __Context > :: bincode :: BorrowDecode < '__de, __Context >
for AnotherNewThing
{
    fn borrow_decode < __D : :: bincode :: de :: BorrowDecoder < '__de,
    Context = __Context > > (decoder : & mut __D) ->core :: result :: Result <
    Self, :: bincode :: error :: DecodeError >
    {
        core :: result :: Result ::
        Ok(Self
        {
            second_hi : :: bincode :: BorrowDecode ::< '_, __Context >::
            borrow_decode(decoder) ?, you_key : :: bincode :: BorrowDecode ::<
            '_, __Context >:: borrow_decode(decoder) ?, there : :: bincode ::
            BorrowDecode ::< '_, __Context >:: borrow_decode(decoder) ?,
            just_cause : :: bincode :: BorrowDecode ::< '_, __Context >::
            borrow_decode(decoder) ?,
        })
    }
}