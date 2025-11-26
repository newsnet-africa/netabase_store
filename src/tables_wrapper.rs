//! Wrapper types to avoid feature bleeding into user crates

/// Wrapper for Tables that works with or without redb
#[derive(Clone, Copy)]
pub struct TablesWrapper<T>(pub T);

#[cfg(feature = "redb")]
pub type Tables<T> = TablesWrapper<T>;

#[cfg(not(feature = "redb"))]
pub type Tables<T> = TablesWrapper<()>;

#[cfg(feature = "redb")]
pub fn make_tables<T: Clone + Copy>(tables: T) -> Tables<T> {
    TablesWrapper(tables)
}

#[cfg(not(feature = "redb"))]
pub fn make_tables<T>(_tables: T) -> Tables<T> {
    TablesWrapper(())
}
