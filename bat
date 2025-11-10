   Compiling netabase_store v0.0.3 (/home/rusta/Projects/NewsNet/netabase_store)
warning: unused import: `ProviderRecord`
  --> src/databases/record_store/model_store.rs:48:19
   |
48 | use libp2p::kad::{ProviderRecord, Record, RecordKey as Key};
   |                   ^^^^^^^^^^^^^^
   |
   = note: `#[warn(unused_imports)]` (part of `#[warn(unused)]`) on by default

warning: unused imports: `NetabaseModelTraitKey` and `NetabaseModelTrait`
 --> src/databases/record_store/redb_impl.rs:6:13
  |
6 | use crate::{NetabaseModelTrait, NetabaseModelTraitKey};
  |             ^^^^^^^^^^^^^^^^^^  ^^^^^^^^^^^^^^^^^^^^^

warning: unexpected `cfg` condition value: `memory`
   --> src/traits/definition.rs:240:11
    |
240 |     #[cfg(feature = "memory")]
    |           ^^^^^^^^^^^^^^^^^^
    |
    = note: expected values for `feature` are: `default`, `indexed_db_futures`, `js-sys`, `libp2p`, `native`, `paxos`, `record-store`, `redb`, `sled`, `uniffi`, `wasm`, `wasm-bindgen`, `wasm-bindgen-futures`, and `web-sys`
    = help: consider adding `memory` as a feature in `Cargo.toml`
    = note: see <https://doc.rust-lang.org/nightly/rustc/check-cfg/cargo-specifics.html> for more information about checking conditional configuration
    = note: `#[warn(unexpected_cfgs)]` on by default

warning: unexpected `cfg` condition value: `memory`
   --> src/traits/definition.rs:247:11
    |
247 |     #[cfg(feature = "memory")]
    |           ^^^^^^^^^^^^^^^^^^
    |
    = note: expected values for `feature` are: `default`, `indexed_db_futures`, `js-sys`, `libp2p`, `native`, `paxos`, `record-store`, `redb`, `sled`, `uniffi`, `wasm`, `wasm-bindgen`, `wasm-bindgen-futures`, and `web-sys`
    = help: consider adding `memory` as a feature in `Cargo.toml`
    = note: see <https://doc.rust-lang.org/nightly/rustc/check-cfg/cargo-specifics.html> for more information about checking conditional configuration

warning: unexpected `cfg` condition value: `memory`
   --> src/traits/definition.rs:254:11
    |
254 |     #[cfg(feature = "memory")]
    |           ^^^^^^^^^^^^^^^^^^
    |
    = note: expected values for `feature` are: `default`, `indexed_db_futures`, `js-sys`, `libp2p`, `native`, `paxos`, `record-store`, `redb`, `sled`, `uniffi`, `wasm`, `wasm-bindgen`, `wasm-bindgen-futures`, and `web-sys`
    = help: consider adding `memory` as a feature in `Cargo.toml`
    = note: see <https://doc.rust-lang.org/nightly/rustc/check-cfg/cargo-specifics.html> for more information about checking conditional configuration

warning: unexpected `cfg` condition value: `indexeddb`
   --> src/traits/definition.rs:261:15
    |
261 |     #[cfg(all(feature = "indexeddb", target_arch = "wasm32"))]
    |               ^^^^^^^^^^^^^^^^^^^^^
    |
    = note: expected values for `feature` are: `default`, `indexed_db_futures`, `js-sys`, `libp2p`, `native`, `paxos`, `record-store`, `redb`, `sled`, `uniffi`, `wasm`, `wasm-bindgen`, `wasm-bindgen-futures`, and `web-sys`
    = help: consider adding `indexeddb` as a feature in `Cargo.toml`
    = note: see <https://doc.rust-lang.org/nightly/rustc/check-cfg/cargo-specifics.html> for more information about checking conditional configuration

warning: unexpected `cfg` condition value: `indexeddb`
   --> src/traits/definition.rs:268:15
    |
268 |     #[cfg(all(feature = "indexeddb", target_arch = "wasm32"))]
    |               ^^^^^^^^^^^^^^^^^^^^^
    |
    = note: expected values for `feature` are: `default`, `indexed_db_futures`, `js-sys`, `libp2p`, `native`, `paxos`, `record-store`, `redb`, `sled`, `uniffi`, `wasm`, `wasm-bindgen`, `wasm-bindgen-futures`, and `web-sys`
    = help: consider adding `indexeddb` as a feature in `Cargo.toml`
    = note: see <https://doc.rust-lang.org/nightly/rustc/check-cfg/cargo-specifics.html> for more information about checking conditional configuration

warning: unexpected `cfg` condition value: `indexeddb`
   --> src/traits/definition.rs:275:15
    |
275 |     #[cfg(all(feature = "indexeddb", target_arch = "wasm32"))]
    |               ^^^^^^^^^^^^^^^^^^^^^
    |
    = note: expected values for `feature` are: `default`, `indexed_db_futures`, `js-sys`, `libp2p`, `native`, `paxos`, `record-store`, `redb`, `sled`, `uniffi`, `wasm`, `wasm-bindgen`, `wasm-bindgen-futures`, and `web-sys`
    = help: consider adding `indexeddb` as a feature in `Cargo.toml`
    = note: see <https://doc.rust-lang.org/nightly/rustc/check-cfg/cargo-specifics.html> for more information about checking conditional configuration

error[E0277]: the trait bound `<M as NetabaseModelTrait<D>>::PrimaryKey: Borrow<...>` is not satisfied
   --> src/databases/redb_store.rs:234:26
    |
234 |             table.insert(primary_key, &model)?;
    |                   ------ ^^^^^^^^^^^ unsatisfied trait bound
    |                   |
    |                   required by a bound introduced by this call
    |
    = help: the trait `Borrow<<<M as model::NetabaseModelTrait<D>>::PrimaryKey as Value>::SelfType<'_>>` is not implemented for `<M as model::NetabaseModelTrait<D>>::PrimaryKey`
note: required by a bound in `Table::<'txn, K, V>::insert`
   --> /home/rusta/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/redb-3.1.0/src/table.rs:204:19
    |
202 |     pub fn insert<'k, 'v>(
    |            ------ required by a bound in this associated function
203 |         &mut self,
204 |         key: impl Borrow<K::SelfType<'k>>,
    |                   ^^^^^^^^^^^^^^^^^^^^^^^ required by this bound in `Table::<'txn, K, V>::insert`
    = note: the full name for the type has been written to '/home/rusta/Projects/NewsNet/netabase_store/target/debug/deps/netabase_store.long-type-16354963235072105157.txt'
    = note: consider using `--verbose` to print the full type name to the console
help: consider further restricting the associated type
    |
221 |     pub fn put(&self, model: M) -> Result<(), NetabaseError> where <M as model::NetabaseModelTrait<D>>::PrimaryKey: Borrow<<<M as model::NetabaseModelTrait<D>>::PrimaryKey as Value>::SelfType<'_>> {
    |                                                              +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++

error[E0277]: the trait bound `<M as NetabaseModelTrait<D>>::PrimaryKey: Borrow<...>` is not satisfied
   --> src/databases/redb_store.rs:234:26
    |
234 |             table.insert(primary_key, &model)?;
    |                   ------ ^^^^^^^^^^^ unsatisfied trait bound
    |                   |
    |                   required by a bound introduced by this call
    |
    = help: the trait `Borrow<<<M as model::NetabaseModelTrait<D>>::PrimaryKey as Value>::SelfType<'_>>` is not implemented for `<M as model::NetabaseModelTrait<D>>::PrimaryKey`
note: required by a bound in `Table::<'txn, K, V>::insert`
   --> /home/rusta/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/redb-3.1.0/src/table.rs:204:19
    |
202 |     pub fn insert<'k, 'v>(
    |            ------ required by a bound in this associated function
203 |         &mut self,
204 |         key: impl Borrow<K::SelfType<'k>>,
    |                   ^^^^^^^^^^^^^^^^^^^^^^^ required by this bound in `Table::<'txn, K, V>::insert`
    = note: the full name for the type has been written to '/home/rusta/Projects/NewsNet/netabase_store/target/debug/deps/netabase_store-7f0e44de3e409e81.long-type-6169350916465383894.txt'
    = note: consider using `--verbose` to print the full type name to the console
help: consider further restricting the associated type
    |
221 |     pub fn put(&self, model: M) -> Result<(), NetabaseError> where <M as model::NetabaseModelTrait<D>>::PrimaryKey: Borrow<<<M as model::NetabaseModelTrait<D>>::PrimaryKey as Value>::SelfType<'_>> {
    |                                                              +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++

error[E0277]: the trait bound `&M: Borrow<<M as Value>::SelfType<'_>>` is not satisfied
   --> src/databases/redb_store.rs:234:39
    |
234 |             table.insert(primary_key, &model)?;
    |                   ------              ^^^^^^ the trait `Borrow<<M as Value>::SelfType<'_>>` is not implemented for `&M`
    |                   |
    |                   required by a bound introduced by this call
    |
note: required by a bound in `Table::<'txn, K, V>::insert`
   --> /home/rusta/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/redb-3.1.0/src/table.rs:205:21
    |
202 |     pub fn insert<'k, 'v>(
    |            ------ required by a bound in this associated function
...
205 |         value: impl Borrow<V::SelfType<'v>>,
    |                     ^^^^^^^^^^^^^^^^^^^^^^^ required by this bound in `Table::<'txn, K, V>::insert`
help: consider introducing a `where` clause, but there might be an alternative better way to express this requirement
    |
221 |     pub fn put(&self, model: M) -> Result<(), NetabaseError> where &M: Borrow<<M as Value>::SelfType<'_>> {
    |                                                              ++++++++++++++++++++++++++++++++++++++++++++
help: consider extending the `where` clause, but there might be an alternative better way to express this requirement
    |
178 |     <D as strum::IntoDiscriminant>::Discriminant: std::convert::AsRef<str>, &M: Borrow<<M as Value>::SelfType<'_>>
    |                                                                             ++++++++++++++++++++++++++++++++++++++

error[E0277]: the trait bound `(<M as NetabaseModelTrait<D>>::SecondaryKeys, ...): Borrow<...>` is not satisfied
   --> src/databases/redb_store.rs:239:38
    |
239 |                     sec_table.insert((sec_key.clone(), primary_key.clone()), ())?;
    |                               ------ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ unsatisfied trait bound
    |                               |
    |                               required by a bound introduced by this call
    |
    = help: the trait `Borrow<<(<M as model::NetabaseModelTrait<D>>::SecondaryKeys, <M as model::NetabaseModelTrait<D>>::PrimaryKey) as Value>::SelfType<'_>>` is not implemented for `(<M as NetabaseModelTrait<D>>::SecondaryKeys, ...)`
note: required by a bound in `Table::<'txn, K, V>::insert`
   --> /home/rusta/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/redb-3.1.0/src/table.rs:204:19
    |
202 |     pub fn insert<'k, 'v>(
    |            ------ required by a bound in this associated function
203 |         &mut self,
204 |         key: impl Borrow<K::SelfType<'k>>,
    |                   ^^^^^^^^^^^^^^^^^^^^^^^ required by this bound in `Table::<'txn, K, V>::insert`
    = note: the full name for the type has been written to '/home/rusta/Projects/NewsNet/netabase_store/target/debug/deps/netabase_store.long-type-13293639113437267573.txt'
    = note: consider using `--verbose` to print the full type name to the console

error[E0277]: the trait bound `(<M as NetabaseModelTrait<D>>::SecondaryKeys, ...): Borrow<...>` is not satisfied
   --> src/databases/redb_store.rs:239:38
    |
239 |                     sec_table.insert((sec_key.clone(), primary_key.clone()), ())?;
    |                               ------ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ unsatisfied trait bound
    |                               |
    |                               required by a bound introduced by this call
    |
    = help: the trait `Borrow<<(<M as model::NetabaseModelTrait<D>>::SecondaryKeys, <M as model::NetabaseModelTrait<D>>::PrimaryKey) as Value>::SelfType<'_>>` is not implemented for `(<M as NetabaseModelTrait<D>>::SecondaryKeys, ...)`
note: required by a bound in `Table::<'txn, K, V>::insert`
   --> /home/rusta/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/redb-3.1.0/src/table.rs:204:19
    |
202 |     pub fn insert<'k, 'v>(
    |            ------ required by a bound in this associated function
203 |         &mut self,
204 |         key: impl Borrow<K::SelfType<'k>>,
    |                   ^^^^^^^^^^^^^^^^^^^^^^^ required by this bound in `Table::<'txn, K, V>::insert`
    = note: the full name for the type has been written to '/home/rusta/Projects/NewsNet/netabase_store/target/debug/deps/netabase_store-7f0e44de3e409e81.long-type-11283088016044373388.txt'
    = note: consider using `--verbose` to print the full type name to the console

error[E0277]: the trait bound `<M as NetabaseModelTrait<D>>::PrimaryKey: Borrow<...>` is not satisfied
   --> src/databases/redb_store.rs:271:25
    |
271 |         match table.get(key)? {
    |                     --- ^^^ unsatisfied trait bound
    |                     |
    |                     required by a bound introduced by this call
    |
    = help: the trait `Borrow<<<M as model::NetabaseModelTrait<D>>::PrimaryKey as Value>::SelfType<'_>>` is not implemented for `<M as model::NetabaseModelTrait<D>>::PrimaryKey`
note: required by a bound in `ReadOnlyTable::<K, V>::get`
   --> /home/rusta/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/redb-3.1.0/src/table.rs:500:19
    |
498 |     pub fn get<'a>(
    |            --- required by a bound in this associated function
499 |         &self,
500 |         key: impl Borrow<K::SelfType<'a>>,
    |                   ^^^^^^^^^^^^^^^^^^^^^^^ required by this bound in `ReadOnlyTable::<K, V>::get`
    = note: the full name for the type has been written to '/home/rusta/Projects/NewsNet/netabase_store/target/debug/deps/netabase_store-7f0e44de3e409e81.long-type-5480637196807288731.txt'
    = note: consider using `--verbose` to print the full type name to the console
help: consider further restricting the associated type
    |
259 |     pub fn get(&self, key: M::PrimaryKey) -> Result<Option<M>, NetabaseError> where <M as model::NetabaseModelTrait<D>>::PrimaryKey: Borrow<<<M as model::NetabaseModelTrait<D>>::PrimaryKey as Value>::SelfType<'_>> {
    |                                                                               +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++

error[E0277]: the trait bound `<M as NetabaseModelTrait<D>>::PrimaryKey: Borrow<...>` is not satisfied
   --> src/databases/redb_store.rs:271:25
    |
271 |         match table.get(key)? {
    |                     --- ^^^ unsatisfied trait bound
    |                     |
    |                     required by a bound introduced by this call
    |
    = help: the trait `Borrow<<<M as model::NetabaseModelTrait<D>>::PrimaryKey as Value>::SelfType<'_>>` is not implemented for `<M as model::NetabaseModelTrait<D>>::PrimaryKey`
note: required by a bound in `ReadOnlyTable::<K, V>::get`
   --> /home/rusta/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/redb-3.1.0/src/table.rs:500:19
    |
498 |     pub fn get<'a>(
    |            --- required by a bound in this associated function
499 |         &self,
500 |         key: impl Borrow<K::SelfType<'a>>,
    |                   ^^^^^^^^^^^^^^^^^^^^^^^ required by this bound in `ReadOnlyTable::<K, V>::get`
    = note: the full name for the type has been written to '/home/rusta/Projects/NewsNet/netabase_store/target/debug/deps/netabase_store.long-type-65878088647593802.txt'
    = note: consider using `--verbose` to print the full type name to the console
help: consider further restricting the associated type
    |
259 |     pub fn get(&self, key: M::PrimaryKey) -> Result<Option<M>, NetabaseError> where <M as model::NetabaseModelTrait<D>>::PrimaryKey: Borrow<<<M as model::NetabaseModelTrait<D>>::PrimaryKey as Value>::SelfType<'_>> {
    |                                                                               +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++

error[E0308]: mismatched types
   --> src/databases/redb_store.rs:274:25
    |
150 | impl<D, M> RedbStoreTree<D, M>
    |         - expected this type parameter
...
274 |                 Ok(Some(model))
    |                    ---- ^^^^^ expected type parameter `M`, found associated type
    |                    |
    |                    arguments to this enum variant are incorrect
    |
    = note: expected type parameter `M`
              found associated type `<M as Value>::SelfType<'_>`
help: the type constructed contains `<M as Value>::SelfType<'_>` due to the type of the argument passed
   --> src/databases/redb_store.rs:274:20
    |
274 |                 Ok(Some(model))
    |                    ^^^^^-----^
    |                         |
    |                         this argument influences the type of `Some`
note: tuple variant defined here
   --> /home/rusta/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs:607:5
    |
607 |     Some(#[stable(feature = "rust1", since = "1.0.0")] T),
    |     ^^^^
help: consider further restricting this bound
    |
153 |     M: NetabaseModelTrait<D> + Debug + bincode::Decode<()> + Value<SelfType<'_> = M>,
    |                                                                   ++++++++++++++++++

error[E0277]: the trait bound `<M as NetabaseModelTrait<D>>::PrimaryKey: Borrow<...>` is not satisfied
   --> src/databases/redb_store.rs:295:26
    |
295 |             table.remove(key.clone())?;
    |                   ------ ^^^^^^^^^^^ unsatisfied trait bound
    |                   |
    |                   required by a bound introduced by this call
    |
    = help: the trait `Borrow<<<M as model::NetabaseModelTrait<D>>::PrimaryKey as Value>::SelfType<'_>>` is not implemented for `<M as model::NetabaseModelTrait<D>>::PrimaryKey`
note: required by a bound in `Table::<'txn, K, V>::remove`
   --> /home/rusta/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/redb-3.1.0/src/table.rs:226:19
    |
224 |     pub fn remove<'a>(
    |            ------ required by a bound in this associated function
225 |         &mut self,
226 |         key: impl Borrow<K::SelfType<'a>>,
    |                   ^^^^^^^^^^^^^^^^^^^^^^^ required by this bound in `Table::<'txn, K, V>::remove`
    = note: the full name for the type has been written to '/home/rusta/Projects/NewsNet/netabase_store/target/debug/deps/netabase_store.long-type-11767591427444541979.txt'
    = note: consider using `--verbose` to print the full type name to the console
help: consider further restricting the associated type
    |
281 |     pub fn remove(&self, key: M::PrimaryKey) -> Result<Option<M>, NetabaseError> where <M as model::NetabaseModelTrait<D>>::PrimaryKey: Borrow<<<M as model::NetabaseModelTrait<D>>::PrimaryKey as Value>::SelfType<'_>> {
    |                                                                                  +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++

error[E0277]: the trait bound `<M as NetabaseModelTrait<D>>::PrimaryKey: Borrow<...>` is not satisfied
   --> src/databases/redb_store.rs:295:26
    |
295 |             table.remove(key.clone())?;
    |                   ------ ^^^^^^^^^^^ unsatisfied trait bound
    |                   |
    |                   required by a bound introduced by this call
    |
    = help: the trait `Borrow<<<M as model::NetabaseModelTrait<D>>::PrimaryKey as Value>::SelfType<'_>>` is not implemented for `<M as model::NetabaseModelTrait<D>>::PrimaryKey`
note: required by a bound in `Table::<'txn, K, V>::remove`
   --> /home/rusta/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/redb-3.1.0/src/table.rs:226:19
    |
224 |     pub fn remove<'a>(
    |            ------ required by a bound in this associated function
225 |         &mut self,
226 |         key: impl Borrow<K::SelfType<'a>>,
    |                   ^^^^^^^^^^^^^^^^^^^^^^^ required by this bound in `Table::<'txn, K, V>::remove`
    = note: the full name for the type has been written to '/home/rusta/Projects/NewsNet/netabase_store/target/debug/deps/netabase_store-7f0e44de3e409e81.long-type-7134639908116121365.txt'
    = note: consider using `--verbose` to print the full type name to the console
help: consider further restricting the associated type
    |
281 |     pub fn remove(&self, key: M::PrimaryKey) -> Result<Option<M>, NetabaseError> where <M as model::NetabaseModelTrait<D>>::PrimaryKey: Borrow<<<M as model::NetabaseModelTrait<D>>::PrimaryKey as Value>::SelfType<'_>> {
    |                                                                                  +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++

error[E0277]: the trait bound `(<M as NetabaseModelTrait<D>>::SecondaryKeys, ...): Borrow<...>` is not satisfied
   --> src/databases/redb_store.rs:304:42
    |
304 |                         sec_table.remove(composite_key)?;
    |                                   ------ ^^^^^^^^^^^^^ unsatisfied trait bound
    |                                   |
    |                                   required by a bound introduced by this call
    |
    = help: the trait `Borrow<<(<M as model::NetabaseModelTrait<D>>::SecondaryKeys, <M as model::NetabaseModelTrait<D>>::PrimaryKey) as Value>::SelfType<'_>>` is not implemented for `(<M as NetabaseModelTrait<D>>::SecondaryKeys, ...)`
note: required by a bound in `Table::<'txn, K, V>::remove`
   --> /home/rusta/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/redb-3.1.0/src/table.rs:226:19
    |
224 |     pub fn remove<'a>(
    |            ------ required by a bound in this associated function
225 |         &mut self,
226 |         key: impl Borrow<K::SelfType<'a>>,
    |                   ^^^^^^^^^^^^^^^^^^^^^^^ required by this bound in `Table::<'txn, K, V>::remove`
    = note: the full name for the type has been written to '/home/rusta/Projects/NewsNet/netabase_store/target/debug/deps/netabase_store-7f0e44de3e409e81.long-type-11283088016044373388.txt'
    = note: consider using `--verbose` to print the full type name to the console

error[E0277]: the trait bound `(<M as NetabaseModelTrait<D>>::SecondaryKeys, ...): Borrow<...>` is not satisfied
   --> src/databases/redb_store.rs:304:42
    |
304 |                         sec_table.remove(composite_key)?;
    |                                   ------ ^^^^^^^^^^^^^ unsatisfied trait bound
    |                                   |
    |                                   required by a bound introduced by this call
    |
    = help: the trait `Borrow<<(<M as model::NetabaseModelTrait<D>>::SecondaryKeys, <M as model::NetabaseModelTrait<D>>::PrimaryKey) as Value>::SelfType<'_>>` is not implemented for `(<M as NetabaseModelTrait<D>>::SecondaryKeys, ...)`
note: required by a bound in `Table::<'txn, K, V>::remove`
   --> /home/rusta/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/redb-3.1.0/src/table.rs:226:19
    |
224 |     pub fn remove<'a>(
    |            ------ required by a bound in this associated function
225 |         &mut self,
226 |         key: impl Borrow<K::SelfType<'a>>,
    |                   ^^^^^^^^^^^^^^^^^^^^^^^ required by this bound in `Table::<'txn, K, V>::remove`
    = note: the full name for the type has been written to '/home/rusta/Projects/NewsNet/netabase_store/target/debug/deps/netabase_store.long-type-13293639113437267573.txt'
    = note: consider using `--verbose` to print the full type name to the console

error[E0308]: mismatched types
   --> src/databases/redb_store.rs:339:12
    |
336 |             results.push((key, model));
    |             -------      ------------ this argument has type `(<<M as model::NetabaseModelTrait<D>>::PrimaryKey as Value>::SelfType<'_>, <M as Value>::SelfType<'_>)`...
    |             |
    |             ... which causes `results` to have type `Vec<(<<M as model::NetabaseModelTrait<D>>::PrimaryKey as Value>::SelfType<'_>, <M as Value>::SelfType<'_>)>`
...
339 |         Ok(results)
    |         -- ^^^^^^^ expected `model::NetabaseModelTrait::PrimaryKey`, found `redb::Value::SelfType`
    |         |
    |         arguments to this enum variant are incorrect
    |
    = note: expected struct `Vec<(<M as model::NetabaseModelTrait<D>>::PrimaryKey, M)>`
               found struct `Vec<(<<M as model::NetabaseModelTrait<D>>::PrimaryKey as Value>::SelfType<'_>, <M as Value>::SelfType<'_>)>`
help: the type constructed contains `Vec<(<<M as model::NetabaseModelTrait<D>>::PrimaryKey as Value>::SelfType<'_>, <M as Value>::SelfType<'_>)>` due to the type of the argument passed
   --> src/databases/redb_store.rs:339:9
    |
339 |         Ok(results)
    |         ^^^-------^
    |            |
    |            this argument influences the type of `Ok`
note: tuple variant defined here
   --> /home/rusta/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:561:5
    |
561 |     Ok(#[stable(feature = "rust1", since = "1.0.0")] T),
    |     ^^

error[E0277]: a value of type `Vec<<M as model::NetabaseModelTrait<D>>::PrimaryKey>` cannot be built from an iterator over elements of type `<<M as model::NetabaseModelTrait<D>>::PrimaryKey as Value>::SelfType<'_>`
    --> src/databases/redb_store.rs:375:26
     |
 375 |                         .collect();
     |                          ^^^^^^^ value of type `Vec<<M as model::NetabaseModelTrait<D>>::PrimaryKey>` cannot be built from `std::iter::Iterator<Item=<<M as model::NetabaseModelTrait<D>>::PrimaryKey as Value>::SelfType<'_>>`
     |
     = help: the trait `FromIterator<<<M as model::NetabaseModelTrait<D>>::PrimaryKey as Value>::SelfType<'_>>` is not implemented for `Vec<<M as model::NetabaseModelTrait<D>>::PrimaryKey>`
note: required by a bound in `std::iter::Iterator::collect`
    --> /home/rusta/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/iter/traits/iterator.rs:2015:19
     |
2015 |     fn collect<B: FromIterator<Self::Item>>(self) -> B
     |                   ^^^^^^^^^^^^^^^^^^^^^^^^ required by this bound in `Iterator::collect`
help: consider introducing a `where` clause, but there might be an alternative better way to express this requirement
     |
 362 |     pub fn clear(&self) -> Result<(), NetabaseError> where Vec<<M as model::NetabaseModelTrait<D>>::PrimaryKey>: FromIterator<<<M as model::NetabaseModelTrait<D>>::PrimaryKey as Value>::SelfType<'_>> {
     |                                                      ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
help: consider extending the `where` clause, but there might be an alternative better way to express this requirement
     |
 178 |     <D as strum::IntoDiscriminant>::Discriminant: std::convert::AsRef<str>, Vec<<M as model::NetabaseModelTrait<D>>::PrimaryKey>: FromIterator<<<M as model::NetabaseModelTrait<D>>::PrimaryKey as Value>::SelfType<'_>>
     |                                                                             ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++

error[E0277]: the trait bound `<M as NetabaseModelTrait<D>>::PrimaryKey: Borrow<...>` is not satisfied
   --> src/databases/redb_store.rs:378:38
    |
378 |                         table.remove(key)?;
    |                               ------ ^^^ unsatisfied trait bound
    |                               |
    |                               required by a bound introduced by this call
    |
    = help: the trait `Borrow<<<M as model::NetabaseModelTrait<D>>::PrimaryKey as Value>::SelfType<'_>>` is not implemented for `<M as model::NetabaseModelTrait<D>>::PrimaryKey`
note: required by a bound in `Table::<'txn, K, V>::remove`
   --> /home/rusta/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/redb-3.1.0/src/table.rs:226:19
    |
224 |     pub fn remove<'a>(
    |            ------ required by a bound in this associated function
225 |         &mut self,
226 |         key: impl Borrow<K::SelfType<'a>>,
    |                   ^^^^^^^^^^^^^^^^^^^^^^^ required by this bound in `Table::<'txn, K, V>::remove`
    = note: the full name for the type has been written to '/home/rusta/Projects/NewsNet/netabase_store/target/debug/deps/netabase_store-7f0e44de3e409e81.long-type-12290163232243200399.txt'
    = note: consider using `--verbose` to print the full type name to the console
help: consider further restricting the associated type
    |
362 |     pub fn clear(&self) -> Result<(), NetabaseError> where <M as model::NetabaseModelTrait<D>>::PrimaryKey: Borrow<<<M as model::NetabaseModelTrait<D>>::PrimaryKey as Value>::SelfType<'_>> {
    |                                                      +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++

error[E0277]: the trait bound `<M as NetabaseModelTrait<D>>::PrimaryKey: Borrow<...>` is not satisfied
   --> src/databases/redb_store.rs:378:38
    |
378 |                         table.remove(key)?;
    |                               ------ ^^^ unsatisfied trait bound
    |                               |
    |                               required by a bound introduced by this call
    |
    = help: the trait `Borrow<<<M as model::NetabaseModelTrait<D>>::PrimaryKey as Value>::SelfType<'_>>` is not implemented for `<M as model::NetabaseModelTrait<D>>::PrimaryKey`
note: required by a bound in `Table::<'txn, K, V>::remove`
   --> /home/rusta/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/redb-3.1.0/src/table.rs:226:19
    |
224 |     pub fn remove<'a>(
    |            ------ required by a bound in this associated function
225 |         &mut self,
226 |         key: impl Borrow<K::SelfType<'a>>,
    |                   ^^^^^^^^^^^^^^^^^^^^^^^ required by this bound in `Table::<'txn, K, V>::remove`
    = note: the full name for the type has been written to '/home/rusta/Projects/NewsNet/netabase_store/target/debug/deps/netabase_store.long-type-17890766609863964302.txt'
    = note: consider using `--verbose` to print the full type name to the console
help: consider further restricting the associated type
    |
362 |     pub fn clear(&self) -> Result<(), NetabaseError> where <M as model::NetabaseModelTrait<D>>::PrimaryKey: Borrow<<<M as model::NetabaseModelTrait<D>>::PrimaryKey as Value>::SelfType<'_>> {
    |                                                      +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++

error[E0277]: a value of type `Vec<(<M as NetabaseModelTrait<D>>::SecondaryKeys, ...)>` cannot be built from an iterator over elements of type `<(..., ...) as Value>::SelfType<'_>`
    --> src/databases/redb_store.rs:394:26
     |
 394 |                         .collect();
     |                          ^^^^^^^ value of type `Vec<(<M as NetabaseModelTrait<D>>::SecondaryKeys, ...)>` cannot be built from `std::iter::Iterator<Item=<(..., ...) as Value>::SelfType<'_>>`
     |
help: the trait `FromIterator<<(<M as model::NetabaseModelTrait<D>>::SecondaryKeys, <M as model::NetabaseModelTrait<D>>::PrimaryKey) as Value>::SelfType<'_>>` is not implemented for `Vec<(<M as NetabaseModelTrait<D>>::SecondaryKeys, ...)>`
      but trait `FromIterator<(<M as model::NetabaseModelTrait<D>>::SecondaryKeys, <M as model::NetabaseModelTrait<D>>::PrimaryKey)>` is implemented for it
    --> /home/rusta/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:3686:1
     |
3686 | impl<T> FromIterator<T> for Vec<T> {
     | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
     = help: for that trait implementation, expected `(<M as NetabaseModelTrait<D>>::SecondaryKeys, ...)`, found `<(..., ...) as Value>::SelfType<'_>`
note: required by a bound in `std::iter::Iterator::collect`
    --> /home/rusta/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/iter/traits/iterator.rs:2015:19
     |
2015 |     fn collect<B: FromIterator<Self::Item>>(self) -> B
     |                   ^^^^^^^^^^^^^^^^^^^^^^^^ required by this bound in `Iterator::collect`
     = note: the full name for the type has been written to '/home/rusta/Projects/NewsNet/netabase_store/target/debug/deps/netabase_store-7f0e44de3e409e81.long-type-2330659640713159451.txt'
     = note: consider using `--verbose` to print the full type name to the console

error[E0277]: a value of type `Vec<(<M as NetabaseModelTrait<D>>::SecondaryKeys, ...)>` cannot be built from an iterator over elements of type `<(..., ...) as Value>::SelfType<'_>`
    --> src/databases/redb_store.rs:394:26
     |
 394 |                         .collect();
     |                          ^^^^^^^ value of type `Vec<(<M as NetabaseModelTrait<D>>::SecondaryKeys, ...)>` cannot be built from `std::iter::Iterator<Item=<(..., ...) as Value>::SelfType<'_>>`
     |
help: the trait `FromIterator<<(<M as model::NetabaseModelTrait<D>>::SecondaryKeys, <M as model::NetabaseModelTrait<D>>::PrimaryKey) as Value>::SelfType<'_>>` is not implemented for `Vec<(<M as NetabaseModelTrait<D>>::SecondaryKeys, ...)>`
      but trait `FromIterator<(<M as model::NetabaseModelTrait<D>>::SecondaryKeys, <M as model::NetabaseModelTrait<D>>::PrimaryKey)>` is implemented for it
    --> /home/rusta/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:3686:1
     |
3686 | impl<T> FromIterator<T> for Vec<T> {
     | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
     = help: for that trait implementation, expected `(<M as NetabaseModelTrait<D>>::SecondaryKeys, ...)`, found `<(..., ...) as Value>::SelfType<'_>`
note: required by a bound in `std::iter::Iterator::collect`
    --> /home/rusta/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/iter/traits/iterator.rs:2015:19
     |
2015 |     fn collect<B: FromIterator<Self::Item>>(self) -> B
     |                   ^^^^^^^^^^^^^^^^^^^^^^^^ required by this bound in `Iterator::collect`
     = note: the full name for the type has been written to '/home/rusta/Projects/NewsNet/netabase_store/target/debug/deps/netabase_store.long-type-8585999970212619897.txt'
     = note: consider using `--verbose` to print the full type name to the console

error[E0277]: the trait bound `(<M as NetabaseModelTrait<D>>::SecondaryKeys, ...): Borrow<...>` is not satisfied
   --> src/databases/redb_store.rs:397:42
    |
397 |                         sec_table.remove(key)?;
    |                                   ------ ^^^ unsatisfied trait bound
    |                                   |
    |                                   required by a bound introduced by this call
    |
    = help: the trait `Borrow<<(<M as model::NetabaseModelTrait<D>>::SecondaryKeys, <M as model::NetabaseModelTrait<D>>::PrimaryKey) as Value>::SelfType<'_>>` is not implemented for `(<M as NetabaseModelTrait<D>>::SecondaryKeys, ...)`
note: required by a bound in `Table::<'txn, K, V>::remove`
   --> /home/rusta/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/redb-3.1.0/src/table.rs:226:19
    |
224 |     pub fn remove<'a>(
    |            ------ required by a bound in this associated function
225 |         &mut self,
226 |         key: impl Borrow<K::SelfType<'a>>,
    |                   ^^^^^^^^^^^^^^^^^^^^^^^ required by this bound in `Table::<'txn, K, V>::remove`
    = note: the full name for the type has been written to '/home/rusta/Projects/NewsNet/netabase_store/target/debug/deps/netabase_store-7f0e44de3e409e81.long-type-11283088016044373388.txt'
    = note: consider using `--verbose` to print the full type name to the console

error[E0277]: the trait bound `(<M as NetabaseModelTrait<D>>::SecondaryKeys, ...): Borrow<...>` is not satisfied
   --> src/databases/redb_store.rs:397:42
    |
397 |                         sec_table.remove(key)?;
    |                                   ------ ^^^ unsatisfied trait bound
    |                                   |
    |                                   required by a bound introduced by this call
    |
    = help: the trait `Borrow<<(<M as model::NetabaseModelTrait<D>>::SecondaryKeys, <M as model::NetabaseModelTrait<D>>::PrimaryKey) as Value>::SelfType<'_>>` is not implemented for `(<M as NetabaseModelTrait<D>>::SecondaryKeys, ...)`
note: required by a bound in `Table::<'txn, K, V>::remove`
   --> /home/rusta/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/redb-3.1.0/src/table.rs:226:19
    |
224 |     pub fn remove<'a>(
    |            ------ required by a bound in this associated function
225 |         &mut self,
226 |         key: impl Borrow<K::SelfType<'a>>,
    |                   ^^^^^^^^^^^^^^^^^^^^^^^ required by this bound in `Table::<'txn, K, V>::remove`
    = note: the full name for the type has been written to '/home/rusta/Projects/NewsNet/netabase_store/target/debug/deps/netabase_store.long-type-13293639113437267573.txt'
    = note: consider using `--verbose` to print the full type name to the console

error[E0308]: mismatched types
   --> src/databases/redb_store.rs:444:17
    |
444 |             let (sec_key, prim_key) = composite_key_guard.value();
    |                 ^^^^^^^^^^^^^^^^^^^   --------------------------- this expression has type `<(..., ...) as Value>::SelfType<'_>`
    |                 |
    |                 expected associated type, found `(_, _)`
    |
    = note: expected associated type `<(<M as model::NetabaseModelTrait<D>>::SecondaryKeys, <M as model::NetabaseModelTrait<D>>::PrimaryKey) as Value>::SelfType<'_>`
                         found tuple `(_, _)`
help: a method is available that returns `<(<M as model::NetabaseModelTrait<D>>::SecondaryKeys, <M as model::NetabaseModelTrait<D>>::PrimaryKey) as Value>::SelfType<'_>`
   --> /home/rusta/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/redb-3.1.0/src/types.rs:106:5
    |
106 | /     fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
107 | |     where
108 | |         Self: 'a;
    | |_________________^ consider calling `redb::Value::from_bytes`
    = note: the full name for the type has been written to '/home/rusta/Projects/NewsNet/netabase_store/target/debug/deps/netabase_store-7f0e44de3e409e81.long-type-7597849078593826008.txt'
    = note: consider using `--verbose` to print the full type name to the console

error[E0308]: mismatched types
   --> src/databases/redb_store.rs:444:17
    |
444 |             let (sec_key, prim_key) = composite_key_guard.value();
    |                 ^^^^^^^^^^^^^^^^^^^   --------------------------- this expression has type `<(..., ...) as Value>::SelfType<'_>`
    |                 |
    |                 expected associated type, found `(_, _)`
    |
    = note: expected associated type `<(<M as model::NetabaseModelTrait<D>>::SecondaryKeys, <M as model::NetabaseModelTrait<D>>::PrimaryKey) as Value>::SelfType<'_>`
                         found tuple `(_, _)`
help: a method is available that returns `<(<M as model::NetabaseModelTrait<D>>::SecondaryKeys, <M as model::NetabaseModelTrait<D>>::PrimaryKey) as Value>::SelfType<'_>`
   --> /home/rusta/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/redb-3.1.0/src/types.rs:106:5
    |
106 | /     fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
107 | |     where
108 | |         Self: 'a;
    | |_________________^ consider calling `redb::Value::from_bytes`
    = note: the full name for the type has been written to '/home/rusta/Projects/NewsNet/netabase_store/target/debug/deps/netabase_store.long-type-10020461572328953268.txt'
    = note: consider using `--verbose` to print the full type name to the console

error[E0308]: mismatched types
   --> src/databases/redb_store.rs:459:12
    |
150 | impl<D, M> RedbStoreTree<D, M>
    |         - expected this type parameter
...
450 |                     results.push(model_guard.value());
    |                     -------      ------------------- this argument has type `<M as Value>::SelfType<'_>`...
    |                     |
    |                     ... which causes `results` to have type `Vec<<M as Value>::SelfType<'_>>`
...
459 |         Ok(results)
    |         -- ^^^^^^^ expected `Vec<M>`, found `Vec<<M as Value>::SelfType<'_>>`
    |         |
    |         arguments to this enum variant are incorrect
    |
    = note: expected struct `Vec<M>`
               found struct `Vec<<M as Value>::SelfType<'_>>`
help: the type constructed contains `Vec<<M as Value>::SelfType<'_>>` due to the type of the argument passed
   --> src/databases/redb_store.rs:459:9
    |
459 |         Ok(results)
    |         ^^^-------^
    |            |
    |            this argument influences the type of `Ok`
note: tuple variant defined here
   --> /home/rusta/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:561:5
    |
561 |     Ok(#[stable(feature = "rust1", since = "1.0.0")] T),
    |     ^^
help: consider further restricting this bound
    |
153 |     M: NetabaseModelTrait<D> + Debug + bincode::Decode<()> + Value<SelfType<'_> = M>,
    |                                                                   ++++++++++++++++++

error[E0308]: mismatched types
   --> src/databases/redb_store.rs:682:20
    |
629 | impl<D, M> crate::traits::store_ops::StoreOpsIter<D, M> for RedbStoreTree<D, M>
    |         - expected this type parameter
...
682 |             items: models.into_iter(),
    |                    ^^^^^^^^^^^^^^^^^^ expected `IntoIter<M>`, found `IntoIter<<M as Value>::SelfType<'_>>`
    |
    = note: expected struct `std::vec::IntoIter<M>`
               found struct `std::vec::IntoIter<<M as Value>::SelfType<'_>>`
help: consider further restricting this bound
    |
639 |         + Value<SelfType<'_> = M>,
    |                ++++++++++++++++++

error[E0277]: the trait bound `<M as NetabaseModelTrait<D>>::PrimaryKey: Borrow<...>` is not satisfied
   --> src/databases/redb_store.rs:870:38
    |
870 |                         table.insert(primary_key.clone(), model)?;
    |                               ------ ^^^^^^^^^^^^^^^^^^^ unsatisfied trait bound
    |                               |
    |                               required by a bound introduced by this call
    |
    = help: the trait `Borrow<<<M as model::NetabaseModelTrait<D>>::PrimaryKey as Value>::SelfType<'_>>` is not implemented for `<M as model::NetabaseModelTrait<D>>::PrimaryKey`
note: required by a bound in `Table::<'txn, K, V>::insert`
   --> /home/rusta/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/redb-3.1.0/src/table.rs:204:19
    |
202 |     pub fn insert<'k, 'v>(
    |            ------ required by a bound in this associated function
203 |         &mut self,
204 |         key: impl Borrow<K::SelfType<'k>>,
    |                   ^^^^^^^^^^^^^^^^^^^^^^^ required by this bound in `Table::<'txn, K, V>::insert`
    = note: the full name for the type has been written to '/home/rusta/Projects/NewsNet/netabase_store/target/debug/deps/netabase_store-7f0e44de3e409e81.long-type-13363968872081149115.txt'
    = note: consider using `--verbose` to print the full type name to the console
help: consider further restricting the associated type
    |
848 |     fn commit(self) -> Result<(), NetabaseError> where <M as model::NetabaseModelTrait<D>>::PrimaryKey: Borrow<<<M as model::NetabaseModelTrait<D>>::PrimaryKey as Value>::SelfType<'_>> {
    |                                                  +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++

error[E0277]: the trait bound `<M as NetabaseModelTrait<D>>::PrimaryKey: Borrow<...>` is not satisfied
   --> src/databases/redb_store.rs:870:38
    |
870 |                         table.insert(primary_key.clone(), model)?;
    |                               ------ ^^^^^^^^^^^^^^^^^^^ unsatisfied trait bound
    |                               |
    |                               required by a bound introduced by this call
    |
    = help: the trait `Borrow<<<M as model::NetabaseModelTrait<D>>::PrimaryKey as Value>::SelfType<'_>>` is not implemented for `<M as model::NetabaseModelTrait<D>>::PrimaryKey`
note: required by a bound in `Table::<'txn, K, V>::insert`
   --> /home/rusta/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/redb-3.1.0/src/table.rs:204:19
    |
202 |     pub fn insert<'k, 'v>(
    |            ------ required by a bound in this associated function
203 |         &mut self,
204 |         key: impl Borrow<K::SelfType<'k>>,
    |                   ^^^^^^^^^^^^^^^^^^^^^^^ required by this bound in `Table::<'txn, K, V>::insert`
    = note: the full name for the type has been written to '/home/rusta/Projects/NewsNet/netabase_store/target/debug/deps/netabase_store.long-type-12263084409769325060.txt'
    = note: consider using `--verbose` to print the full type name to the console
help: consider further restricting the associated type
    |
848 |     fn commit(self) -> Result<(), NetabaseError> where <M as model::NetabaseModelTrait<D>>::PrimaryKey: Borrow<<<M as model::NetabaseModelTrait<D>>::PrimaryKey as Value>::SelfType<'_>> {
    |                                                  +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++

error[E0277]: the trait bound `M: Borrow<<M as Value>::SelfType<'_>>` is not satisfied
   --> src/databases/redb_store.rs:870:59
    |
870 |                         table.insert(primary_key.clone(), model)?;
    |                               ------                      ^^^^^ the trait `Borrow<<M as Value>::SelfType<'_>>` is not implemented for `M`
    |                               |
    |                               required by a bound introduced by this call
    |
note: required by a bound in `Table::<'txn, K, V>::insert`
   --> /home/rusta/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/redb-3.1.0/src/table.rs:205:21
    |
202 |     pub fn insert<'k, 'v>(
    |            ------ required by a bound in this associated function
...
205 |         value: impl Borrow<V::SelfType<'v>>,
    |                     ^^^^^^^^^^^^^^^^^^^^^^^ required by this bound in `Table::<'txn, K, V>::insert`
help: consider further restricting type parameter `M` with trait `Borrow`
    |
810 |         + Value + std::borrow::Borrow<<M as redb::Value>::SelfType<'_>>,
    |                 +++++++++++++++++++++++++++++++++++++++++++++++++++++++

error[E0277]: the trait bound `(<M as NetabaseModelTrait<D>>::SecondaryKeys, ...): Borrow<...>` is not satisfied
   --> src/databases/redb_store.rs:875:50
    |
875 | ...                   sec_table.insert((sec_key.clone(), primary_key.clone()), ())?;
    |                                 ------ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ unsatisfied trait bound
    |                                 |
    |                                 required by a bound introduced by this call
    |
    = help: the trait `Borrow<<(<M as model::NetabaseModelTrait<D>>::SecondaryKeys, <M as model::NetabaseModelTrait<D>>::PrimaryKey) as Value>::SelfType<'_>>` is not implemented for `(<M as NetabaseModelTrait<D>>::SecondaryKeys, ...)`
note: required by a bound in `Table::<'txn, K, V>::insert`
   --> /home/rusta/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/redb-3.1.0/src/table.rs:204:19
    |
202 |     pub fn insert<'k, 'v>(
    |            ------ required by a bound in this associated function
203 |         &mut self,
204 |         key: impl Borrow<K::SelfType<'k>>,
    |                   ^^^^^^^^^^^^^^^^^^^^^^^ required by this bound in `Table::<'txn, K, V>::insert`
    = note: the full name for the type has been written to '/home/rusta/Projects/NewsNet/netabase_store/target/debug/deps/netabase_store-7f0e44de3e409e81.long-type-11283088016044373388.txt'
    = note: consider using `--verbose` to print the full type name to the console

error[E0277]: the trait bound `(<M as NetabaseModelTrait<D>>::SecondaryKeys, ...): Borrow<...>` is not satisfied
   --> src/databases/redb_store.rs:875:50
    |
875 | ...                   sec_table.insert((sec_key.clone(), primary_key.clone()), ())?;
    |                                 ------ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ unsatisfied trait bound
    |                                 |
    |                                 required by a bound introduced by this call
    |
    = help: the trait `Borrow<<(<M as model::NetabaseModelTrait<D>>::SecondaryKeys, <M as model::NetabaseModelTrait<D>>::PrimaryKey) as Value>::SelfType<'_>>` is not implemented for `(<M as NetabaseModelTrait<D>>::SecondaryKeys, ...)`
note: required by a bound in `Table::<'txn, K, V>::insert`
   --> /home/rusta/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/redb-3.1.0/src/table.rs:204:19
    |
202 |     pub fn insert<'k, 'v>(
    |            ------ required by a bound in this associated function
203 |         &mut self,
204 |         key: impl Borrow<K::SelfType<'k>>,
    |                   ^^^^^^^^^^^^^^^^^^^^^^^ required by this bound in `Table::<'txn, K, V>::insert`
    = note: the full name for the type has been written to '/home/rusta/Projects/NewsNet/netabase_store/target/debug/deps/netabase_store.long-type-13293639113437267573.txt'
    = note: consider using `--verbose` to print the full type name to the console

error[E0277]: the trait bound `<M as NetabaseModelTrait<D>>::PrimaryKey: Borrow<...>` is not satisfied
   --> src/databases/redb_store.rs:881:83
    |
881 |                         let secondary_keys = if let Some(model_guard) = table.get(key.clone())? {
    |                                                                               --- ^^^^^^^^^^^ unsatisfied trait bound
    |                                                                               |
    |                                                                               required by a bound introduced by this call
    |
    = help: the trait `Borrow<<<M as model::NetabaseModelTrait<D>>::PrimaryKey as Value>::SelfType<'_>>` is not implemented for `<M as model::NetabaseModelTrait<D>>::PrimaryKey`
note: required by a bound in `redb::ReadableTable::get`
   --> /home/rusta/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/redb-3.1.0/src/table.rs:375:33
    |
375 |     fn get<'a>(&self, key: impl Borrow<K::SelfType<'a>>) -> Result<Option<AccessGuard<'_, V>>>;
    |                                 ^^^^^^^^^^^^^^^^^^^^^^^ required by this bound in `ReadableTable::get`
    = note: the full name for the type has been written to '/home/rusta/Projects/NewsNet/netabase_store/target/debug/deps/netabase_store-7f0e44de3e409e81.long-type-3356614347356891770.txt'
    = note: consider using `--verbose` to print the full type name to the console
help: consider further restricting the associated type
    |
848 |     fn commit(self) -> Result<(), NetabaseError> where <M as model::NetabaseModelTrait<D>>::PrimaryKey: Borrow<<<M as model::NetabaseModelTrait<D>>::PrimaryKey as Value>::SelfType<'_>> {
    |                                                  +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++

error[E0277]: the trait bound `<M as NetabaseModelTrait<D>>::PrimaryKey: Borrow<...>` is not satisfied
   --> src/databases/redb_store.rs:881:83
    |
881 |                         let secondary_keys = if let Some(model_guard) = table.get(key.clone())? {
    |                                                                               --- ^^^^^^^^^^^ unsatisfied trait bound
    |                                                                               |
    |                                                                               required by a bound introduced by this call
    |
    = help: the trait `Borrow<<<M as model::NetabaseModelTrait<D>>::PrimaryKey as Value>::SelfType<'_>>` is not implemented for `<M as model::NetabaseModelTrait<D>>::PrimaryKey`
note: required by a bound in `redb::ReadableTable::get`
   --> /home/rusta/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/redb-3.1.0/src/table.rs:375:33
    |
375 |     fn get<'a>(&self, key: impl Borrow<K::SelfType<'a>>) -> Result<Option<AccessGuard<'_, V>>>;
    |                                 ^^^^^^^^^^^^^^^^^^^^^^^ required by this bound in `ReadableTable::get`
    = note: the full name for the type has been written to '/home/rusta/Projects/NewsNet/netabase_store/target/debug/deps/netabase_store.long-type-5969535299645717873.txt'
    = note: consider using `--verbose` to print the full type name to the console
help: consider further restricting the associated type
    |
848 |     fn commit(self) -> Result<(), NetabaseError> where <M as model::NetabaseModelTrait<D>>::PrimaryKey: Borrow<<<M as model::NetabaseModelTrait<D>>::PrimaryKey as Value>::SelfType<'_>> {
    |                                                  +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++

error[E0599]: no method named `secondary_keys` found for associated type `<M as Value>::SelfType<'_>` in the current scope
   --> src/databases/redb_store.rs:883:35
    |
883 | ...                   model.secondary_keys()
    |                             ^^^^^^^^^^^^^^ method not found in `<M as Value>::SelfType<'_>`
    |
    = help: items from traits can only be used if the trait is implemented and in scope
note: `model::NetabaseModelTrait` defines an item `secondary_keys`, perhaps you need to implement it
   --> src/traits/model.rs:69:1
    |
 69 | / pub trait NetabaseModelTrait<D: NetabaseDefinitionTrait>:
 70 | |     bincode::Encode + Sized + Clone + MaybeSend + MaybeSync + 'static
    | |_____________________________________________________________________^

error[E0277]: the trait bound `<M as NetabaseModelTrait<D>>::PrimaryKey: Borrow<...>` is not satisfied
   --> src/databases/redb_store.rs:889:38
    |
889 |                         table.remove(key.clone())?;
    |                               ------ ^^^^^^^^^^^ unsatisfied trait bound
    |                               |
    |                               required by a bound introduced by this call
    |
    = help: the trait `Borrow<<<M as model::NetabaseModelTrait<D>>::PrimaryKey as Value>::SelfType<'_>>` is not implemented for `<M as model::NetabaseModelTrait<D>>::PrimaryKey`
note: required by a bound in `Table::<'txn, K, V>::remove`
   --> /home/rusta/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/redb-3.1.0/src/table.rs:226:19
    |
224 |     pub fn remove<'a>(
    |            ------ required by a bound in this associated function
225 |         &mut self,
226 |         key: impl Borrow<K::SelfType<'a>>,
    |                   ^^^^^^^^^^^^^^^^^^^^^^^ required by this bound in `Table::<'txn, K, V>::remove`
    = note: the full name for the type has been written to '/home/rusta/Projects/NewsNet/netabase_store/target/debug/deps/netabase_store-7f0e44de3e409e81.long-type-7803747713108255642.txt'
    = note: consider using `--verbose` to print the full type name to the console
help: consider further restricting the associated type
    |
848 |     fn commit(self) -> Result<(), NetabaseError> where <M as model::NetabaseModelTrait<D>>::PrimaryKey: Borrow<<<M as model::NetabaseModelTrait<D>>::PrimaryKey as Value>::SelfType<'_>> {
    |                                                  +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++

error[E0277]: the trait bound `<M as NetabaseModelTrait<D>>::PrimaryKey: Borrow<...>` is not satisfied
   --> src/databases/redb_store.rs:889:38
    |
889 |                         table.remove(key.clone())?;
    |                               ------ ^^^^^^^^^^^ unsatisfied trait bound
    |                               |
    |                               required by a bound introduced by this call
    |
    = help: the trait `Borrow<<<M as model::NetabaseModelTrait<D>>::PrimaryKey as Value>::SelfType<'_>>` is not implemented for `<M as model::NetabaseModelTrait<D>>::PrimaryKey`
note: required by a bound in `Table::<'txn, K, V>::remove`
   --> /home/rusta/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/redb-3.1.0/src/table.rs:226:19
    |
224 |     pub fn remove<'a>(
    |            ------ required by a bound in this associated function
225 |         &mut self,
226 |         key: impl Borrow<K::SelfType<'a>>,
    |                   ^^^^^^^^^^^^^^^^^^^^^^^ required by this bound in `Table::<'txn, K, V>::remove`
    = note: the full name for the type has been written to '/home/rusta/Projects/NewsNet/netabase_store/target/debug/deps/netabase_store.long-type-14662780101526846098.txt'
    = note: consider using `--verbose` to print the full type name to the console
help: consider further restricting the associated type
    |
848 |     fn commit(self) -> Result<(), NetabaseError> where <M as model::NetabaseModelTrait<D>>::PrimaryKey: Borrow<<<M as model::NetabaseModelTrait<D>>::PrimaryKey as Value>::SelfType<'_>> {
    |                                                  +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++

error[E0277]: the trait bound `(_, <M as NetabaseModelTrait<D>>::PrimaryKey): Borrow<...>` is not satisfied
   --> src/databases/redb_store.rs:895:50
    |
895 | ...                   sec_table.remove(composite_key)?;
    |                                 ------ ^^^^^^^^^^^^^ unsatisfied trait bound
    |                                 |
    |                                 required by a bound introduced by this call
    |
    = help: the trait `Borrow<<(<M as model::NetabaseModelTrait<D>>::SecondaryKeys, <M as model::NetabaseModelTrait<D>>::PrimaryKey) as Value>::SelfType<'_>>` is not implemented for `(_, <M as model::NetabaseModelTrait<D>>::PrimaryKey)`
note: required by a bound in `Table::<'txn, K, V>::remove`
   --> /home/rusta/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/redb-3.1.0/src/table.rs:226:19
    |
224 |     pub fn remove<'a>(
    |            ------ required by a bound in this associated function
225 |         &mut self,
226 |         key: impl Borrow<K::SelfType<'a>>,
    |                   ^^^^^^^^^^^^^^^^^^^^^^^ required by this bound in `Table::<'txn, K, V>::remove`
    = note: the full name for the type has been written to '/home/rusta/Projects/NewsNet/netabase_store/target/debug/deps/netabase_store-7f0e44de3e409e81.long-type-7933115945128625960.txt'
    = note: consider using `--verbose` to print the full type name to the console

error[E0277]: the trait bound `(_, <M as NetabaseModelTrait<D>>::PrimaryKey): Borrow<...>` is not satisfied
   --> src/databases/redb_store.rs:895:50
    |
895 | ...                   sec_table.remove(composite_key)?;
    |                                 ------ ^^^^^^^^^^^^^ unsatisfied trait bound
    |                                 |
    |                                 required by a bound introduced by this call
    |
    = help: the trait `Borrow<<(<M as model::NetabaseModelTrait<D>>::SecondaryKeys, <M as model::NetabaseModelTrait<D>>::PrimaryKey) as Value>::SelfType<'_>>` is not implemented for `(_, <M as model::NetabaseModelTrait<D>>::PrimaryKey)`
note: required by a bound in `Table::<'txn, K, V>::remove`
   --> /home/rusta/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/redb-3.1.0/src/table.rs:226:19
    |
224 |     pub fn remove<'a>(
    |            ------ required by a bound in this associated function
225 |         &mut self,
226 |         key: impl Borrow<K::SelfType<'a>>,
    |                   ^^^^^^^^^^^^^^^^^^^^^^^ required by this bound in `Table::<'txn, K, V>::remove`
    = note: the full name for the type has been written to '/home/rusta/Projects/NewsNet/netabase_store/target/debug/deps/netabase_store.long-type-15092224377984126117.txt'
    = note: consider using `--verbose` to print the full type name to the console

Some errors have detailed explanations: E0277, E0308, E0599.
For more information about an error, try `rustc --explain E0277`.
warning: `netabase_store` (lib) generated 8 warnings (8 duplicates)
error: could not compile `netabase_store` (lib) due to 22 previous errors; 8 warnings emitted
warning: build failed, waiting for other jobs to finish...
warning: `netabase_store` (lib test) generated 8 warnings
error: could not compile `netabase_store` (lib test) due to 22 previous errors; 8 warnings emitted
