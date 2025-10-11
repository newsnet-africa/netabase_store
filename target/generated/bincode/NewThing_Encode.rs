impl :: bincode :: Encode for NewThing
{
    fn encode < __E : :: bincode :: enc :: Encoder >
    (& self, encoder : & mut __E) ->core :: result :: Result < (), :: bincode
    :: error :: EncodeError >
    {
        :: bincode :: Encode :: encode(&self.hi, encoder) ?; :: bincode ::
        Encode :: encode(&self.you_key, encoder) ?; :: bincode :: Encode ::
        encode(&self.there, encoder) ?; core :: result :: Result :: Ok(())
    }
}