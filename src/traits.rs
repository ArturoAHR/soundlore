pub trait Identifiable {
    type Identifier;

    fn id(&self) -> &Self::Identifier;
}
