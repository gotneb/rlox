use uid::Id as IdT;

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct T(());

pub type Id = IdT<T>;

pub fn new_uid() -> Id {
    Id::new()
}