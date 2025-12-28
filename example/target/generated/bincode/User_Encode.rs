impl :: bincode :: Encode for User
{
    fn encode < __E : :: bincode :: enc :: Encoder >
    (& self, encoder : & mut __E) ->core :: result :: Result < (), :: bincode
    :: error :: EncodeError >
    {
        :: bincode :: Encode :: encode(&self.id, encoder) ?; :: bincode ::
        Encode :: encode(&self.name, encoder) ?; :: bincode :: Encode ::
        encode(&self.age, encoder) ?; :: bincode :: Encode ::
        encode(&self.partner, encoder) ?; :: bincode :: Encode ::
        encode(&self.category, encoder) ?; :: bincode :: Encode ::
        encode(&self.subscriptions, encoder) ?; core :: result :: Result ::
        Ok(())
    }
}