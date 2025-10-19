impl :: bincode :: Encode for SerializableRecord
{
    fn encode < __E : :: bincode :: enc :: Encoder >
    (& self, encoder : & mut __E) ->core :: result :: Result < (), :: bincode
    :: error :: EncodeError >
    {
        :: bincode :: Encode :: encode(&self.key, encoder) ?; :: bincode ::
        Encode :: encode(&self.value, encoder) ?; :: bincode :: Encode ::
        encode(&self.publisher, encoder) ?; core :: result :: Result :: Ok(())
    }
}