impl :: bincode :: Encode for Post
{
    fn encode < __E : :: bincode :: enc :: Encoder >
    (& self, encoder : & mut __E) ->core :: result :: Result < (), :: bincode
    :: error :: EncodeError >
    {
        :: bincode :: Encode :: encode(&self.id, encoder) ?; :: bincode ::
        Encode :: encode(&self.title, encoder) ?; :: bincode :: Encode ::
        encode(&self.author, encoder) ?; core :: result :: Result :: Ok(())
    }
}