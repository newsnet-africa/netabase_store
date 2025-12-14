 pub trait NBStore<D> {
     fn execute_transaction<F: Fn()>(f: F)
 }
