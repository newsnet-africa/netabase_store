impl :: bincode :: Encode for User
{
    fn encode < __E : :: bincode :: enc :: Encoder >
    (& self, encoder : & mut __E) ->core :: result :: Result < (), :: bincode
    :: error :: EncodeError >
    {
        :: bincode :: Encode :: encode(&self.id, encoder) ?; :: bincode ::
        Encode :: encode(&self.username, encoder) ?; :: bincode :: Encode ::
        encode(&self.email, encoder) ?; core :: result :: Result :: Ok(())
    }
}