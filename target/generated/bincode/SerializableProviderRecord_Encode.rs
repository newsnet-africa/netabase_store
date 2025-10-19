impl :: bincode :: Encode for SerializableProviderRecord
{
    fn encode < __E : :: bincode :: enc :: Encoder >
    (& self, encoder : & mut __E) ->core :: result :: Result < (), :: bincode
    :: error :: EncodeError >
    {
        :: bincode :: Encode :: encode(&self.key, encoder) ?; :: bincode ::
        Encode :: encode(&self.provider, encoder) ?; :: bincode :: Encode ::
        encode(&self.addresses, encoder) ?; core :: result :: Result :: Ok(())
    }
}