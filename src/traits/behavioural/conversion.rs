pub trait IntoDiscriminant {
    type Discriminant: Copy + Eq + std::hash::Hash + std::fmt::Debug;
    fn discriminant(&self) -> Self::Discriminant;
}
