impl :: bincode :: Encode for EmailSecondaryKey
{
    fn encode < __E : :: bincode :: enc :: Encoder >
    (& self, encoder : & mut __E) ->core :: result :: Result < (), :: bincode
    :: error :: EncodeError >
    {
        :: bincode :: Encode :: encode(&self.0, encoder) ?; core :: result ::
        Result :: Ok(())
    }
}