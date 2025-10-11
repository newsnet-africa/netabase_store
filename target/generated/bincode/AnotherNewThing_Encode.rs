impl :: bincode :: Encode for AnotherNewThing
{
    fn encode < __E : :: bincode :: enc :: Encoder >
    (& self, encoder : & mut __E) ->core :: result :: Result < (), :: bincode
    :: error :: EncodeError >
    {
        :: bincode :: Encode :: encode(&self.second_hi, encoder) ?; :: bincode
        :: Encode :: encode(&self.you_key, encoder) ?; :: bincode :: Encode ::
        encode(&self.there, encoder) ?; :: bincode :: Encode ::
        encode(&self.just_cause, encoder) ?; core :: result :: Result ::
        Ok(())
    }
}