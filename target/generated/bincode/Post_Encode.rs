impl :: bincode :: Encode for Post
{
    fn encode < __E : :: bincode :: enc :: Encoder >
    (& self, encoder : & mut __E) ->core :: result :: Result < (), :: bincode
    :: error :: EncodeError >
    {
        :: bincode :: Encode :: encode(&self.id, encoder) ?; :: bincode ::
        Encode :: encode(&self.title, encoder) ?; :: bincode :: Encode ::
        encode(&self.content, encoder) ?; :: bincode :: Encode ::
        encode(&self.author_id, encoder) ?; :: bincode :: Encode ::
        encode(&self.published, encoder) ?; core :: result :: Result :: Ok(())
    }
}