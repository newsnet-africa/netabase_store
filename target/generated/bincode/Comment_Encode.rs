impl :: bincode :: Encode for Comment
{
    fn encode < __E : :: bincode :: enc :: Encoder >
    (& self, encoder : & mut __E) ->core :: result :: Result < (), :: bincode
    :: error :: EncodeError >
    {
        :: bincode :: Encode :: encode(&self.id, encoder) ?; :: bincode ::
        Encode :: encode(&self.post_id, encoder) ?; :: bincode :: Encode ::
        encode(&self.author, encoder) ?; :: bincode :: Encode ::
        encode(&self.content, encoder) ?; core :: result :: Result :: Ok(())
    }
}