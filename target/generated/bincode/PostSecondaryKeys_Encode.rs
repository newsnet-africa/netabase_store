impl :: bincode :: Encode for PostSecondaryKeys
{
    fn encode < __E : :: bincode :: enc :: Encoder >
    (& self, encoder : & mut __E) ->core :: result :: Result < (), :: bincode
    :: error :: EncodeError >
    {
        match self
        {
            Self ::AuthorId(field_0)
            =>{
                < u32 as :: bincode :: Encode >:: encode(& (0u32), encoder) ?
                ; :: bincode :: Encode :: encode(field_0, encoder) ?; core ::
                result :: Result :: Ok(())
            }, Self ::Published(field_0)
            =>{
                < u32 as :: bincode :: Encode >:: encode(& (1u32), encoder) ?
                ; :: bincode :: Encode :: encode(field_0, encoder) ?; core ::
                result :: Result :: Ok(())
            },
        }
    }
}