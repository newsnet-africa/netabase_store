#![feature(prelude_import)]
extern crate std;
#[prelude_import]
use std::prelude::rust_2021::*;
use std::fmt::Debug;
use rewrite::traits::structural::blob::{NetabaseBlobItem, ChunkSize, BlobItemChunk};
use rewrite::results::{NetabaseError, BlobReconstructionError, NetabaseResult};
struct PartialFieldBlob {
    #[chunk_size(64)]
    header: String,
    #[chunk_size(256)]
    payload: Vec<u8>,
}
#[automatically_derived]
///An archived [`PartialFieldBlob`]
#[bytecheck(crate = ::rkyv::bytecheck)]
#[repr(C)]
struct ArchivedPartialFieldBlob
where
    String: ::rkyv::Archive,
    Vec<u8>: ::rkyv::Archive,
{
    ///The archived counterpart of [`PartialFieldBlob::header`]
    header: <String as ::rkyv::Archive>::Archived,
    ///The archived counterpart of [`PartialFieldBlob::payload`]
    payload: <Vec<u8> as ::rkyv::Archive>::Archived,
}
#[automatically_derived]
unsafe impl<
    __C: ::rkyv::bytecheck::rancor::Fallible + ?::core::marker::Sized,
> ::rkyv::bytecheck::CheckBytes<__C> for ArchivedPartialFieldBlob
where
    String: ::rkyv::Archive,
    Vec<u8>: ::rkyv::Archive,
    <__C as ::rkyv::bytecheck::rancor::Fallible>::Error: ::rkyv::bytecheck::rancor::Trace,
    <String as ::rkyv::Archive>::Archived: ::rkyv::bytecheck::CheckBytes<__C>,
    <Vec<u8> as ::rkyv::Archive>::Archived: ::rkyv::bytecheck::CheckBytes<__C>,
{
    unsafe fn check_bytes(
        value: *const Self,
        context: &mut __C,
    ) -> ::core::result::Result<
        (),
        <__C as ::rkyv::bytecheck::rancor::Fallible>::Error,
    > {
        <<String as ::rkyv::Archive>::Archived as ::rkyv::bytecheck::CheckBytes<
            __C,
        >>::check_bytes(&raw const (*value).header, context)
            .map_err(|e| {
                <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Trace>::trace(
                    e,
                    ::rkyv::bytecheck::StructCheckContext {
                        struct_name: "ArchivedPartialFieldBlob",
                        field_name: "header",
                    },
                )
            })?;
        <<Vec<
            u8,
        > as ::rkyv::Archive>::Archived as ::rkyv::bytecheck::CheckBytes<
            __C,
        >>::check_bytes(&raw const (*value).payload, context)
            .map_err(|e| {
                <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Trace>::trace(
                    e,
                    ::rkyv::bytecheck::StructCheckContext {
                        struct_name: "ArchivedPartialFieldBlob",
                        field_name: "payload",
                    },
                )
            })?;
        ::core::result::Result::Ok(())
    }
}
#[automatically_derived]
///The resolver for an archived [`PartialFieldBlob`]
struct PartialFieldBlobResolver
where
    String: ::rkyv::Archive,
    Vec<u8>: ::rkyv::Archive,
{
    header: <String as ::rkyv::Archive>::Resolver,
    payload: <Vec<u8> as ::rkyv::Archive>::Resolver,
}
impl ::rkyv::Archive for PartialFieldBlob
where
    String: ::rkyv::Archive,
    Vec<u8>: ::rkyv::Archive,
{
    type Archived = ArchivedPartialFieldBlob;
    type Resolver = PartialFieldBlobResolver;
    const COPY_OPTIMIZATION: ::rkyv::traits::CopyOptimization<Self> = unsafe {
        ::rkyv::traits::CopyOptimization::enable_if(
            0 + ::core::mem::size_of::<String>() + ::core::mem::size_of::<Vec<u8>>()
                == ::core::mem::size_of::<PartialFieldBlob>()
                && <String as ::rkyv::Archive>::COPY_OPTIMIZATION.is_enabled()
                && const { builtin # offset_of(PartialFieldBlob, header) }
                    == const { builtin # offset_of(ArchivedPartialFieldBlob, header) }
                && <Vec<u8> as ::rkyv::Archive>::COPY_OPTIMIZATION.is_enabled()
                && const { builtin # offset_of(PartialFieldBlob, payload) }
                    == const { builtin # offset_of(ArchivedPartialFieldBlob, payload) },
        )
    };
    #[allow(clippy::unit_arg)]
    fn resolve(&self, resolver: Self::Resolver, out: ::rkyv::Place<Self::Archived>) {
        let field_ptr = unsafe { &raw mut (*out.ptr()).header };
        let field_out = unsafe { ::rkyv::Place::from_field_unchecked(out, field_ptr) };
        <String as ::rkyv::Archive>::resolve(&self.header, resolver.header, field_out);
        let field_ptr = unsafe { &raw mut (*out.ptr()).payload };
        let field_out = unsafe { ::rkyv::Place::from_field_unchecked(out, field_ptr) };
        <Vec<
            u8,
        > as ::rkyv::Archive>::resolve(&self.payload, resolver.payload, field_out);
    }
}
unsafe impl ::rkyv::traits::Portable for ArchivedPartialFieldBlob
where
    String: ::rkyv::Archive,
    Vec<u8>: ::rkyv::Archive,
    <String as ::rkyv::Archive>::Archived: ::rkyv::traits::Portable,
    <Vec<u8> as ::rkyv::Archive>::Archived: ::rkyv::traits::Portable,
{}
#[automatically_derived]
impl<__S: ::rkyv::rancor::Fallible + ?Sized> ::rkyv::Serialize<__S> for PartialFieldBlob
where
    String: ::rkyv::Serialize<__S>,
    Vec<u8>: ::rkyv::Serialize<__S>,
{
    fn serialize(
        &self,
        serializer: &mut __S,
    ) -> ::core::result::Result<
        <Self as ::rkyv::Archive>::Resolver,
        <__S as ::rkyv::rancor::Fallible>::Error,
    > {
        let __this = self;
        ::core::result::Result::Ok(PartialFieldBlobResolver {
            header: <String as ::rkyv::Serialize<
                __S,
            >>::serialize(&__this.header, serializer)?,
            payload: <Vec<
                u8,
            > as ::rkyv::Serialize<__S>>::serialize(&__this.payload, serializer)?,
        })
    }
}
#[automatically_derived]
impl<__D: ::rkyv::rancor::Fallible + ?Sized> ::rkyv::Deserialize<PartialFieldBlob, __D>
for ::rkyv::Archived<PartialFieldBlob>
where
    String: ::rkyv::Archive,
    <String as ::rkyv::Archive>::Archived: ::rkyv::Deserialize<String, __D>,
    Vec<u8>: ::rkyv::Archive,
    <Vec<u8> as ::rkyv::Archive>::Archived: ::rkyv::Deserialize<Vec<u8>, __D>,
{
    fn deserialize(
        &self,
        deserializer: &mut __D,
    ) -> ::core::result::Result<
        PartialFieldBlob,
        <__D as ::rkyv::rancor::Fallible>::Error,
    > {
        let __this = self;
        ::core::result::Result::Ok(PartialFieldBlob {
            header: <<String as ::rkyv::Archive>::Archived as ::rkyv::Deserialize<
                String,
                __D,
            >>::deserialize(&__this.header, deserializer)?,
            payload: <<Vec<
                u8,
            > as ::rkyv::Archive>::Archived as ::rkyv::Deserialize<
                Vec<u8>,
                __D,
            >>::deserialize(&__this.payload, deserializer)?,
        })
    }
}
#[automatically_derived]
impl ::core::fmt::Debug for PartialFieldBlob {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::debug_struct_field2_finish(
            f,
            "PartialFieldBlob",
            "header",
            &self.header,
            "payload",
            &&self.payload,
        )
    }
}
#[automatically_derived]
impl ::core::clone::Clone for PartialFieldBlob {
    #[inline]
    fn clone(&self) -> PartialFieldBlob {
        PartialFieldBlob {
            header: ::core::clone::Clone::clone(&self.header),
            payload: ::core::clone::Clone::clone(&self.payload),
        }
    }
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for PartialFieldBlob {}
#[automatically_derived]
impl ::core::cmp::PartialEq for PartialFieldBlob {
    #[inline]
    fn eq(&self, other: &PartialFieldBlob) -> bool {
        self.header == other.header && self.payload == other.payload
    }
}
#[automatically_derived]
impl ::core::cmp::Eq for PartialFieldBlob {
    #[inline]
    #[doc(hidden)]
    #[coverage(off)]
    fn assert_fields_are_eq(&self) {
        let _: ::core::cmp::AssertParamIsEq<String>;
        let _: ::core::cmp::AssertParamIsEq<Vec<u8>>;
    }
}
pub struct PartialFieldBlobHeaderChunk {
    pub index: usize,
    pub data: Vec<u8>,
}
#[automatically_derived]
impl ::core::fmt::Debug for PartialFieldBlobHeaderChunk {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::debug_struct_field2_finish(
            f,
            "PartialFieldBlobHeaderChunk",
            "index",
            &self.index,
            "data",
            &&self.data,
        )
    }
}
#[automatically_derived]
impl ::core::clone::Clone for PartialFieldBlobHeaderChunk {
    #[inline]
    fn clone(&self) -> PartialFieldBlobHeaderChunk {
        PartialFieldBlobHeaderChunk {
            index: ::core::clone::Clone::clone(&self.index),
            data: ::core::clone::Clone::clone(&self.data),
        }
    }
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for PartialFieldBlobHeaderChunk {}
#[automatically_derived]
impl ::core::cmp::PartialEq for PartialFieldBlobHeaderChunk {
    #[inline]
    fn eq(&self, other: &PartialFieldBlobHeaderChunk) -> bool {
        self.index == other.index && self.data == other.data
    }
}
#[automatically_derived]
impl ::core::cmp::Eq for PartialFieldBlobHeaderChunk {
    #[inline]
    #[doc(hidden)]
    #[coverage(off)]
    fn assert_fields_are_eq(&self) {
        let _: ::core::cmp::AssertParamIsEq<usize>;
        let _: ::core::cmp::AssertParamIsEq<Vec<u8>>;
    }
}
#[automatically_derived]
impl ::core::cmp::PartialOrd for PartialFieldBlobHeaderChunk {
    #[inline]
    fn partial_cmp(
        &self,
        other: &PartialFieldBlobHeaderChunk,
    ) -> ::core::option::Option<::core::cmp::Ordering> {
        match ::core::cmp::PartialOrd::partial_cmp(&self.index, &other.index) {
            ::core::option::Option::Some(::core::cmp::Ordering::Equal) => {
                ::core::cmp::PartialOrd::partial_cmp(&self.data, &other.data)
            }
            cmp => cmp,
        }
    }
}
#[automatically_derived]
impl ::core::cmp::Ord for PartialFieldBlobHeaderChunk {
    #[inline]
    fn cmp(&self, other: &PartialFieldBlobHeaderChunk) -> ::core::cmp::Ordering {
        match ::core::cmp::Ord::cmp(&self.index, &other.index) {
            ::core::cmp::Ordering::Equal => {
                ::core::cmp::Ord::cmp(&self.data, &other.data)
            }
            cmp => cmp,
        }
    }
}
#[automatically_derived]
///An archived [`PartialFieldBlobHeaderChunk`]
#[bytecheck(crate = ::rkyv::bytecheck)]
#[repr(C)]
pub struct ArchivedPartialFieldBlobHeaderChunk
where
    usize: ::rkyv::Archive,
    Vec<u8>: ::rkyv::Archive,
{
    ///The archived counterpart of [`PartialFieldBlobHeaderChunk::index`]
    pub index: <usize as ::rkyv::Archive>::Archived,
    ///The archived counterpart of [`PartialFieldBlobHeaderChunk::data`]
    pub data: <Vec<u8> as ::rkyv::Archive>::Archived,
}
#[automatically_derived]
unsafe impl<
    __C: ::rkyv::bytecheck::rancor::Fallible + ?::core::marker::Sized,
> ::rkyv::bytecheck::CheckBytes<__C> for ArchivedPartialFieldBlobHeaderChunk
where
    usize: ::rkyv::Archive,
    Vec<u8>: ::rkyv::Archive,
    <__C as ::rkyv::bytecheck::rancor::Fallible>::Error: ::rkyv::bytecheck::rancor::Trace,
    <usize as ::rkyv::Archive>::Archived: ::rkyv::bytecheck::CheckBytes<__C>,
    <Vec<u8> as ::rkyv::Archive>::Archived: ::rkyv::bytecheck::CheckBytes<__C>,
{
    unsafe fn check_bytes(
        value: *const Self,
        context: &mut __C,
    ) -> ::core::result::Result<
        (),
        <__C as ::rkyv::bytecheck::rancor::Fallible>::Error,
    > {
        <<usize as ::rkyv::Archive>::Archived as ::rkyv::bytecheck::CheckBytes<
            __C,
        >>::check_bytes(&raw const (*value).index, context)
            .map_err(|e| {
                <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Trace>::trace(
                    e,
                    ::rkyv::bytecheck::StructCheckContext {
                        struct_name: "ArchivedPartialFieldBlobHeaderChunk",
                        field_name: "index",
                    },
                )
            })?;
        <<Vec<
            u8,
        > as ::rkyv::Archive>::Archived as ::rkyv::bytecheck::CheckBytes<
            __C,
        >>::check_bytes(&raw const (*value).data, context)
            .map_err(|e| {
                <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Trace>::trace(
                    e,
                    ::rkyv::bytecheck::StructCheckContext {
                        struct_name: "ArchivedPartialFieldBlobHeaderChunk",
                        field_name: "data",
                    },
                )
            })?;
        ::core::result::Result::Ok(())
    }
}
#[automatically_derived]
///The resolver for an archived [`PartialFieldBlobHeaderChunk`]
pub struct PartialFieldBlobHeaderChunkResolver
where
    usize: ::rkyv::Archive,
    Vec<u8>: ::rkyv::Archive,
{
    index: <usize as ::rkyv::Archive>::Resolver,
    data: <Vec<u8> as ::rkyv::Archive>::Resolver,
}
impl ::rkyv::Archive for PartialFieldBlobHeaderChunk
where
    usize: ::rkyv::Archive,
    Vec<u8>: ::rkyv::Archive,
{
    type Archived = ArchivedPartialFieldBlobHeaderChunk;
    type Resolver = PartialFieldBlobHeaderChunkResolver;
    const COPY_OPTIMIZATION: ::rkyv::traits::CopyOptimization<Self> = unsafe {
        ::rkyv::traits::CopyOptimization::enable_if(
            0 + ::core::mem::size_of::<usize>() + ::core::mem::size_of::<Vec<u8>>()
                == ::core::mem::size_of::<PartialFieldBlobHeaderChunk>()
                && <usize as ::rkyv::Archive>::COPY_OPTIMIZATION.is_enabled()
                && const { builtin # offset_of(PartialFieldBlobHeaderChunk, index) }
                    == const {
                        builtin # offset_of(ArchivedPartialFieldBlobHeaderChunk, index)
                    } && <Vec<u8> as ::rkyv::Archive>::COPY_OPTIMIZATION.is_enabled()
                && const { builtin # offset_of(PartialFieldBlobHeaderChunk, data) }
                    == const {
                        builtin # offset_of(ArchivedPartialFieldBlobHeaderChunk, data)
                    },
        )
    };
    #[allow(clippy::unit_arg)]
    fn resolve(&self, resolver: Self::Resolver, out: ::rkyv::Place<Self::Archived>) {
        let field_ptr = unsafe { &raw mut (*out.ptr()).index };
        let field_out = unsafe { ::rkyv::Place::from_field_unchecked(out, field_ptr) };
        <usize as ::rkyv::Archive>::resolve(&self.index, resolver.index, field_out);
        let field_ptr = unsafe { &raw mut (*out.ptr()).data };
        let field_out = unsafe { ::rkyv::Place::from_field_unchecked(out, field_ptr) };
        <Vec<u8> as ::rkyv::Archive>::resolve(&self.data, resolver.data, field_out);
    }
}
unsafe impl ::rkyv::traits::Portable for ArchivedPartialFieldBlobHeaderChunk
where
    usize: ::rkyv::Archive,
    Vec<u8>: ::rkyv::Archive,
    <usize as ::rkyv::Archive>::Archived: ::rkyv::traits::Portable,
    <Vec<u8> as ::rkyv::Archive>::Archived: ::rkyv::traits::Portable,
{}
#[automatically_derived]
impl<__S: ::rkyv::rancor::Fallible + ?Sized> ::rkyv::Serialize<__S>
for PartialFieldBlobHeaderChunk
where
    usize: ::rkyv::Serialize<__S>,
    Vec<u8>: ::rkyv::Serialize<__S>,
{
    fn serialize(
        &self,
        serializer: &mut __S,
    ) -> ::core::result::Result<
        <Self as ::rkyv::Archive>::Resolver,
        <__S as ::rkyv::rancor::Fallible>::Error,
    > {
        let __this = self;
        ::core::result::Result::Ok(PartialFieldBlobHeaderChunkResolver {
            index: <usize as ::rkyv::Serialize<
                __S,
            >>::serialize(&__this.index, serializer)?,
            data: <Vec<
                u8,
            > as ::rkyv::Serialize<__S>>::serialize(&__this.data, serializer)?,
        })
    }
}
#[automatically_derived]
impl<
    __D: ::rkyv::rancor::Fallible + ?Sized,
> ::rkyv::Deserialize<PartialFieldBlobHeaderChunk, __D>
for ::rkyv::Archived<PartialFieldBlobHeaderChunk>
where
    usize: ::rkyv::Archive,
    <usize as ::rkyv::Archive>::Archived: ::rkyv::Deserialize<usize, __D>,
    Vec<u8>: ::rkyv::Archive,
    <Vec<u8> as ::rkyv::Archive>::Archived: ::rkyv::Deserialize<Vec<u8>, __D>,
{
    fn deserialize(
        &self,
        deserializer: &mut __D,
    ) -> ::core::result::Result<
        PartialFieldBlobHeaderChunk,
        <__D as ::rkyv::rancor::Fallible>::Error,
    > {
        let __this = self;
        ::core::result::Result::Ok(PartialFieldBlobHeaderChunk {
            index: <<usize as ::rkyv::Archive>::Archived as ::rkyv::Deserialize<
                usize,
                __D,
            >>::deserialize(&__this.index, deserializer)?,
            data: <<Vec<
                u8,
            > as ::rkyv::Archive>::Archived as ::rkyv::Deserialize<
                Vec<u8>,
                __D,
            >>::deserialize(&__this.data, deserializer)?,
        })
    }
}
impl ::rewrite::traits::structural::blob::BlobItemChunk for PartialFieldBlobHeaderChunk {
    type Index = usize;
    fn get_index(&self) -> &Self::Index {
        &self.index
    }
}
pub struct PartialFieldBlobPayloadChunk {
    pub index: usize,
    pub data: Vec<u8>,
}
#[automatically_derived]
impl ::core::fmt::Debug for PartialFieldBlobPayloadChunk {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::debug_struct_field2_finish(
            f,
            "PartialFieldBlobPayloadChunk",
            "index",
            &self.index,
            "data",
            &&self.data,
        )
    }
}
#[automatically_derived]
impl ::core::clone::Clone for PartialFieldBlobPayloadChunk {
    #[inline]
    fn clone(&self) -> PartialFieldBlobPayloadChunk {
        PartialFieldBlobPayloadChunk {
            index: ::core::clone::Clone::clone(&self.index),
            data: ::core::clone::Clone::clone(&self.data),
        }
    }
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for PartialFieldBlobPayloadChunk {}
#[automatically_derived]
impl ::core::cmp::PartialEq for PartialFieldBlobPayloadChunk {
    #[inline]
    fn eq(&self, other: &PartialFieldBlobPayloadChunk) -> bool {
        self.index == other.index && self.data == other.data
    }
}
#[automatically_derived]
impl ::core::cmp::Eq for PartialFieldBlobPayloadChunk {
    #[inline]
    #[doc(hidden)]
    #[coverage(off)]
    fn assert_fields_are_eq(&self) {
        let _: ::core::cmp::AssertParamIsEq<usize>;
        let _: ::core::cmp::AssertParamIsEq<Vec<u8>>;
    }
}
#[automatically_derived]
impl ::core::cmp::PartialOrd for PartialFieldBlobPayloadChunk {
    #[inline]
    fn partial_cmp(
        &self,
        other: &PartialFieldBlobPayloadChunk,
    ) -> ::core::option::Option<::core::cmp::Ordering> {
        match ::core::cmp::PartialOrd::partial_cmp(&self.index, &other.index) {
            ::core::option::Option::Some(::core::cmp::Ordering::Equal) => {
                ::core::cmp::PartialOrd::partial_cmp(&self.data, &other.data)
            }
            cmp => cmp,
        }
    }
}
#[automatically_derived]
impl ::core::cmp::Ord for PartialFieldBlobPayloadChunk {
    #[inline]
    fn cmp(&self, other: &PartialFieldBlobPayloadChunk) -> ::core::cmp::Ordering {
        match ::core::cmp::Ord::cmp(&self.index, &other.index) {
            ::core::cmp::Ordering::Equal => {
                ::core::cmp::Ord::cmp(&self.data, &other.data)
            }
            cmp => cmp,
        }
    }
}
#[automatically_derived]
///An archived [`PartialFieldBlobPayloadChunk`]
#[bytecheck(crate = ::rkyv::bytecheck)]
#[repr(C)]
pub struct ArchivedPartialFieldBlobPayloadChunk
where
    usize: ::rkyv::Archive,
    Vec<u8>: ::rkyv::Archive,
{
    ///The archived counterpart of [`PartialFieldBlobPayloadChunk::index`]
    pub index: <usize as ::rkyv::Archive>::Archived,
    ///The archived counterpart of [`PartialFieldBlobPayloadChunk::data`]
    pub data: <Vec<u8> as ::rkyv::Archive>::Archived,
}
#[automatically_derived]
unsafe impl<
    __C: ::rkyv::bytecheck::rancor::Fallible + ?::core::marker::Sized,
> ::rkyv::bytecheck::CheckBytes<__C> for ArchivedPartialFieldBlobPayloadChunk
where
    usize: ::rkyv::Archive,
    Vec<u8>: ::rkyv::Archive,
    <__C as ::rkyv::bytecheck::rancor::Fallible>::Error: ::rkyv::bytecheck::rancor::Trace,
    <usize as ::rkyv::Archive>::Archived: ::rkyv::bytecheck::CheckBytes<__C>,
    <Vec<u8> as ::rkyv::Archive>::Archived: ::rkyv::bytecheck::CheckBytes<__C>,
{
    unsafe fn check_bytes(
        value: *const Self,
        context: &mut __C,
    ) -> ::core::result::Result<
        (),
        <__C as ::rkyv::bytecheck::rancor::Fallible>::Error,
    > {
        <<usize as ::rkyv::Archive>::Archived as ::rkyv::bytecheck::CheckBytes<
            __C,
        >>::check_bytes(&raw const (*value).index, context)
            .map_err(|e| {
                <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Trace>::trace(
                    e,
                    ::rkyv::bytecheck::StructCheckContext {
                        struct_name: "ArchivedPartialFieldBlobPayloadChunk",
                        field_name: "index",
                    },
                )
            })?;
        <<Vec<
            u8,
        > as ::rkyv::Archive>::Archived as ::rkyv::bytecheck::CheckBytes<
            __C,
        >>::check_bytes(&raw const (*value).data, context)
            .map_err(|e| {
                <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Trace>::trace(
                    e,
                    ::rkyv::bytecheck::StructCheckContext {
                        struct_name: "ArchivedPartialFieldBlobPayloadChunk",
                        field_name: "data",
                    },
                )
            })?;
        ::core::result::Result::Ok(())
    }
}
#[automatically_derived]
///The resolver for an archived [`PartialFieldBlobPayloadChunk`]
pub struct PartialFieldBlobPayloadChunkResolver
where
    usize: ::rkyv::Archive,
    Vec<u8>: ::rkyv::Archive,
{
    index: <usize as ::rkyv::Archive>::Resolver,
    data: <Vec<u8> as ::rkyv::Archive>::Resolver,
}
impl ::rkyv::Archive for PartialFieldBlobPayloadChunk
where
    usize: ::rkyv::Archive,
    Vec<u8>: ::rkyv::Archive,
{
    type Archived = ArchivedPartialFieldBlobPayloadChunk;
    type Resolver = PartialFieldBlobPayloadChunkResolver;
    const COPY_OPTIMIZATION: ::rkyv::traits::CopyOptimization<Self> = unsafe {
        ::rkyv::traits::CopyOptimization::enable_if(
            0 + ::core::mem::size_of::<usize>() + ::core::mem::size_of::<Vec<u8>>()
                == ::core::mem::size_of::<PartialFieldBlobPayloadChunk>()
                && <usize as ::rkyv::Archive>::COPY_OPTIMIZATION.is_enabled()
                && const { builtin # offset_of(PartialFieldBlobPayloadChunk, index) }
                    == const {
                        builtin # offset_of(ArchivedPartialFieldBlobPayloadChunk, index)
                    } && <Vec<u8> as ::rkyv::Archive>::COPY_OPTIMIZATION.is_enabled()
                && const { builtin # offset_of(PartialFieldBlobPayloadChunk, data) }
                    == const {
                        builtin # offset_of(ArchivedPartialFieldBlobPayloadChunk, data)
                    },
        )
    };
    #[allow(clippy::unit_arg)]
    fn resolve(&self, resolver: Self::Resolver, out: ::rkyv::Place<Self::Archived>) {
        let field_ptr = unsafe { &raw mut (*out.ptr()).index };
        let field_out = unsafe { ::rkyv::Place::from_field_unchecked(out, field_ptr) };
        <usize as ::rkyv::Archive>::resolve(&self.index, resolver.index, field_out);
        let field_ptr = unsafe { &raw mut (*out.ptr()).data };
        let field_out = unsafe { ::rkyv::Place::from_field_unchecked(out, field_ptr) };
        <Vec<u8> as ::rkyv::Archive>::resolve(&self.data, resolver.data, field_out);
    }
}
unsafe impl ::rkyv::traits::Portable for ArchivedPartialFieldBlobPayloadChunk
where
    usize: ::rkyv::Archive,
    Vec<u8>: ::rkyv::Archive,
    <usize as ::rkyv::Archive>::Archived: ::rkyv::traits::Portable,
    <Vec<u8> as ::rkyv::Archive>::Archived: ::rkyv::traits::Portable,
{}
#[automatically_derived]
impl<__S: ::rkyv::rancor::Fallible + ?Sized> ::rkyv::Serialize<__S>
for PartialFieldBlobPayloadChunk
where
    usize: ::rkyv::Serialize<__S>,
    Vec<u8>: ::rkyv::Serialize<__S>,
{
    fn serialize(
        &self,
        serializer: &mut __S,
    ) -> ::core::result::Result<
        <Self as ::rkyv::Archive>::Resolver,
        <__S as ::rkyv::rancor::Fallible>::Error,
    > {
        let __this = self;
        ::core::result::Result::Ok(PartialFieldBlobPayloadChunkResolver {
            index: <usize as ::rkyv::Serialize<
                __S,
            >>::serialize(&__this.index, serializer)?,
            data: <Vec<
                u8,
            > as ::rkyv::Serialize<__S>>::serialize(&__this.data, serializer)?,
        })
    }
}
#[automatically_derived]
impl<
    __D: ::rkyv::rancor::Fallible + ?Sized,
> ::rkyv::Deserialize<PartialFieldBlobPayloadChunk, __D>
for ::rkyv::Archived<PartialFieldBlobPayloadChunk>
where
    usize: ::rkyv::Archive,
    <usize as ::rkyv::Archive>::Archived: ::rkyv::Deserialize<usize, __D>,
    Vec<u8>: ::rkyv::Archive,
    <Vec<u8> as ::rkyv::Archive>::Archived: ::rkyv::Deserialize<Vec<u8>, __D>,
{
    fn deserialize(
        &self,
        deserializer: &mut __D,
    ) -> ::core::result::Result<
        PartialFieldBlobPayloadChunk,
        <__D as ::rkyv::rancor::Fallible>::Error,
    > {
        let __this = self;
        ::core::result::Result::Ok(PartialFieldBlobPayloadChunk {
            index: <<usize as ::rkyv::Archive>::Archived as ::rkyv::Deserialize<
                usize,
                __D,
            >>::deserialize(&__this.index, deserializer)?,
            data: <<Vec<
                u8,
            > as ::rkyv::Archive>::Archived as ::rkyv::Deserialize<
                Vec<u8>,
                __D,
            >>::deserialize(&__this.data, deserializer)?,
        })
    }
}
impl ::rewrite::traits::structural::blob::BlobItemChunk
for PartialFieldBlobPayloadChunk {
    type Index = usize;
    fn get_index(&self) -> &Self::Index {
        &self.index
    }
}
pub enum PartialFieldBlobChunk {
    Header(PartialFieldBlobHeaderChunk),
    Payload(PartialFieldBlobPayloadChunk),
    Missing,
}
#[automatically_derived]
impl ::core::fmt::Debug for PartialFieldBlobChunk {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match self {
            PartialFieldBlobChunk::Header(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "Header", &__self_0)
            }
            PartialFieldBlobChunk::Payload(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(
                    f,
                    "Payload",
                    &__self_0,
                )
            }
            PartialFieldBlobChunk::Missing => {
                ::core::fmt::Formatter::write_str(f, "Missing")
            }
        }
    }
}
#[automatically_derived]
impl ::core::clone::Clone for PartialFieldBlobChunk {
    #[inline]
    fn clone(&self) -> PartialFieldBlobChunk {
        match self {
            PartialFieldBlobChunk::Header(__self_0) => {
                PartialFieldBlobChunk::Header(::core::clone::Clone::clone(__self_0))
            }
            PartialFieldBlobChunk::Payload(__self_0) => {
                PartialFieldBlobChunk::Payload(::core::clone::Clone::clone(__self_0))
            }
            PartialFieldBlobChunk::Missing => PartialFieldBlobChunk::Missing,
        }
    }
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for PartialFieldBlobChunk {}
#[automatically_derived]
impl ::core::cmp::PartialEq for PartialFieldBlobChunk {
    #[inline]
    fn eq(&self, other: &PartialFieldBlobChunk) -> bool {
        let __self_discr = ::core::intrinsics::discriminant_value(self);
        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
        __self_discr == __arg1_discr
            && match (self, other) {
                (
                    PartialFieldBlobChunk::Header(__self_0),
                    PartialFieldBlobChunk::Header(__arg1_0),
                ) => __self_0 == __arg1_0,
                (
                    PartialFieldBlobChunk::Payload(__self_0),
                    PartialFieldBlobChunk::Payload(__arg1_0),
                ) => __self_0 == __arg1_0,
                _ => true,
            }
    }
}
#[automatically_derived]
impl ::core::cmp::Eq for PartialFieldBlobChunk {
    #[inline]
    #[doc(hidden)]
    #[coverage(off)]
    fn assert_fields_are_eq(&self) {
        let _: ::core::cmp::AssertParamIsEq<PartialFieldBlobHeaderChunk>;
        let _: ::core::cmp::AssertParamIsEq<PartialFieldBlobPayloadChunk>;
    }
}
#[automatically_derived]
impl ::core::cmp::PartialOrd for PartialFieldBlobChunk {
    #[inline]
    fn partial_cmp(
        &self,
        other: &PartialFieldBlobChunk,
    ) -> ::core::option::Option<::core::cmp::Ordering> {
        let __self_discr = ::core::intrinsics::discriminant_value(self);
        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
        match (self, other) {
            (
                PartialFieldBlobChunk::Header(__self_0),
                PartialFieldBlobChunk::Header(__arg1_0),
            ) => ::core::cmp::PartialOrd::partial_cmp(__self_0, __arg1_0),
            (
                PartialFieldBlobChunk::Payload(__self_0),
                PartialFieldBlobChunk::Payload(__arg1_0),
            ) => ::core::cmp::PartialOrd::partial_cmp(__self_0, __arg1_0),
            _ => ::core::cmp::PartialOrd::partial_cmp(&__self_discr, &__arg1_discr),
        }
    }
}
#[automatically_derived]
impl ::core::cmp::Ord for PartialFieldBlobChunk {
    #[inline]
    fn cmp(&self, other: &PartialFieldBlobChunk) -> ::core::cmp::Ordering {
        let __self_discr = ::core::intrinsics::discriminant_value(self);
        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
        match ::core::cmp::Ord::cmp(&__self_discr, &__arg1_discr) {
            ::core::cmp::Ordering::Equal => {
                match (self, other) {
                    (
                        PartialFieldBlobChunk::Header(__self_0),
                        PartialFieldBlobChunk::Header(__arg1_0),
                    ) => ::core::cmp::Ord::cmp(__self_0, __arg1_0),
                    (
                        PartialFieldBlobChunk::Payload(__self_0),
                        PartialFieldBlobChunk::Payload(__arg1_0),
                    ) => ::core::cmp::Ord::cmp(__self_0, __arg1_0),
                    _ => ::core::cmp::Ordering::Equal,
                }
            }
            cmp => cmp,
        }
    }
}
#[automatically_derived]
///An archived [`PartialFieldBlobChunk`]
#[bytecheck(crate = ::rkyv::bytecheck)]
#[repr(u8)]
pub enum ArchivedPartialFieldBlobChunk
where
    PartialFieldBlobHeaderChunk: ::rkyv::Archive,
    PartialFieldBlobPayloadChunk: ::rkyv::Archive,
{
    ///The archived counterpart of [`PartialFieldBlobChunk::Header`]
    #[allow(dead_code)]
    Header(
        ///The archived counterpart of [`PartialFieldBlobChunk::Header::0`]
        <PartialFieldBlobHeaderChunk as ::rkyv::Archive>::Archived,
    ),
    ///The archived counterpart of [`PartialFieldBlobChunk::Payload`]
    #[allow(dead_code)]
    Payload(
        ///The archived counterpart of [`PartialFieldBlobChunk::Payload::0`]
        <PartialFieldBlobPayloadChunk as ::rkyv::Archive>::Archived,
    ),
    ///The archived counterpart of [`PartialFieldBlobChunk::Missing`]
    #[allow(dead_code)]
    Missing,
}
const _: () = {
    #[repr(u8)]
    enum Tag {
        Header,
        Payload,
        Missing,
    }
    struct Discriminant;
    #[automatically_derived]
    impl Discriminant {
        #[allow(non_upper_case_globals)]
        const Header: u8 = Tag::Header as u8;
        #[allow(non_upper_case_globals)]
        const Payload: u8 = Tag::Payload as u8;
        #[allow(non_upper_case_globals)]
        const Missing: u8 = Tag::Missing as u8;
    }
    #[repr(C)]
    struct VariantHeader(
        Tag,
        <PartialFieldBlobHeaderChunk as ::rkyv::Archive>::Archived,
        ::core::marker::PhantomData<ArchivedPartialFieldBlobChunk>,
    )
    where
        PartialFieldBlobHeaderChunk: ::rkyv::Archive,
        PartialFieldBlobPayloadChunk: ::rkyv::Archive;
    #[repr(C)]
    struct VariantPayload(
        Tag,
        <PartialFieldBlobPayloadChunk as ::rkyv::Archive>::Archived,
        ::core::marker::PhantomData<ArchivedPartialFieldBlobChunk>,
    )
    where
        PartialFieldBlobHeaderChunk: ::rkyv::Archive,
        PartialFieldBlobPayloadChunk: ::rkyv::Archive;
    #[automatically_derived]
    unsafe impl<
        __C: ::rkyv::bytecheck::rancor::Fallible + ?::core::marker::Sized,
    > ::rkyv::bytecheck::CheckBytes<__C> for ArchivedPartialFieldBlobChunk
    where
        PartialFieldBlobHeaderChunk: ::rkyv::Archive,
        PartialFieldBlobPayloadChunk: ::rkyv::Archive,
        <__C as ::rkyv::bytecheck::rancor::Fallible>::Error: ::rkyv::bytecheck::rancor::Source,
        <PartialFieldBlobHeaderChunk as ::rkyv::Archive>::Archived: ::rkyv::bytecheck::CheckBytes<
            __C,
        >,
        <PartialFieldBlobPayloadChunk as ::rkyv::Archive>::Archived: ::rkyv::bytecheck::CheckBytes<
            __C,
        >,
    {
        unsafe fn check_bytes(
            value: *const Self,
            context: &mut __C,
        ) -> ::core::result::Result<
            (),
            <__C as ::rkyv::bytecheck::rancor::Fallible>::Error,
        > {
            let tag = *value.cast::<u8>();
            match tag {
                Discriminant::Header => {
                    let value = value.cast::<VariantHeader>();
                    <<PartialFieldBlobHeaderChunk as ::rkyv::Archive>::Archived as ::rkyv::bytecheck::CheckBytes<
                        __C,
                    >>::check_bytes(&raw const (*value).1, context)
                        .map_err(|e| {
                            <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Trace>::trace(
                                e,
                                ::rkyv::bytecheck::UnnamedEnumVariantCheckContext {
                                    enum_name: "ArchivedPartialFieldBlobChunk",
                                    variant_name: "Header",
                                    field_index: 1,
                                },
                            )
                        })?;
                }
                Discriminant::Payload => {
                    let value = value.cast::<VariantPayload>();
                    <<PartialFieldBlobPayloadChunk as ::rkyv::Archive>::Archived as ::rkyv::bytecheck::CheckBytes<
                        __C,
                    >>::check_bytes(&raw const (*value).1, context)
                        .map_err(|e| {
                            <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Trace>::trace(
                                e,
                                ::rkyv::bytecheck::UnnamedEnumVariantCheckContext {
                                    enum_name: "ArchivedPartialFieldBlobChunk",
                                    variant_name: "Payload",
                                    field_index: 1,
                                },
                            )
                        })?;
                }
                Discriminant::Missing => {}
                _ => {
                    return ::core::result::Result::Err(
                        <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Source>::new(::rkyv::bytecheck::InvalidEnumDiscriminantError {
                            enum_name: "ArchivedPartialFieldBlobChunk",
                            invalid_discriminant: tag,
                        }),
                    );
                }
            }
            ::core::result::Result::Ok(())
        }
    }
};
#[automatically_derived]
///The resolver for an archived [`PartialFieldBlobChunk`]
pub enum PartialFieldBlobChunkResolver
where
    PartialFieldBlobHeaderChunk: ::rkyv::Archive,
    PartialFieldBlobPayloadChunk: ::rkyv::Archive,
{
    ///The resolver for [`PartialFieldBlobChunk::Header`]
    #[allow(dead_code)]
    Header(<PartialFieldBlobHeaderChunk as ::rkyv::Archive>::Resolver),
    ///The resolver for [`PartialFieldBlobChunk::Payload`]
    #[allow(dead_code)]
    Payload(<PartialFieldBlobPayloadChunk as ::rkyv::Archive>::Resolver),
    ///The resolver for [`PartialFieldBlobChunk::Missing`]
    #[allow(dead_code)]
    Missing,
}
const _: () = {
    #[repr(u8)]
    enum ArchivedTag {
        Header,
        Payload,
        Missing,
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for ArchivedTag {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for ArchivedTag {
        #[inline]
        fn eq(&self, other: &ArchivedTag) -> bool {
            let __self_discr = ::core::intrinsics::discriminant_value(self);
            let __arg1_discr = ::core::intrinsics::discriminant_value(other);
            __self_discr == __arg1_discr
        }
    }
    #[automatically_derived]
    impl ::core::cmp::PartialOrd for ArchivedTag {
        #[inline]
        fn partial_cmp(
            &self,
            other: &ArchivedTag,
        ) -> ::core::option::Option<::core::cmp::Ordering> {
            let __self_discr = ::core::intrinsics::discriminant_value(self);
            let __arg1_discr = ::core::intrinsics::discriminant_value(other);
            ::core::cmp::PartialOrd::partial_cmp(&__self_discr, &__arg1_discr)
        }
    }
    #[repr(C)]
    struct ArchivedVariantHeader(
        ArchivedTag,
        <PartialFieldBlobHeaderChunk as ::rkyv::Archive>::Archived,
        ::core::marker::PhantomData<PartialFieldBlobChunk>,
    )
    where
        PartialFieldBlobHeaderChunk: ::rkyv::Archive,
        PartialFieldBlobPayloadChunk: ::rkyv::Archive;
    #[repr(C)]
    struct ArchivedVariantPayload(
        ArchivedTag,
        <PartialFieldBlobPayloadChunk as ::rkyv::Archive>::Archived,
        ::core::marker::PhantomData<PartialFieldBlobChunk>,
    )
    where
        PartialFieldBlobHeaderChunk: ::rkyv::Archive,
        PartialFieldBlobPayloadChunk: ::rkyv::Archive;
    impl ::rkyv::Archive for PartialFieldBlobChunk
    where
        PartialFieldBlobHeaderChunk: ::rkyv::Archive,
        PartialFieldBlobPayloadChunk: ::rkyv::Archive,
    {
        type Archived = ArchivedPartialFieldBlobChunk;
        type Resolver = PartialFieldBlobChunkResolver;
        #[allow(clippy::unit_arg)]
        fn resolve(
            &self,
            resolver: <Self as ::rkyv::Archive>::Resolver,
            out: ::rkyv::Place<<Self as ::rkyv::Archive>::Archived>,
        ) {
            let __this = self;
            match resolver {
                PartialFieldBlobChunkResolver::Header(resolver_0) => {
                    match __this {
                        PartialFieldBlobChunk::Header(self_0, ..) => {
                            let out = unsafe {
                                out.cast_unchecked::<ArchivedVariantHeader>()
                            };
                            let tag_ptr = unsafe { &raw mut (*out.ptr()).0 };
                            unsafe {
                                tag_ptr.write(ArchivedTag::Header);
                            }
                            let field_ptr = unsafe { &raw mut (*out.ptr()).1 };
                            let out_field = unsafe {
                                ::rkyv::Place::from_field_unchecked(out, field_ptr)
                            };
                            <PartialFieldBlobHeaderChunk as ::rkyv::Archive>::resolve(
                                self_0,
                                resolver_0,
                                out_field,
                            );
                        }
                        #[allow(unreachable_patterns)]
                        _ => unsafe { ::core::hint::unreachable_unchecked() }
                    }
                }
                PartialFieldBlobChunkResolver::Payload(resolver_0) => {
                    match __this {
                        PartialFieldBlobChunk::Payload(self_0, ..) => {
                            let out = unsafe {
                                out.cast_unchecked::<ArchivedVariantPayload>()
                            };
                            let tag_ptr = unsafe { &raw mut (*out.ptr()).0 };
                            unsafe {
                                tag_ptr.write(ArchivedTag::Payload);
                            }
                            let field_ptr = unsafe { &raw mut (*out.ptr()).1 };
                            let out_field = unsafe {
                                ::rkyv::Place::from_field_unchecked(out, field_ptr)
                            };
                            <PartialFieldBlobPayloadChunk as ::rkyv::Archive>::resolve(
                                self_0,
                                resolver_0,
                                out_field,
                            );
                        }
                        #[allow(unreachable_patterns)]
                        _ => unsafe { ::core::hint::unreachable_unchecked() }
                    }
                }
                PartialFieldBlobChunkResolver::Missing => {
                    let out = unsafe { out.cast_unchecked::<ArchivedTag>() };
                    unsafe {
                        out.write_unchecked(ArchivedTag::Missing);
                    }
                }
            }
        }
    }
};
unsafe impl ::rkyv::traits::Portable for ArchivedPartialFieldBlobChunk
where
    PartialFieldBlobHeaderChunk: ::rkyv::Archive,
    PartialFieldBlobPayloadChunk: ::rkyv::Archive,
    <PartialFieldBlobHeaderChunk as ::rkyv::Archive>::Archived: ::rkyv::traits::Portable,
    <PartialFieldBlobPayloadChunk as ::rkyv::Archive>::Archived: ::rkyv::traits::Portable,
{}
#[automatically_derived]
impl<__S: ::rkyv::rancor::Fallible + ?Sized> ::rkyv::Serialize<__S>
for PartialFieldBlobChunk
where
    PartialFieldBlobHeaderChunk: ::rkyv::Serialize<__S>,
    PartialFieldBlobPayloadChunk: ::rkyv::Serialize<__S>,
{
    fn serialize(
        &self,
        serializer: &mut __S,
    ) -> ::core::result::Result<
        <Self as ::rkyv::Archive>::Resolver,
        <__S as ::rkyv::rancor::Fallible>::Error,
    > {
        let __this = self;
        ::core::result::Result::Ok(
            match __this {
                PartialFieldBlobChunk::Header(_0, ..) => {
                    PartialFieldBlobChunkResolver::Header(
                        <PartialFieldBlobHeaderChunk as ::rkyv::Serialize<
                            __S,
                        >>::serialize(_0, serializer)?,
                    )
                }
                PartialFieldBlobChunk::Payload(_0, ..) => {
                    PartialFieldBlobChunkResolver::Payload(
                        <PartialFieldBlobPayloadChunk as ::rkyv::Serialize<
                            __S,
                        >>::serialize(_0, serializer)?,
                    )
                }
                PartialFieldBlobChunk::Missing => PartialFieldBlobChunkResolver::Missing,
            },
        )
    }
}
#[automatically_derived]
impl<
    __D: ::rkyv::rancor::Fallible + ?Sized,
> ::rkyv::Deserialize<PartialFieldBlobChunk, __D>
for ::rkyv::Archived<PartialFieldBlobChunk>
where
    PartialFieldBlobHeaderChunk: ::rkyv::Archive,
    <PartialFieldBlobHeaderChunk as ::rkyv::Archive>::Archived: ::rkyv::Deserialize<
        PartialFieldBlobHeaderChunk,
        __D,
    >,
    PartialFieldBlobPayloadChunk: ::rkyv::Archive,
    <PartialFieldBlobPayloadChunk as ::rkyv::Archive>::Archived: ::rkyv::Deserialize<
        PartialFieldBlobPayloadChunk,
        __D,
    >,
{
    fn deserialize(
        &self,
        deserializer: &mut __D,
    ) -> ::core::result::Result<
        PartialFieldBlobChunk,
        <__D as ::rkyv::rancor::Fallible>::Error,
    > {
        let __this = self;
        ::core::result::Result::Ok(
            match __this {
                Self::Header(_0, ..) => {
                    PartialFieldBlobChunk::Header(
                        <<PartialFieldBlobHeaderChunk as ::rkyv::Archive>::Archived as ::rkyv::Deserialize<
                            PartialFieldBlobHeaderChunk,
                            __D,
                        >>::deserialize(_0, deserializer)?,
                    )
                }
                Self::Payload(_0, ..) => {
                    PartialFieldBlobChunk::Payload(
                        <<PartialFieldBlobPayloadChunk as ::rkyv::Archive>::Archived as ::rkyv::Deserialize<
                            PartialFieldBlobPayloadChunk,
                            __D,
                        >>::deserialize(_0, deserializer)?,
                    )
                }
                Self::Missing => PartialFieldBlobChunk::Missing,
            },
        )
    }
}
pub enum PartialFieldBlobChunkFill {
    Full(usize),
    Partial(usize),
    Corrupted(usize),
}
#[automatically_derived]
impl ::core::fmt::Debug for PartialFieldBlobChunkFill {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match self {
            PartialFieldBlobChunkFill::Full(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "Full", &__self_0)
            }
            PartialFieldBlobChunkFill::Partial(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(
                    f,
                    "Partial",
                    &__self_0,
                )
            }
            PartialFieldBlobChunkFill::Corrupted(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(
                    f,
                    "Corrupted",
                    &__self_0,
                )
            }
        }
    }
}
#[automatically_derived]
#[doc(hidden)]
unsafe impl ::core::clone::TrivialClone for PartialFieldBlobChunkFill {}
#[automatically_derived]
impl ::core::clone::Clone for PartialFieldBlobChunkFill {
    #[inline]
    fn clone(&self) -> PartialFieldBlobChunkFill {
        let _: ::core::clone::AssertParamIsClone<usize>;
        *self
    }
}
#[automatically_derived]
impl ::core::marker::Copy for PartialFieldBlobChunkFill {}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for PartialFieldBlobChunkFill {}
#[automatically_derived]
impl ::core::cmp::PartialEq for PartialFieldBlobChunkFill {
    #[inline]
    fn eq(&self, other: &PartialFieldBlobChunkFill) -> bool {
        let __self_discr = ::core::intrinsics::discriminant_value(self);
        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
        __self_discr == __arg1_discr
            && match (self, other) {
                (
                    PartialFieldBlobChunkFill::Full(__self_0),
                    PartialFieldBlobChunkFill::Full(__arg1_0),
                ) => __self_0 == __arg1_0,
                (
                    PartialFieldBlobChunkFill::Partial(__self_0),
                    PartialFieldBlobChunkFill::Partial(__arg1_0),
                ) => __self_0 == __arg1_0,
                (
                    PartialFieldBlobChunkFill::Corrupted(__self_0),
                    PartialFieldBlobChunkFill::Corrupted(__arg1_0),
                ) => __self_0 == __arg1_0,
                _ => unsafe { ::core::intrinsics::unreachable() }
            }
    }
}
#[automatically_derived]
impl ::core::cmp::Eq for PartialFieldBlobChunkFill {
    #[inline]
    #[doc(hidden)]
    #[coverage(off)]
    fn assert_fields_are_eq(&self) {
        let _: ::core::cmp::AssertParamIsEq<usize>;
    }
}
#[automatically_derived]
impl ::core::cmp::PartialOrd for PartialFieldBlobChunkFill {
    #[inline]
    fn partial_cmp(
        &self,
        other: &PartialFieldBlobChunkFill,
    ) -> ::core::option::Option<::core::cmp::Ordering> {
        let __self_discr = ::core::intrinsics::discriminant_value(self);
        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
        match (self, other) {
            (
                PartialFieldBlobChunkFill::Full(__self_0),
                PartialFieldBlobChunkFill::Full(__arg1_0),
            ) => ::core::cmp::PartialOrd::partial_cmp(__self_0, __arg1_0),
            (
                PartialFieldBlobChunkFill::Partial(__self_0),
                PartialFieldBlobChunkFill::Partial(__arg1_0),
            ) => ::core::cmp::PartialOrd::partial_cmp(__self_0, __arg1_0),
            (
                PartialFieldBlobChunkFill::Corrupted(__self_0),
                PartialFieldBlobChunkFill::Corrupted(__arg1_0),
            ) => ::core::cmp::PartialOrd::partial_cmp(__self_0, __arg1_0),
            _ => ::core::cmp::PartialOrd::partial_cmp(&__self_discr, &__arg1_discr),
        }
    }
}
#[automatically_derived]
impl ::core::cmp::Ord for PartialFieldBlobChunkFill {
    #[inline]
    fn cmp(&self, other: &PartialFieldBlobChunkFill) -> ::core::cmp::Ordering {
        let __self_discr = ::core::intrinsics::discriminant_value(self);
        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
        match ::core::cmp::Ord::cmp(&__self_discr, &__arg1_discr) {
            ::core::cmp::Ordering::Equal => {
                match (self, other) {
                    (
                        PartialFieldBlobChunkFill::Full(__self_0),
                        PartialFieldBlobChunkFill::Full(__arg1_0),
                    ) => ::core::cmp::Ord::cmp(__self_0, __arg1_0),
                    (
                        PartialFieldBlobChunkFill::Partial(__self_0),
                        PartialFieldBlobChunkFill::Partial(__arg1_0),
                    ) => ::core::cmp::Ord::cmp(__self_0, __arg1_0),
                    (
                        PartialFieldBlobChunkFill::Corrupted(__self_0),
                        PartialFieldBlobChunkFill::Corrupted(__arg1_0),
                    ) => ::core::cmp::Ord::cmp(__self_0, __arg1_0),
                    _ => unsafe { ::core::intrinsics::unreachable() }
                }
            }
            cmp => cmp,
        }
    }
}
#[automatically_derived]
///An archived [`PartialFieldBlobChunkFill`]
#[bytecheck(crate = ::rkyv::bytecheck)]
#[repr(u8)]
pub enum ArchivedPartialFieldBlobChunkFill
where
    usize: ::rkyv::Archive,
    usize: ::rkyv::Archive,
    usize: ::rkyv::Archive,
{
    ///The archived counterpart of [`PartialFieldBlobChunkFill::Full`]
    #[allow(dead_code)]
    Full(
        ///The archived counterpart of [`PartialFieldBlobChunkFill::Full::0`]
        <usize as ::rkyv::Archive>::Archived,
    ),
    ///The archived counterpart of [`PartialFieldBlobChunkFill::Partial`]
    #[allow(dead_code)]
    Partial(
        ///The archived counterpart of [`PartialFieldBlobChunkFill::Partial::0`]
        <usize as ::rkyv::Archive>::Archived,
    ),
    ///The archived counterpart of [`PartialFieldBlobChunkFill::Corrupted`]
    #[allow(dead_code)]
    Corrupted(
        ///The archived counterpart of [`PartialFieldBlobChunkFill::Corrupted::0`]
        <usize as ::rkyv::Archive>::Archived,
    ),
}
const _: () = {
    #[repr(u8)]
    enum Tag {
        Full,
        Partial,
        Corrupted,
    }
    struct Discriminant;
    #[automatically_derived]
    impl Discriminant {
        #[allow(non_upper_case_globals)]
        const Full: u8 = Tag::Full as u8;
        #[allow(non_upper_case_globals)]
        const Partial: u8 = Tag::Partial as u8;
        #[allow(non_upper_case_globals)]
        const Corrupted: u8 = Tag::Corrupted as u8;
    }
    #[repr(C)]
    struct VariantFull(
        Tag,
        <usize as ::rkyv::Archive>::Archived,
        ::core::marker::PhantomData<ArchivedPartialFieldBlobChunkFill>,
    )
    where
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive;
    #[repr(C)]
    struct VariantPartial(
        Tag,
        <usize as ::rkyv::Archive>::Archived,
        ::core::marker::PhantomData<ArchivedPartialFieldBlobChunkFill>,
    )
    where
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive;
    #[repr(C)]
    struct VariantCorrupted(
        Tag,
        <usize as ::rkyv::Archive>::Archived,
        ::core::marker::PhantomData<ArchivedPartialFieldBlobChunkFill>,
    )
    where
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive;
    #[automatically_derived]
    unsafe impl<
        __C: ::rkyv::bytecheck::rancor::Fallible + ?::core::marker::Sized,
    > ::rkyv::bytecheck::CheckBytes<__C> for ArchivedPartialFieldBlobChunkFill
    where
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive,
        <__C as ::rkyv::bytecheck::rancor::Fallible>::Error: ::rkyv::bytecheck::rancor::Source,
        <usize as ::rkyv::Archive>::Archived: ::rkyv::bytecheck::CheckBytes<__C>,
        <usize as ::rkyv::Archive>::Archived: ::rkyv::bytecheck::CheckBytes<__C>,
        <usize as ::rkyv::Archive>::Archived: ::rkyv::bytecheck::CheckBytes<__C>,
    {
        unsafe fn check_bytes(
            value: *const Self,
            context: &mut __C,
        ) -> ::core::result::Result<
            (),
            <__C as ::rkyv::bytecheck::rancor::Fallible>::Error,
        > {
            let tag = *value.cast::<u8>();
            match tag {
                Discriminant::Full => {
                    let value = value.cast::<VariantFull>();
                    <<usize as ::rkyv::Archive>::Archived as ::rkyv::bytecheck::CheckBytes<
                        __C,
                    >>::check_bytes(&raw const (*value).1, context)
                        .map_err(|e| {
                            <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Trace>::trace(
                                e,
                                ::rkyv::bytecheck::UnnamedEnumVariantCheckContext {
                                    enum_name: "ArchivedPartialFieldBlobChunkFill",
                                    variant_name: "Full",
                                    field_index: 1,
                                },
                            )
                        })?;
                }
                Discriminant::Partial => {
                    let value = value.cast::<VariantPartial>();
                    <<usize as ::rkyv::Archive>::Archived as ::rkyv::bytecheck::CheckBytes<
                        __C,
                    >>::check_bytes(&raw const (*value).1, context)
                        .map_err(|e| {
                            <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Trace>::trace(
                                e,
                                ::rkyv::bytecheck::UnnamedEnumVariantCheckContext {
                                    enum_name: "ArchivedPartialFieldBlobChunkFill",
                                    variant_name: "Partial",
                                    field_index: 1,
                                },
                            )
                        })?;
                }
                Discriminant::Corrupted => {
                    let value = value.cast::<VariantCorrupted>();
                    <<usize as ::rkyv::Archive>::Archived as ::rkyv::bytecheck::CheckBytes<
                        __C,
                    >>::check_bytes(&raw const (*value).1, context)
                        .map_err(|e| {
                            <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Trace>::trace(
                                e,
                                ::rkyv::bytecheck::UnnamedEnumVariantCheckContext {
                                    enum_name: "ArchivedPartialFieldBlobChunkFill",
                                    variant_name: "Corrupted",
                                    field_index: 1,
                                },
                            )
                        })?;
                }
                _ => {
                    return ::core::result::Result::Err(
                        <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Source>::new(::rkyv::bytecheck::InvalidEnumDiscriminantError {
                            enum_name: "ArchivedPartialFieldBlobChunkFill",
                            invalid_discriminant: tag,
                        }),
                    );
                }
            }
            ::core::result::Result::Ok(())
        }
    }
};
#[automatically_derived]
///The resolver for an archived [`PartialFieldBlobChunkFill`]
pub enum PartialFieldBlobChunkFillResolver
where
    usize: ::rkyv::Archive,
    usize: ::rkyv::Archive,
    usize: ::rkyv::Archive,
{
    ///The resolver for [`PartialFieldBlobChunkFill::Full`]
    #[allow(dead_code)]
    Full(<usize as ::rkyv::Archive>::Resolver),
    ///The resolver for [`PartialFieldBlobChunkFill::Partial`]
    #[allow(dead_code)]
    Partial(<usize as ::rkyv::Archive>::Resolver),
    ///The resolver for [`PartialFieldBlobChunkFill::Corrupted`]
    #[allow(dead_code)]
    Corrupted(<usize as ::rkyv::Archive>::Resolver),
}
const _: () = {
    #[repr(u8)]
    enum ArchivedTag {
        Full,
        Partial,
        Corrupted,
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for ArchivedTag {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for ArchivedTag {
        #[inline]
        fn eq(&self, other: &ArchivedTag) -> bool {
            let __self_discr = ::core::intrinsics::discriminant_value(self);
            let __arg1_discr = ::core::intrinsics::discriminant_value(other);
            __self_discr == __arg1_discr
        }
    }
    #[automatically_derived]
    impl ::core::cmp::PartialOrd for ArchivedTag {
        #[inline]
        fn partial_cmp(
            &self,
            other: &ArchivedTag,
        ) -> ::core::option::Option<::core::cmp::Ordering> {
            let __self_discr = ::core::intrinsics::discriminant_value(self);
            let __arg1_discr = ::core::intrinsics::discriminant_value(other);
            ::core::cmp::PartialOrd::partial_cmp(&__self_discr, &__arg1_discr)
        }
    }
    #[repr(C)]
    struct ArchivedVariantFull(
        ArchivedTag,
        <usize as ::rkyv::Archive>::Archived,
        ::core::marker::PhantomData<PartialFieldBlobChunkFill>,
    )
    where
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive;
    #[repr(C)]
    struct ArchivedVariantPartial(
        ArchivedTag,
        <usize as ::rkyv::Archive>::Archived,
        ::core::marker::PhantomData<PartialFieldBlobChunkFill>,
    )
    where
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive;
    #[repr(C)]
    struct ArchivedVariantCorrupted(
        ArchivedTag,
        <usize as ::rkyv::Archive>::Archived,
        ::core::marker::PhantomData<PartialFieldBlobChunkFill>,
    )
    where
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive;
    impl ::rkyv::Archive for PartialFieldBlobChunkFill
    where
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive,
    {
        type Archived = ArchivedPartialFieldBlobChunkFill;
        type Resolver = PartialFieldBlobChunkFillResolver;
        #[allow(clippy::unit_arg)]
        fn resolve(
            &self,
            resolver: <Self as ::rkyv::Archive>::Resolver,
            out: ::rkyv::Place<<Self as ::rkyv::Archive>::Archived>,
        ) {
            let __this = self;
            match resolver {
                PartialFieldBlobChunkFillResolver::Full(resolver_0) => {
                    match __this {
                        PartialFieldBlobChunkFill::Full(self_0, ..) => {
                            let out = unsafe {
                                out.cast_unchecked::<ArchivedVariantFull>()
                            };
                            let tag_ptr = unsafe { &raw mut (*out.ptr()).0 };
                            unsafe {
                                tag_ptr.write(ArchivedTag::Full);
                            }
                            let field_ptr = unsafe { &raw mut (*out.ptr()).1 };
                            let out_field = unsafe {
                                ::rkyv::Place::from_field_unchecked(out, field_ptr)
                            };
                            <usize as ::rkyv::Archive>::resolve(
                                self_0,
                                resolver_0,
                                out_field,
                            );
                        }
                        #[allow(unreachable_patterns)]
                        _ => unsafe { ::core::hint::unreachable_unchecked() }
                    }
                }
                PartialFieldBlobChunkFillResolver::Partial(resolver_0) => {
                    match __this {
                        PartialFieldBlobChunkFill::Partial(self_0, ..) => {
                            let out = unsafe {
                                out.cast_unchecked::<ArchivedVariantPartial>()
                            };
                            let tag_ptr = unsafe { &raw mut (*out.ptr()).0 };
                            unsafe {
                                tag_ptr.write(ArchivedTag::Partial);
                            }
                            let field_ptr = unsafe { &raw mut (*out.ptr()).1 };
                            let out_field = unsafe {
                                ::rkyv::Place::from_field_unchecked(out, field_ptr)
                            };
                            <usize as ::rkyv::Archive>::resolve(
                                self_0,
                                resolver_0,
                                out_field,
                            );
                        }
                        #[allow(unreachable_patterns)]
                        _ => unsafe { ::core::hint::unreachable_unchecked() }
                    }
                }
                PartialFieldBlobChunkFillResolver::Corrupted(resolver_0) => {
                    match __this {
                        PartialFieldBlobChunkFill::Corrupted(self_0, ..) => {
                            let out = unsafe {
                                out.cast_unchecked::<ArchivedVariantCorrupted>()
                            };
                            let tag_ptr = unsafe { &raw mut (*out.ptr()).0 };
                            unsafe {
                                tag_ptr.write(ArchivedTag::Corrupted);
                            }
                            let field_ptr = unsafe { &raw mut (*out.ptr()).1 };
                            let out_field = unsafe {
                                ::rkyv::Place::from_field_unchecked(out, field_ptr)
                            };
                            <usize as ::rkyv::Archive>::resolve(
                                self_0,
                                resolver_0,
                                out_field,
                            );
                        }
                        #[allow(unreachable_patterns)]
                        _ => unsafe { ::core::hint::unreachable_unchecked() }
                    }
                }
            }
        }
    }
};
unsafe impl ::rkyv::traits::Portable for ArchivedPartialFieldBlobChunkFill
where
    usize: ::rkyv::Archive,
    usize: ::rkyv::Archive,
    usize: ::rkyv::Archive,
    <usize as ::rkyv::Archive>::Archived: ::rkyv::traits::Portable,
    <usize as ::rkyv::Archive>::Archived: ::rkyv::traits::Portable,
    <usize as ::rkyv::Archive>::Archived: ::rkyv::traits::Portable,
{}
#[automatically_derived]
impl<__S: ::rkyv::rancor::Fallible + ?Sized> ::rkyv::Serialize<__S>
for PartialFieldBlobChunkFill
where
    usize: ::rkyv::Serialize<__S>,
    usize: ::rkyv::Serialize<__S>,
    usize: ::rkyv::Serialize<__S>,
{
    fn serialize(
        &self,
        serializer: &mut __S,
    ) -> ::core::result::Result<
        <Self as ::rkyv::Archive>::Resolver,
        <__S as ::rkyv::rancor::Fallible>::Error,
    > {
        let __this = self;
        ::core::result::Result::Ok(
            match __this {
                PartialFieldBlobChunkFill::Full(_0, ..) => {
                    PartialFieldBlobChunkFillResolver::Full(
                        <usize as ::rkyv::Serialize<__S>>::serialize(_0, serializer)?,
                    )
                }
                PartialFieldBlobChunkFill::Partial(_0, ..) => {
                    PartialFieldBlobChunkFillResolver::Partial(
                        <usize as ::rkyv::Serialize<__S>>::serialize(_0, serializer)?,
                    )
                }
                PartialFieldBlobChunkFill::Corrupted(_0, ..) => {
                    PartialFieldBlobChunkFillResolver::Corrupted(
                        <usize as ::rkyv::Serialize<__S>>::serialize(_0, serializer)?,
                    )
                }
            },
        )
    }
}
#[automatically_derived]
impl<
    __D: ::rkyv::rancor::Fallible + ?Sized,
> ::rkyv::Deserialize<PartialFieldBlobChunkFill, __D>
for ::rkyv::Archived<PartialFieldBlobChunkFill>
where
    usize: ::rkyv::Archive,
    <usize as ::rkyv::Archive>::Archived: ::rkyv::Deserialize<usize, __D>,
    usize: ::rkyv::Archive,
    <usize as ::rkyv::Archive>::Archived: ::rkyv::Deserialize<usize, __D>,
    usize: ::rkyv::Archive,
    <usize as ::rkyv::Archive>::Archived: ::rkyv::Deserialize<usize, __D>,
{
    fn deserialize(
        &self,
        deserializer: &mut __D,
    ) -> ::core::result::Result<
        PartialFieldBlobChunkFill,
        <__D as ::rkyv::rancor::Fallible>::Error,
    > {
        let __this = self;
        ::core::result::Result::Ok(
            match __this {
                Self::Full(_0, ..) => {
                    PartialFieldBlobChunkFill::Full(
                        <<usize as ::rkyv::Archive>::Archived as ::rkyv::Deserialize<
                            usize,
                            __D,
                        >>::deserialize(_0, deserializer)?,
                    )
                }
                Self::Partial(_0, ..) => {
                    PartialFieldBlobChunkFill::Partial(
                        <<usize as ::rkyv::Archive>::Archived as ::rkyv::Deserialize<
                            usize,
                            __D,
                        >>::deserialize(_0, deserializer)?,
                    )
                }
                Self::Corrupted(_0, ..) => {
                    PartialFieldBlobChunkFill::Corrupted(
                        <<usize as ::rkyv::Archive>::Archived as ::rkyv::Deserialize<
                            usize,
                            __D,
                        >>::deserialize(_0, deserializer)?,
                    )
                }
            },
        )
    }
}
impl PartialFieldBlobChunkFill {
    pub fn from_size(actual: usize, expected: usize) -> Self {
        if actual == expected {
            Self::Full(actual)
        } else if actual < expected {
            Self::Partial(actual)
        } else {
            Self::Corrupted(actual)
        }
    }
}
impl rewrite::traits::structural::blob::BlobItemChunk for PartialFieldBlobChunk {
    type Index = usize;
    fn get_index(&self) -> &Self::Index {
        match self {
            Self::Header(c) => c.get_index(),
            Self::Payload(c) => c.get_index(),
            _ => {
                ::core::panicking::panic_fmt(
                    format_args!("get_index called on missing chunk"),
                );
            }
        }
    }
}
impl rewrite::traits::structural::blob::NetabaseBlobItem for PartialFieldBlob {
    type Chunk = PartialFieldBlobChunk;
    type BlobIter = std::vec::IntoIter<rewrite::results::NetabaseResult<Self::Chunk>>;
    const DEFAULT_CHUNK_SIZE: usize = 0usize;
    fn into_chunks(
        self,
        size: rewrite::traits::structural::blob::ChunkSize,
    ) -> Box<dyn Iterator<Item = Self::Chunk>> {
        Box::new(self.into_chunks_iter(size).filter_map(|r| r.ok()))
    }
    fn into_chunks_iter(
        self,
        size: rewrite::traits::structural::blob::ChunkSize,
    ) -> Self::BlobIter {
        let mut all_chunks = Vec::new();
        {
            let serialized_field: rewrite::results::NetabaseResult<Vec<u8>> = rkyv::to_bytes::<
                rkyv::rancor::Error,
            >(&self.header)
                .map_err(|e| rewrite::results::NetabaseError::Serialization(
                    ::alloc::__export::must_use({
                        ::alloc::fmt::format(
                            format_args!(
                                "rkyv serialization failed for field {0}: {1:?}",
                                "header",
                                e,
                            ),
                        )
                    }),
                ))
                .map(|d| d.to_vec());
            match serialized_field {
                Ok(data) => {
                    let chunk_size = match size {
                        rewrite::traits::structural::blob::ChunkSize::Default => {
                            let default = 64usize;
                            if default > 0 { default } else { 1024 }
                        }
                        rewrite::traits::structural::blob::ChunkSize::Size(n) => n,
                    };
                    all_chunks
                        .extend(
                            data
                                .chunks(chunk_size)
                                .enumerate()
                                .map(|(index, chunk_data)| {
                                    Ok(
                                        Self::Chunk::Header(PartialFieldBlobHeaderChunk {
                                            index,
                                            data: chunk_data.to_vec(),
                                        }),
                                    )
                                }),
                        );
                }
                Err(e) => {
                    all_chunks.push(Err(e));
                }
            }
        }
        {
            let serialized_field: rewrite::results::NetabaseResult<Vec<u8>> = rkyv::to_bytes::<
                rkyv::rancor::Error,
            >(&self.payload)
                .map_err(|e| rewrite::results::NetabaseError::Serialization(
                    ::alloc::__export::must_use({
                        ::alloc::fmt::format(
                            format_args!(
                                "rkyv serialization failed for field {0}: {1:?}",
                                "payload",
                                e,
                            ),
                        )
                    }),
                ))
                .map(|d| d.to_vec());
            match serialized_field {
                Ok(data) => {
                    let chunk_size = match size {
                        rewrite::traits::structural::blob::ChunkSize::Default => {
                            let default = 256usize;
                            if default > 0 { default } else { 1024 }
                        }
                        rewrite::traits::structural::blob::ChunkSize::Size(n) => n,
                    };
                    all_chunks
                        .extend(
                            data
                                .chunks(chunk_size)
                                .enumerate()
                                .map(|(index, chunk_data)| {
                                    Ok(
                                        Self::Chunk::Payload(PartialFieldBlobPayloadChunk {
                                            index,
                                            data: chunk_data.to_vec(),
                                        }),
                                    )
                                }),
                        );
                }
                Err(e) => {
                    all_chunks.push(Err(e));
                }
            }
        }
        all_chunks.into_iter()
    }
    fn try_from_chunks(
        chunks: impl Iterator<Item = Self::Chunk>,
        size: rewrite::traits::structural::blob::ChunkSize,
    ) -> rewrite::results::NetabaseResult<Self> {
        let mut chunks_header = Vec::new();
        let mut chunks_payload = Vec::new();
        for chunk in chunks {
            match chunk {
                Self::Chunk::Header(c) => chunks_header.push(c),
                Self::Chunk::Payload(c) => chunks_payload.push(c),
                _ => {}
            }
        }
        let header = {
            if chunks_header.is_empty() {
                return Err(
                    rewrite::results::NetabaseError::BlobReconstruction(
                        rewrite::results::BlobReconstructionError::MissingChunks,
                    ),
                );
            }
            let mut sorted = chunks_header;
            sorted.sort_by_key(|c| c.index);
            let chunk_size = match size {
                rewrite::traits::structural::blob::ChunkSize::Default => {
                    let default = 64usize;
                    if default > 0 { default } else { 1024 }
                }
                rewrite::traits::structural::blob::ChunkSize::Size(n) => n,
            };
            let mut missing_details = Vec::new();
            let mut next_expected = 0;
            let max_idx = sorted.last().map(|c| c.index).unwrap_or(0);
            for chunk in &sorted {
                while chunk.index > next_expected {
                    missing_details
                        .push(
                            ::alloc::__export::must_use({
                                ::alloc::fmt::format(
                                    format_args!(
                                        "{0:?}({{ Index: {1}, Size: {2} }})",
                                        PartialFieldBlobChunkFill::Full(chunk_size),
                                        next_expected,
                                        chunk_size,
                                    ),
                                )
                            }),
                        );
                    next_expected += 1;
                }
                let fill = PartialFieldBlobChunkFill::from_size(
                    chunk.data.len(),
                    chunk_size,
                );
                match fill {
                    PartialFieldBlobChunkFill::Corrupted(size) => {
                        return Err(
                            rewrite::results::NetabaseError::BlobReconstruction(
                                rewrite::results::BlobReconstructionError::InvalidChunkData(
                                    ::alloc::__export::must_use({
                                        ::alloc::fmt::format(
                                            format_args!(
                                                "Corrupted chunk detected for field {0}: {1:?}({{ Index: {2}, Size: {3} }}). Max allowed size is {4}.",
                                                "header",
                                                fill,
                                                chunk.index,
                                                size,
                                                chunk_size,
                                            ),
                                        )
                                    }),
                                ),
                            ),
                        );
                    }
                    PartialFieldBlobChunkFill::Partial(
                        size,
                    ) if chunk.index < max_idx => {
                        return Err(
                            rewrite::results::NetabaseError::BlobReconstruction(
                                rewrite::results::BlobReconstructionError::InvalidChunkData(
                                    ::alloc::__export::must_use({
                                        ::alloc::fmt::format(
                                            format_args!(
                                                "Unexpected partial chunk in middle of stream for field {0}: {1:?}({{ Index: {2}, Size: {3} }}). Expected {4} bytes.",
                                                "header",
                                                fill,
                                                chunk.index,
                                                size,
                                                chunk_size,
                                            ),
                                        )
                                    }),
                                ),
                            ),
                        );
                    }
                    _ => {}
                }
                if chunk.index == next_expected {
                    next_expected += 1;
                }
            }
            if !missing_details.is_empty() {
                if let Some(last) = sorted.last() {
                    let fill = PartialFieldBlobChunkFill::from_size(
                        last.data.len(),
                        chunk_size,
                    );
                    if #[allow(non_exhaustive_omitted_patterns)]
                    match fill {
                        PartialFieldBlobChunkFill::Full(_) => true,
                        _ => false,
                    } {
                        missing_details
                            .push(
                                ::alloc::__export::must_use({
                                    ::alloc::fmt::format(
                                        format_args!(
                                            "... (Stream truncated for field {0}: last chunk was Full, expected more data after Index {1})",
                                            "header",
                                            last.index,
                                        ),
                                    )
                                }),
                            );
                    }
                }
            }
            if !missing_details.is_empty() {
                return Err(
                    rewrite::results::NetabaseError::BlobReconstruction(
                        rewrite::results::BlobReconstructionError::InvalidChunkData(
                            ::alloc::__export::must_use({
                                ::alloc::fmt::format(
                                    format_args!(
                                        "Missing chunks for field {0}: [{1}]. Total chunks present: {2}",
                                        "header",
                                        missing_details.join(", "),
                                        sorted.len(),
                                    ),
                                )
                            }),
                        ),
                    ),
                );
            }
            let data: Vec<u8> = sorted.into_iter().flat_map(|c| c.data).collect();
            rkyv::from_bytes::<String, rkyv::rancor::Error>(&data)
                .map_err(|e| rewrite::results::NetabaseError::Serialization(
                    ::alloc::__export::must_use({
                        ::alloc::fmt::format(
                            format_args!(
                                "rkyv deserialization failed for field {0}: {1:?}",
                                "header",
                                e,
                            ),
                        )
                    }),
                ))?
        };
        let payload = {
            if chunks_payload.is_empty() {
                return Err(
                    rewrite::results::NetabaseError::BlobReconstruction(
                        rewrite::results::BlobReconstructionError::MissingChunks,
                    ),
                );
            }
            let mut sorted = chunks_payload;
            sorted.sort_by_key(|c| c.index);
            let chunk_size = match size {
                rewrite::traits::structural::blob::ChunkSize::Default => {
                    let default = 256usize;
                    if default > 0 { default } else { 1024 }
                }
                rewrite::traits::structural::blob::ChunkSize::Size(n) => n,
            };
            let mut missing_details = Vec::new();
            let mut next_expected = 0;
            let max_idx = sorted.last().map(|c| c.index).unwrap_or(0);
            for chunk in &sorted {
                while chunk.index > next_expected {
                    missing_details
                        .push(
                            ::alloc::__export::must_use({
                                ::alloc::fmt::format(
                                    format_args!(
                                        "{0:?}({{ Index: {1}, Size: {2} }})",
                                        PartialFieldBlobChunkFill::Full(chunk_size),
                                        next_expected,
                                        chunk_size,
                                    ),
                                )
                            }),
                        );
                    next_expected += 1;
                }
                let fill = PartialFieldBlobChunkFill::from_size(
                    chunk.data.len(),
                    chunk_size,
                );
                match fill {
                    PartialFieldBlobChunkFill::Corrupted(size) => {
                        return Err(
                            rewrite::results::NetabaseError::BlobReconstruction(
                                rewrite::results::BlobReconstructionError::InvalidChunkData(
                                    ::alloc::__export::must_use({
                                        ::alloc::fmt::format(
                                            format_args!(
                                                "Corrupted chunk detected for field {0}: {1:?}({{ Index: {2}, Size: {3} }}). Max allowed size is {4}.",
                                                "payload",
                                                fill,
                                                chunk.index,
                                                size,
                                                chunk_size,
                                            ),
                                        )
                                    }),
                                ),
                            ),
                        );
                    }
                    PartialFieldBlobChunkFill::Partial(
                        size,
                    ) if chunk.index < max_idx => {
                        return Err(
                            rewrite::results::NetabaseError::BlobReconstruction(
                                rewrite::results::BlobReconstructionError::InvalidChunkData(
                                    ::alloc::__export::must_use({
                                        ::alloc::fmt::format(
                                            format_args!(
                                                "Unexpected partial chunk in middle of stream for field {0}: {1:?}({{ Index: {2}, Size: {3} }}). Expected {4} bytes.",
                                                "payload",
                                                fill,
                                                chunk.index,
                                                size,
                                                chunk_size,
                                            ),
                                        )
                                    }),
                                ),
                            ),
                        );
                    }
                    _ => {}
                }
                if chunk.index == next_expected {
                    next_expected += 1;
                }
            }
            if !missing_details.is_empty() {
                if let Some(last) = sorted.last() {
                    let fill = PartialFieldBlobChunkFill::from_size(
                        last.data.len(),
                        chunk_size,
                    );
                    if #[allow(non_exhaustive_omitted_patterns)]
                    match fill {
                        PartialFieldBlobChunkFill::Full(_) => true,
                        _ => false,
                    } {
                        missing_details
                            .push(
                                ::alloc::__export::must_use({
                                    ::alloc::fmt::format(
                                        format_args!(
                                            "... (Stream truncated for field {0}: last chunk was Full, expected more data after Index {1})",
                                            "payload",
                                            last.index,
                                        ),
                                    )
                                }),
                            );
                    }
                }
            }
            if !missing_details.is_empty() {
                return Err(
                    rewrite::results::NetabaseError::BlobReconstruction(
                        rewrite::results::BlobReconstructionError::InvalidChunkData(
                            ::alloc::__export::must_use({
                                ::alloc::fmt::format(
                                    format_args!(
                                        "Missing chunks for field {0}: [{1}]. Total chunks present: {2}",
                                        "payload",
                                        missing_details.join(", "),
                                        sorted.len(),
                                    ),
                                )
                            }),
                        ),
                    ),
                );
            }
            let data: Vec<u8> = sorted.into_iter().flat_map(|c| c.data).collect();
            rkyv::from_bytes::<Vec<u8>, rkyv::rancor::Error>(&data)
                .map_err(|e| rewrite::results::NetabaseError::Serialization(
                    ::alloc::__export::must_use({
                        ::alloc::fmt::format(
                            format_args!(
                                "rkyv deserialization failed for field {0}: {1:?}",
                                "payload",
                                e,
                            ),
                        )
                    }),
                ))?
        };
        Ok(Self { header, payload })
    }
    fn get_blob(&self) -> &Self::Chunk {
        {
            ::core::panicking::panic_fmt(
                format_args!(
                    "not implemented: {0}",
                    format_args!("get_blob() requires storing a chunk reference"),
                ),
            );
        }
    }
}
impl IntoIterator for PartialFieldBlob {
    type Item = rewrite::results::NetabaseResult<PartialFieldBlobChunk>;
    type IntoIter = std::vec::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        rewrite::traits::structural::blob::NetabaseBlobItem::into_chunks_iter(
            self,
            rewrite::traits::structural::blob::ChunkSize::Default,
        )
    }
}
struct NestedBlob {
    id: u32,
}
#[automatically_derived]
///An archived [`NestedBlob`]
#[bytecheck(crate = ::rkyv::bytecheck)]
#[repr(C)]
struct ArchivedNestedBlob
where
    u32: ::rkyv::Archive,
{
    ///The archived counterpart of [`NestedBlob::id`]
    id: <u32 as ::rkyv::Archive>::Archived,
}
#[automatically_derived]
unsafe impl<
    __C: ::rkyv::bytecheck::rancor::Fallible + ?::core::marker::Sized,
> ::rkyv::bytecheck::CheckBytes<__C> for ArchivedNestedBlob
where
    u32: ::rkyv::Archive,
    <__C as ::rkyv::bytecheck::rancor::Fallible>::Error: ::rkyv::bytecheck::rancor::Trace,
    <u32 as ::rkyv::Archive>::Archived: ::rkyv::bytecheck::CheckBytes<__C>,
{
    unsafe fn check_bytes(
        value: *const Self,
        context: &mut __C,
    ) -> ::core::result::Result<
        (),
        <__C as ::rkyv::bytecheck::rancor::Fallible>::Error,
    > {
        <<u32 as ::rkyv::Archive>::Archived as ::rkyv::bytecheck::CheckBytes<
            __C,
        >>::check_bytes(&raw const (*value).id, context)
            .map_err(|e| {
                <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Trace>::trace(
                    e,
                    ::rkyv::bytecheck::StructCheckContext {
                        struct_name: "ArchivedNestedBlob",
                        field_name: "id",
                    },
                )
            })?;
        ::core::result::Result::Ok(())
    }
}
#[automatically_derived]
///The resolver for an archived [`NestedBlob`]
struct NestedBlobResolver
where
    u32: ::rkyv::Archive,
{
    id: <u32 as ::rkyv::Archive>::Resolver,
}
impl ::rkyv::Archive for NestedBlob
where
    u32: ::rkyv::Archive,
{
    type Archived = ArchivedNestedBlob;
    type Resolver = NestedBlobResolver;
    const COPY_OPTIMIZATION: ::rkyv::traits::CopyOptimization<Self> = unsafe {
        ::rkyv::traits::CopyOptimization::enable_if(
            0 + ::core::mem::size_of::<u32>() == ::core::mem::size_of::<NestedBlob>()
                && <u32 as ::rkyv::Archive>::COPY_OPTIMIZATION.is_enabled()
                && const { builtin # offset_of(NestedBlob, id) }
                    == const { builtin # offset_of(ArchivedNestedBlob, id) },
        )
    };
    #[allow(clippy::unit_arg)]
    fn resolve(&self, resolver: Self::Resolver, out: ::rkyv::Place<Self::Archived>) {
        let field_ptr = unsafe { &raw mut (*out.ptr()).id };
        let field_out = unsafe { ::rkyv::Place::from_field_unchecked(out, field_ptr) };
        <u32 as ::rkyv::Archive>::resolve(&self.id, resolver.id, field_out);
    }
}
unsafe impl ::rkyv::traits::Portable for ArchivedNestedBlob
where
    u32: ::rkyv::Archive,
    <u32 as ::rkyv::Archive>::Archived: ::rkyv::traits::Portable,
{}
#[automatically_derived]
impl<__S: ::rkyv::rancor::Fallible + ?Sized> ::rkyv::Serialize<__S> for NestedBlob
where
    u32: ::rkyv::Serialize<__S>,
{
    fn serialize(
        &self,
        serializer: &mut __S,
    ) -> ::core::result::Result<
        <Self as ::rkyv::Archive>::Resolver,
        <__S as ::rkyv::rancor::Fallible>::Error,
    > {
        let __this = self;
        ::core::result::Result::Ok(NestedBlobResolver {
            id: <u32 as ::rkyv::Serialize<__S>>::serialize(&__this.id, serializer)?,
        })
    }
}
#[automatically_derived]
impl<__D: ::rkyv::rancor::Fallible + ?Sized> ::rkyv::Deserialize<NestedBlob, __D>
for ::rkyv::Archived<NestedBlob>
where
    u32: ::rkyv::Archive,
    <u32 as ::rkyv::Archive>::Archived: ::rkyv::Deserialize<u32, __D>,
{
    fn deserialize(
        &self,
        deserializer: &mut __D,
    ) -> ::core::result::Result<NestedBlob, <__D as ::rkyv::rancor::Fallible>::Error> {
        let __this = self;
        ::core::result::Result::Ok(NestedBlob {
            id: <<u32 as ::rkyv::Archive>::Archived as ::rkyv::Deserialize<
                u32,
                __D,
            >>::deserialize(&__this.id, deserializer)?,
        })
    }
}
#[automatically_derived]
impl ::core::fmt::Debug for NestedBlob {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::debug_struct_field1_finish(
            f,
            "NestedBlob",
            "id",
            &&self.id,
        )
    }
}
#[automatically_derived]
impl ::core::clone::Clone for NestedBlob {
    #[inline]
    fn clone(&self) -> NestedBlob {
        NestedBlob {
            id: ::core::clone::Clone::clone(&self.id),
        }
    }
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for NestedBlob {}
#[automatically_derived]
impl ::core::cmp::PartialEq for NestedBlob {
    #[inline]
    fn eq(&self, other: &NestedBlob) -> bool {
        self.id == other.id
    }
}
#[automatically_derived]
impl ::core::cmp::Eq for NestedBlob {
    #[inline]
    #[doc(hidden)]
    #[coverage(off)]
    fn assert_fields_are_eq(&self) {
        let _: ::core::cmp::AssertParamIsEq<u32>;
    }
}
pub struct NestedBlobChunk {
    pub index: usize,
    pub data: Vec<u8>,
}
#[automatically_derived]
impl ::core::fmt::Debug for NestedBlobChunk {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::debug_struct_field2_finish(
            f,
            "NestedBlobChunk",
            "index",
            &self.index,
            "data",
            &&self.data,
        )
    }
}
#[automatically_derived]
impl ::core::clone::Clone for NestedBlobChunk {
    #[inline]
    fn clone(&self) -> NestedBlobChunk {
        NestedBlobChunk {
            index: ::core::clone::Clone::clone(&self.index),
            data: ::core::clone::Clone::clone(&self.data),
        }
    }
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for NestedBlobChunk {}
#[automatically_derived]
impl ::core::cmp::PartialEq for NestedBlobChunk {
    #[inline]
    fn eq(&self, other: &NestedBlobChunk) -> bool {
        self.index == other.index && self.data == other.data
    }
}
#[automatically_derived]
impl ::core::cmp::Eq for NestedBlobChunk {
    #[inline]
    #[doc(hidden)]
    #[coverage(off)]
    fn assert_fields_are_eq(&self) {
        let _: ::core::cmp::AssertParamIsEq<usize>;
        let _: ::core::cmp::AssertParamIsEq<Vec<u8>>;
    }
}
#[automatically_derived]
impl ::core::cmp::PartialOrd for NestedBlobChunk {
    #[inline]
    fn partial_cmp(
        &self,
        other: &NestedBlobChunk,
    ) -> ::core::option::Option<::core::cmp::Ordering> {
        match ::core::cmp::PartialOrd::partial_cmp(&self.index, &other.index) {
            ::core::option::Option::Some(::core::cmp::Ordering::Equal) => {
                ::core::cmp::PartialOrd::partial_cmp(&self.data, &other.data)
            }
            cmp => cmp,
        }
    }
}
#[automatically_derived]
impl ::core::cmp::Ord for NestedBlobChunk {
    #[inline]
    fn cmp(&self, other: &NestedBlobChunk) -> ::core::cmp::Ordering {
        match ::core::cmp::Ord::cmp(&self.index, &other.index) {
            ::core::cmp::Ordering::Equal => {
                ::core::cmp::Ord::cmp(&self.data, &other.data)
            }
            cmp => cmp,
        }
    }
}
#[automatically_derived]
///An archived [`NestedBlobChunk`]
#[bytecheck(crate = ::rkyv::bytecheck)]
#[repr(C)]
pub struct ArchivedNestedBlobChunk
where
    usize: ::rkyv::Archive,
    Vec<u8>: ::rkyv::Archive,
{
    ///The archived counterpart of [`NestedBlobChunk::index`]
    pub index: <usize as ::rkyv::Archive>::Archived,
    ///The archived counterpart of [`NestedBlobChunk::data`]
    pub data: <Vec<u8> as ::rkyv::Archive>::Archived,
}
#[automatically_derived]
unsafe impl<
    __C: ::rkyv::bytecheck::rancor::Fallible + ?::core::marker::Sized,
> ::rkyv::bytecheck::CheckBytes<__C> for ArchivedNestedBlobChunk
where
    usize: ::rkyv::Archive,
    Vec<u8>: ::rkyv::Archive,
    <__C as ::rkyv::bytecheck::rancor::Fallible>::Error: ::rkyv::bytecheck::rancor::Trace,
    <usize as ::rkyv::Archive>::Archived: ::rkyv::bytecheck::CheckBytes<__C>,
    <Vec<u8> as ::rkyv::Archive>::Archived: ::rkyv::bytecheck::CheckBytes<__C>,
{
    unsafe fn check_bytes(
        value: *const Self,
        context: &mut __C,
    ) -> ::core::result::Result<
        (),
        <__C as ::rkyv::bytecheck::rancor::Fallible>::Error,
    > {
        <<usize as ::rkyv::Archive>::Archived as ::rkyv::bytecheck::CheckBytes<
            __C,
        >>::check_bytes(&raw const (*value).index, context)
            .map_err(|e| {
                <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Trace>::trace(
                    e,
                    ::rkyv::bytecheck::StructCheckContext {
                        struct_name: "ArchivedNestedBlobChunk",
                        field_name: "index",
                    },
                )
            })?;
        <<Vec<
            u8,
        > as ::rkyv::Archive>::Archived as ::rkyv::bytecheck::CheckBytes<
            __C,
        >>::check_bytes(&raw const (*value).data, context)
            .map_err(|e| {
                <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Trace>::trace(
                    e,
                    ::rkyv::bytecheck::StructCheckContext {
                        struct_name: "ArchivedNestedBlobChunk",
                        field_name: "data",
                    },
                )
            })?;
        ::core::result::Result::Ok(())
    }
}
#[automatically_derived]
///The resolver for an archived [`NestedBlobChunk`]
pub struct NestedBlobChunkResolver
where
    usize: ::rkyv::Archive,
    Vec<u8>: ::rkyv::Archive,
{
    index: <usize as ::rkyv::Archive>::Resolver,
    data: <Vec<u8> as ::rkyv::Archive>::Resolver,
}
impl ::rkyv::Archive for NestedBlobChunk
where
    usize: ::rkyv::Archive,
    Vec<u8>: ::rkyv::Archive,
{
    type Archived = ArchivedNestedBlobChunk;
    type Resolver = NestedBlobChunkResolver;
    const COPY_OPTIMIZATION: ::rkyv::traits::CopyOptimization<Self> = unsafe {
        ::rkyv::traits::CopyOptimization::enable_if(
            0 + ::core::mem::size_of::<usize>() + ::core::mem::size_of::<Vec<u8>>()
                == ::core::mem::size_of::<NestedBlobChunk>()
                && <usize as ::rkyv::Archive>::COPY_OPTIMIZATION.is_enabled()
                && const { builtin # offset_of(NestedBlobChunk, index) }
                    == const { builtin # offset_of(ArchivedNestedBlobChunk, index) }
                && <Vec<u8> as ::rkyv::Archive>::COPY_OPTIMIZATION.is_enabled()
                && const { builtin # offset_of(NestedBlobChunk, data) }
                    == const { builtin # offset_of(ArchivedNestedBlobChunk, data) },
        )
    };
    #[allow(clippy::unit_arg)]
    fn resolve(&self, resolver: Self::Resolver, out: ::rkyv::Place<Self::Archived>) {
        let field_ptr = unsafe { &raw mut (*out.ptr()).index };
        let field_out = unsafe { ::rkyv::Place::from_field_unchecked(out, field_ptr) };
        <usize as ::rkyv::Archive>::resolve(&self.index, resolver.index, field_out);
        let field_ptr = unsafe { &raw mut (*out.ptr()).data };
        let field_out = unsafe { ::rkyv::Place::from_field_unchecked(out, field_ptr) };
        <Vec<u8> as ::rkyv::Archive>::resolve(&self.data, resolver.data, field_out);
    }
}
unsafe impl ::rkyv::traits::Portable for ArchivedNestedBlobChunk
where
    usize: ::rkyv::Archive,
    Vec<u8>: ::rkyv::Archive,
    <usize as ::rkyv::Archive>::Archived: ::rkyv::traits::Portable,
    <Vec<u8> as ::rkyv::Archive>::Archived: ::rkyv::traits::Portable,
{}
#[automatically_derived]
impl<__S: ::rkyv::rancor::Fallible + ?Sized> ::rkyv::Serialize<__S> for NestedBlobChunk
where
    usize: ::rkyv::Serialize<__S>,
    Vec<u8>: ::rkyv::Serialize<__S>,
{
    fn serialize(
        &self,
        serializer: &mut __S,
    ) -> ::core::result::Result<
        <Self as ::rkyv::Archive>::Resolver,
        <__S as ::rkyv::rancor::Fallible>::Error,
    > {
        let __this = self;
        ::core::result::Result::Ok(NestedBlobChunkResolver {
            index: <usize as ::rkyv::Serialize<
                __S,
            >>::serialize(&__this.index, serializer)?,
            data: <Vec<
                u8,
            > as ::rkyv::Serialize<__S>>::serialize(&__this.data, serializer)?,
        })
    }
}
#[automatically_derived]
impl<__D: ::rkyv::rancor::Fallible + ?Sized> ::rkyv::Deserialize<NestedBlobChunk, __D>
for ::rkyv::Archived<NestedBlobChunk>
where
    usize: ::rkyv::Archive,
    <usize as ::rkyv::Archive>::Archived: ::rkyv::Deserialize<usize, __D>,
    Vec<u8>: ::rkyv::Archive,
    <Vec<u8> as ::rkyv::Archive>::Archived: ::rkyv::Deserialize<Vec<u8>, __D>,
{
    fn deserialize(
        &self,
        deserializer: &mut __D,
    ) -> ::core::result::Result<
        NestedBlobChunk,
        <__D as ::rkyv::rancor::Fallible>::Error,
    > {
        let __this = self;
        ::core::result::Result::Ok(NestedBlobChunk {
            index: <<usize as ::rkyv::Archive>::Archived as ::rkyv::Deserialize<
                usize,
                __D,
            >>::deserialize(&__this.index, deserializer)?,
            data: <<Vec<
                u8,
            > as ::rkyv::Archive>::Archived as ::rkyv::Deserialize<
                Vec<u8>,
                __D,
            >>::deserialize(&__this.data, deserializer)?,
        })
    }
}
impl ::rewrite::traits::structural::blob::BlobItemChunk for NestedBlobChunk {
    type Index = usize;
    fn get_index(&self) -> &Self::Index {
        &self.index
    }
}
pub enum NestedBlobChunkFill {
    Full(usize),
    Partial(usize),
    Corrupted(usize),
}
#[automatically_derived]
impl ::core::fmt::Debug for NestedBlobChunkFill {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match self {
            NestedBlobChunkFill::Full(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "Full", &__self_0)
            }
            NestedBlobChunkFill::Partial(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(
                    f,
                    "Partial",
                    &__self_0,
                )
            }
            NestedBlobChunkFill::Corrupted(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(
                    f,
                    "Corrupted",
                    &__self_0,
                )
            }
        }
    }
}
#[automatically_derived]
#[doc(hidden)]
unsafe impl ::core::clone::TrivialClone for NestedBlobChunkFill {}
#[automatically_derived]
impl ::core::clone::Clone for NestedBlobChunkFill {
    #[inline]
    fn clone(&self) -> NestedBlobChunkFill {
        let _: ::core::clone::AssertParamIsClone<usize>;
        *self
    }
}
#[automatically_derived]
impl ::core::marker::Copy for NestedBlobChunkFill {}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for NestedBlobChunkFill {}
#[automatically_derived]
impl ::core::cmp::PartialEq for NestedBlobChunkFill {
    #[inline]
    fn eq(&self, other: &NestedBlobChunkFill) -> bool {
        let __self_discr = ::core::intrinsics::discriminant_value(self);
        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
        __self_discr == __arg1_discr
            && match (self, other) {
                (
                    NestedBlobChunkFill::Full(__self_0),
                    NestedBlobChunkFill::Full(__arg1_0),
                ) => __self_0 == __arg1_0,
                (
                    NestedBlobChunkFill::Partial(__self_0),
                    NestedBlobChunkFill::Partial(__arg1_0),
                ) => __self_0 == __arg1_0,
                (
                    NestedBlobChunkFill::Corrupted(__self_0),
                    NestedBlobChunkFill::Corrupted(__arg1_0),
                ) => __self_0 == __arg1_0,
                _ => unsafe { ::core::intrinsics::unreachable() }
            }
    }
}
#[automatically_derived]
impl ::core::cmp::Eq for NestedBlobChunkFill {
    #[inline]
    #[doc(hidden)]
    #[coverage(off)]
    fn assert_fields_are_eq(&self) {
        let _: ::core::cmp::AssertParamIsEq<usize>;
    }
}
#[automatically_derived]
impl ::core::cmp::PartialOrd for NestedBlobChunkFill {
    #[inline]
    fn partial_cmp(
        &self,
        other: &NestedBlobChunkFill,
    ) -> ::core::option::Option<::core::cmp::Ordering> {
        let __self_discr = ::core::intrinsics::discriminant_value(self);
        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
        match (self, other) {
            (
                NestedBlobChunkFill::Full(__self_0),
                NestedBlobChunkFill::Full(__arg1_0),
            ) => ::core::cmp::PartialOrd::partial_cmp(__self_0, __arg1_0),
            (
                NestedBlobChunkFill::Partial(__self_0),
                NestedBlobChunkFill::Partial(__arg1_0),
            ) => ::core::cmp::PartialOrd::partial_cmp(__self_0, __arg1_0),
            (
                NestedBlobChunkFill::Corrupted(__self_0),
                NestedBlobChunkFill::Corrupted(__arg1_0),
            ) => ::core::cmp::PartialOrd::partial_cmp(__self_0, __arg1_0),
            _ => ::core::cmp::PartialOrd::partial_cmp(&__self_discr, &__arg1_discr),
        }
    }
}
#[automatically_derived]
impl ::core::cmp::Ord for NestedBlobChunkFill {
    #[inline]
    fn cmp(&self, other: &NestedBlobChunkFill) -> ::core::cmp::Ordering {
        let __self_discr = ::core::intrinsics::discriminant_value(self);
        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
        match ::core::cmp::Ord::cmp(&__self_discr, &__arg1_discr) {
            ::core::cmp::Ordering::Equal => {
                match (self, other) {
                    (
                        NestedBlobChunkFill::Full(__self_0),
                        NestedBlobChunkFill::Full(__arg1_0),
                    ) => ::core::cmp::Ord::cmp(__self_0, __arg1_0),
                    (
                        NestedBlobChunkFill::Partial(__self_0),
                        NestedBlobChunkFill::Partial(__arg1_0),
                    ) => ::core::cmp::Ord::cmp(__self_0, __arg1_0),
                    (
                        NestedBlobChunkFill::Corrupted(__self_0),
                        NestedBlobChunkFill::Corrupted(__arg1_0),
                    ) => ::core::cmp::Ord::cmp(__self_0, __arg1_0),
                    _ => unsafe { ::core::intrinsics::unreachable() }
                }
            }
            cmp => cmp,
        }
    }
}
#[automatically_derived]
///An archived [`NestedBlobChunkFill`]
#[bytecheck(crate = ::rkyv::bytecheck)]
#[repr(u8)]
pub enum ArchivedNestedBlobChunkFill
where
    usize: ::rkyv::Archive,
    usize: ::rkyv::Archive,
    usize: ::rkyv::Archive,
{
    ///The archived counterpart of [`NestedBlobChunkFill::Full`]
    #[allow(dead_code)]
    Full(
        ///The archived counterpart of [`NestedBlobChunkFill::Full::0`]
        <usize as ::rkyv::Archive>::Archived,
    ),
    ///The archived counterpart of [`NestedBlobChunkFill::Partial`]
    #[allow(dead_code)]
    Partial(
        ///The archived counterpart of [`NestedBlobChunkFill::Partial::0`]
        <usize as ::rkyv::Archive>::Archived,
    ),
    ///The archived counterpart of [`NestedBlobChunkFill::Corrupted`]
    #[allow(dead_code)]
    Corrupted(
        ///The archived counterpart of [`NestedBlobChunkFill::Corrupted::0`]
        <usize as ::rkyv::Archive>::Archived,
    ),
}
const _: () = {
    #[repr(u8)]
    enum Tag {
        Full,
        Partial,
        Corrupted,
    }
    struct Discriminant;
    #[automatically_derived]
    impl Discriminant {
        #[allow(non_upper_case_globals)]
        const Full: u8 = Tag::Full as u8;
        #[allow(non_upper_case_globals)]
        const Partial: u8 = Tag::Partial as u8;
        #[allow(non_upper_case_globals)]
        const Corrupted: u8 = Tag::Corrupted as u8;
    }
    #[repr(C)]
    struct VariantFull(
        Tag,
        <usize as ::rkyv::Archive>::Archived,
        ::core::marker::PhantomData<ArchivedNestedBlobChunkFill>,
    )
    where
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive;
    #[repr(C)]
    struct VariantPartial(
        Tag,
        <usize as ::rkyv::Archive>::Archived,
        ::core::marker::PhantomData<ArchivedNestedBlobChunkFill>,
    )
    where
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive;
    #[repr(C)]
    struct VariantCorrupted(
        Tag,
        <usize as ::rkyv::Archive>::Archived,
        ::core::marker::PhantomData<ArchivedNestedBlobChunkFill>,
    )
    where
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive;
    #[automatically_derived]
    unsafe impl<
        __C: ::rkyv::bytecheck::rancor::Fallible + ?::core::marker::Sized,
    > ::rkyv::bytecheck::CheckBytes<__C> for ArchivedNestedBlobChunkFill
    where
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive,
        <__C as ::rkyv::bytecheck::rancor::Fallible>::Error: ::rkyv::bytecheck::rancor::Source,
        <usize as ::rkyv::Archive>::Archived: ::rkyv::bytecheck::CheckBytes<__C>,
        <usize as ::rkyv::Archive>::Archived: ::rkyv::bytecheck::CheckBytes<__C>,
        <usize as ::rkyv::Archive>::Archived: ::rkyv::bytecheck::CheckBytes<__C>,
    {
        unsafe fn check_bytes(
            value: *const Self,
            context: &mut __C,
        ) -> ::core::result::Result<
            (),
            <__C as ::rkyv::bytecheck::rancor::Fallible>::Error,
        > {
            let tag = *value.cast::<u8>();
            match tag {
                Discriminant::Full => {
                    let value = value.cast::<VariantFull>();
                    <<usize as ::rkyv::Archive>::Archived as ::rkyv::bytecheck::CheckBytes<
                        __C,
                    >>::check_bytes(&raw const (*value).1, context)
                        .map_err(|e| {
                            <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Trace>::trace(
                                e,
                                ::rkyv::bytecheck::UnnamedEnumVariantCheckContext {
                                    enum_name: "ArchivedNestedBlobChunkFill",
                                    variant_name: "Full",
                                    field_index: 1,
                                },
                            )
                        })?;
                }
                Discriminant::Partial => {
                    let value = value.cast::<VariantPartial>();
                    <<usize as ::rkyv::Archive>::Archived as ::rkyv::bytecheck::CheckBytes<
                        __C,
                    >>::check_bytes(&raw const (*value).1, context)
                        .map_err(|e| {
                            <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Trace>::trace(
                                e,
                                ::rkyv::bytecheck::UnnamedEnumVariantCheckContext {
                                    enum_name: "ArchivedNestedBlobChunkFill",
                                    variant_name: "Partial",
                                    field_index: 1,
                                },
                            )
                        })?;
                }
                Discriminant::Corrupted => {
                    let value = value.cast::<VariantCorrupted>();
                    <<usize as ::rkyv::Archive>::Archived as ::rkyv::bytecheck::CheckBytes<
                        __C,
                    >>::check_bytes(&raw const (*value).1, context)
                        .map_err(|e| {
                            <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Trace>::trace(
                                e,
                                ::rkyv::bytecheck::UnnamedEnumVariantCheckContext {
                                    enum_name: "ArchivedNestedBlobChunkFill",
                                    variant_name: "Corrupted",
                                    field_index: 1,
                                },
                            )
                        })?;
                }
                _ => {
                    return ::core::result::Result::Err(
                        <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Source>::new(::rkyv::bytecheck::InvalidEnumDiscriminantError {
                            enum_name: "ArchivedNestedBlobChunkFill",
                            invalid_discriminant: tag,
                        }),
                    );
                }
            }
            ::core::result::Result::Ok(())
        }
    }
};
#[automatically_derived]
///The resolver for an archived [`NestedBlobChunkFill`]
pub enum NestedBlobChunkFillResolver
where
    usize: ::rkyv::Archive,
    usize: ::rkyv::Archive,
    usize: ::rkyv::Archive,
{
    ///The resolver for [`NestedBlobChunkFill::Full`]
    #[allow(dead_code)]
    Full(<usize as ::rkyv::Archive>::Resolver),
    ///The resolver for [`NestedBlobChunkFill::Partial`]
    #[allow(dead_code)]
    Partial(<usize as ::rkyv::Archive>::Resolver),
    ///The resolver for [`NestedBlobChunkFill::Corrupted`]
    #[allow(dead_code)]
    Corrupted(<usize as ::rkyv::Archive>::Resolver),
}
const _: () = {
    #[repr(u8)]
    enum ArchivedTag {
        Full,
        Partial,
        Corrupted,
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for ArchivedTag {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for ArchivedTag {
        #[inline]
        fn eq(&self, other: &ArchivedTag) -> bool {
            let __self_discr = ::core::intrinsics::discriminant_value(self);
            let __arg1_discr = ::core::intrinsics::discriminant_value(other);
            __self_discr == __arg1_discr
        }
    }
    #[automatically_derived]
    impl ::core::cmp::PartialOrd for ArchivedTag {
        #[inline]
        fn partial_cmp(
            &self,
            other: &ArchivedTag,
        ) -> ::core::option::Option<::core::cmp::Ordering> {
            let __self_discr = ::core::intrinsics::discriminant_value(self);
            let __arg1_discr = ::core::intrinsics::discriminant_value(other);
            ::core::cmp::PartialOrd::partial_cmp(&__self_discr, &__arg1_discr)
        }
    }
    #[repr(C)]
    struct ArchivedVariantFull(
        ArchivedTag,
        <usize as ::rkyv::Archive>::Archived,
        ::core::marker::PhantomData<NestedBlobChunkFill>,
    )
    where
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive;
    #[repr(C)]
    struct ArchivedVariantPartial(
        ArchivedTag,
        <usize as ::rkyv::Archive>::Archived,
        ::core::marker::PhantomData<NestedBlobChunkFill>,
    )
    where
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive;
    #[repr(C)]
    struct ArchivedVariantCorrupted(
        ArchivedTag,
        <usize as ::rkyv::Archive>::Archived,
        ::core::marker::PhantomData<NestedBlobChunkFill>,
    )
    where
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive;
    impl ::rkyv::Archive for NestedBlobChunkFill
    where
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive,
    {
        type Archived = ArchivedNestedBlobChunkFill;
        type Resolver = NestedBlobChunkFillResolver;
        #[allow(clippy::unit_arg)]
        fn resolve(
            &self,
            resolver: <Self as ::rkyv::Archive>::Resolver,
            out: ::rkyv::Place<<Self as ::rkyv::Archive>::Archived>,
        ) {
            let __this = self;
            match resolver {
                NestedBlobChunkFillResolver::Full(resolver_0) => {
                    match __this {
                        NestedBlobChunkFill::Full(self_0, ..) => {
                            let out = unsafe {
                                out.cast_unchecked::<ArchivedVariantFull>()
                            };
                            let tag_ptr = unsafe { &raw mut (*out.ptr()).0 };
                            unsafe {
                                tag_ptr.write(ArchivedTag::Full);
                            }
                            let field_ptr = unsafe { &raw mut (*out.ptr()).1 };
                            let out_field = unsafe {
                                ::rkyv::Place::from_field_unchecked(out, field_ptr)
                            };
                            <usize as ::rkyv::Archive>::resolve(
                                self_0,
                                resolver_0,
                                out_field,
                            );
                        }
                        #[allow(unreachable_patterns)]
                        _ => unsafe { ::core::hint::unreachable_unchecked() }
                    }
                }
                NestedBlobChunkFillResolver::Partial(resolver_0) => {
                    match __this {
                        NestedBlobChunkFill::Partial(self_0, ..) => {
                            let out = unsafe {
                                out.cast_unchecked::<ArchivedVariantPartial>()
                            };
                            let tag_ptr = unsafe { &raw mut (*out.ptr()).0 };
                            unsafe {
                                tag_ptr.write(ArchivedTag::Partial);
                            }
                            let field_ptr = unsafe { &raw mut (*out.ptr()).1 };
                            let out_field = unsafe {
                                ::rkyv::Place::from_field_unchecked(out, field_ptr)
                            };
                            <usize as ::rkyv::Archive>::resolve(
                                self_0,
                                resolver_0,
                                out_field,
                            );
                        }
                        #[allow(unreachable_patterns)]
                        _ => unsafe { ::core::hint::unreachable_unchecked() }
                    }
                }
                NestedBlobChunkFillResolver::Corrupted(resolver_0) => {
                    match __this {
                        NestedBlobChunkFill::Corrupted(self_0, ..) => {
                            let out = unsafe {
                                out.cast_unchecked::<ArchivedVariantCorrupted>()
                            };
                            let tag_ptr = unsafe { &raw mut (*out.ptr()).0 };
                            unsafe {
                                tag_ptr.write(ArchivedTag::Corrupted);
                            }
                            let field_ptr = unsafe { &raw mut (*out.ptr()).1 };
                            let out_field = unsafe {
                                ::rkyv::Place::from_field_unchecked(out, field_ptr)
                            };
                            <usize as ::rkyv::Archive>::resolve(
                                self_0,
                                resolver_0,
                                out_field,
                            );
                        }
                        #[allow(unreachable_patterns)]
                        _ => unsafe { ::core::hint::unreachable_unchecked() }
                    }
                }
            }
        }
    }
};
unsafe impl ::rkyv::traits::Portable for ArchivedNestedBlobChunkFill
where
    usize: ::rkyv::Archive,
    usize: ::rkyv::Archive,
    usize: ::rkyv::Archive,
    <usize as ::rkyv::Archive>::Archived: ::rkyv::traits::Portable,
    <usize as ::rkyv::Archive>::Archived: ::rkyv::traits::Portable,
    <usize as ::rkyv::Archive>::Archived: ::rkyv::traits::Portable,
{}
#[automatically_derived]
impl<__S: ::rkyv::rancor::Fallible + ?Sized> ::rkyv::Serialize<__S>
for NestedBlobChunkFill
where
    usize: ::rkyv::Serialize<__S>,
    usize: ::rkyv::Serialize<__S>,
    usize: ::rkyv::Serialize<__S>,
{
    fn serialize(
        &self,
        serializer: &mut __S,
    ) -> ::core::result::Result<
        <Self as ::rkyv::Archive>::Resolver,
        <__S as ::rkyv::rancor::Fallible>::Error,
    > {
        let __this = self;
        ::core::result::Result::Ok(
            match __this {
                NestedBlobChunkFill::Full(_0, ..) => {
                    NestedBlobChunkFillResolver::Full(
                        <usize as ::rkyv::Serialize<__S>>::serialize(_0, serializer)?,
                    )
                }
                NestedBlobChunkFill::Partial(_0, ..) => {
                    NestedBlobChunkFillResolver::Partial(
                        <usize as ::rkyv::Serialize<__S>>::serialize(_0, serializer)?,
                    )
                }
                NestedBlobChunkFill::Corrupted(_0, ..) => {
                    NestedBlobChunkFillResolver::Corrupted(
                        <usize as ::rkyv::Serialize<__S>>::serialize(_0, serializer)?,
                    )
                }
            },
        )
    }
}
#[automatically_derived]
impl<
    __D: ::rkyv::rancor::Fallible + ?Sized,
> ::rkyv::Deserialize<NestedBlobChunkFill, __D> for ::rkyv::Archived<NestedBlobChunkFill>
where
    usize: ::rkyv::Archive,
    <usize as ::rkyv::Archive>::Archived: ::rkyv::Deserialize<usize, __D>,
    usize: ::rkyv::Archive,
    <usize as ::rkyv::Archive>::Archived: ::rkyv::Deserialize<usize, __D>,
    usize: ::rkyv::Archive,
    <usize as ::rkyv::Archive>::Archived: ::rkyv::Deserialize<usize, __D>,
{
    fn deserialize(
        &self,
        deserializer: &mut __D,
    ) -> ::core::result::Result<
        NestedBlobChunkFill,
        <__D as ::rkyv::rancor::Fallible>::Error,
    > {
        let __this = self;
        ::core::result::Result::Ok(
            match __this {
                Self::Full(_0, ..) => {
                    NestedBlobChunkFill::Full(
                        <<usize as ::rkyv::Archive>::Archived as ::rkyv::Deserialize<
                            usize,
                            __D,
                        >>::deserialize(_0, deserializer)?,
                    )
                }
                Self::Partial(_0, ..) => {
                    NestedBlobChunkFill::Partial(
                        <<usize as ::rkyv::Archive>::Archived as ::rkyv::Deserialize<
                            usize,
                            __D,
                        >>::deserialize(_0, deserializer)?,
                    )
                }
                Self::Corrupted(_0, ..) => {
                    NestedBlobChunkFill::Corrupted(
                        <<usize as ::rkyv::Archive>::Archived as ::rkyv::Deserialize<
                            usize,
                            __D,
                        >>::deserialize(_0, deserializer)?,
                    )
                }
            },
        )
    }
}
impl NestedBlobChunkFill {
    pub fn from_size(actual: usize, expected: usize) -> Self {
        if actual == expected {
            Self::Full(actual)
        } else if actual < expected {
            Self::Partial(actual)
        } else {
            Self::Corrupted(actual)
        }
    }
}
impl rewrite::traits::structural::blob::NetabaseBlobItem for NestedBlob {
    type Chunk = NestedBlobChunk;
    type BlobIter = std::vec::IntoIter<rewrite::results::NetabaseResult<Self::Chunk>>;
    const DEFAULT_CHUNK_SIZE: usize = 0usize;
    fn into_chunks(
        self,
        size: rewrite::traits::structural::blob::ChunkSize,
    ) -> Box<dyn Iterator<Item = Self::Chunk>> {
        Box::new(self.into_chunks_iter(size).filter_map(|r| r.ok()))
    }
    fn into_chunks_iter(
        self,
        size: rewrite::traits::structural::blob::ChunkSize,
    ) -> Self::BlobIter {
        let serialized_data: rewrite::results::NetabaseResult<Vec<u8>> = (|| -> rewrite::results::NetabaseResult<
            Vec<u8>,
        > {
            Ok(
                rkyv::to_bytes::<rkyv::rancor::Error>(&self)
                    .map_err(|e| rewrite::results::NetabaseError::Serialization(
                        ::alloc::__export::must_use({
                            ::alloc::fmt::format(
                                format_args!("rkyv serialization failed: {0:?}", e),
                            )
                        }),
                    ))?
                    .to_vec(),
            )
        })();
        let chunk_size = match size {
            rewrite::traits::structural::blob::ChunkSize::Default => {
                if Self::DEFAULT_CHUNK_SIZE > 0 {
                    Self::DEFAULT_CHUNK_SIZE
                } else {
                    1024
                }
            }
            rewrite::traits::structural::blob::ChunkSize::Size(n) => n,
        };
        match serialized_data {
            Ok(data) => {
                data.chunks(chunk_size)
                    .enumerate()
                    .map(|(index, chunk_data)| {
                        Ok(Self::Chunk {
                            index,
                            data: chunk_data.to_vec(),
                        })
                    })
                    .collect::<Vec<_>>()
                    .into_iter()
            }
            Err(e) => {
                ::alloc::boxed::box_assume_init_into_vec_unsafe(
                        ::alloc::intrinsics::write_box_via_move(
                            ::alloc::boxed::Box::new_uninit(),
                            [Err(e)],
                        ),
                    )
                    .into_iter()
            }
        }
    }
    fn try_from_chunks(
        chunks: impl Iterator<Item = Self::Chunk>,
        size: rewrite::traits::structural::blob::ChunkSize,
    ) -> rewrite::results::NetabaseResult<Self> {
        let mut sorted_chunks: Vec<_> = chunks.collect();
        sorted_chunks.sort_by_key(|c| c.index);
        if sorted_chunks.is_empty() {
            return Err(
                rewrite::results::NetabaseError::BlobReconstruction(
                    rewrite::results::BlobReconstructionError::MissingChunks,
                ),
            );
        }
        let chunk_size = match size {
            rewrite::traits::structural::blob::ChunkSize::Default => {
                if Self::DEFAULT_CHUNK_SIZE > 0 {
                    Self::DEFAULT_CHUNK_SIZE
                } else {
                    1024
                }
            }
            rewrite::traits::structural::blob::ChunkSize::Size(n) => n,
        };
        let mut missing_details = Vec::new();
        let mut next_expected = 0;
        let max_idx = sorted_chunks.last().map(|c| c.index).unwrap_or(0);
        for chunk in &sorted_chunks {
            while chunk.index > next_expected {
                missing_details
                    .push(
                        ::alloc::__export::must_use({
                            ::alloc::fmt::format(
                                format_args!(
                                    "{0:?}({{ Index: {1}, Size: {2} }})",
                                    NestedBlobChunkFill::Full(chunk_size),
                                    next_expected,
                                    chunk_size,
                                ),
                            )
                        }),
                    );
                next_expected += 1;
            }
            let fill = NestedBlobChunkFill::from_size(chunk.data.len(), chunk_size);
            match fill {
                NestedBlobChunkFill::Corrupted(size) => {
                    return Err(
                        rewrite::results::NetabaseError::BlobReconstruction(
                            rewrite::results::BlobReconstructionError::InvalidChunkData(
                                ::alloc::__export::must_use({
                                    ::alloc::fmt::format(
                                        format_args!(
                                            "Corrupted chunk detected: {0:?}({{ Index: {1}, Size: {2} }}). Max allowed size is {3}.",
                                            fill,
                                            chunk.index,
                                            size,
                                            chunk_size,
                                        ),
                                    )
                                }),
                            ),
                        ),
                    );
                }
                NestedBlobChunkFill::Partial(size) if chunk.index < max_idx => {
                    return Err(
                        rewrite::results::NetabaseError::BlobReconstruction(
                            rewrite::results::BlobReconstructionError::InvalidChunkData(
                                ::alloc::__export::must_use({
                                    ::alloc::fmt::format(
                                        format_args!(
                                            "Unexpected partial chunk in middle of stream: {0:?}({{ Index: {1}, Size: {2} }}). Expected {3} bytes.",
                                            fill,
                                            chunk.index,
                                            size,
                                            chunk_size,
                                        ),
                                    )
                                }),
                            ),
                        ),
                    );
                }
                _ => {}
            }
            if chunk.index == next_expected {
                next_expected += 1;
            }
        }
        if !missing_details.is_empty() {
            if let Some(last) = sorted_chunks.last() {
                let fill = NestedBlobChunkFill::from_size(last.data.len(), chunk_size);
                if #[allow(non_exhaustive_omitted_patterns)]
                match fill {
                    NestedBlobChunkFill::Full(_) => true,
                    _ => false,
                } {
                    missing_details
                        .push(
                            ::alloc::__export::must_use({
                                ::alloc::fmt::format(
                                    format_args!(
                                        "... (Stream truncated: last chunk was Full, expected more data after Index {0})",
                                        last.index,
                                    ),
                                )
                            }),
                        );
                }
            }
        }
        if !missing_details.is_empty() {
            return Err(
                rewrite::results::NetabaseError::BlobReconstruction(
                    rewrite::results::BlobReconstructionError::InvalidChunkData(
                        ::alloc::__export::must_use({
                            ::alloc::fmt::format(
                                format_args!(
                                    "Missing chunks: [{0}]. Total chunks present: {1}",
                                    missing_details.join(", "),
                                    sorted_chunks.len(),
                                ),
                            )
                        }),
                    ),
                ),
            );
        }
        let serialized_data: Vec<u8> = sorted_chunks
            .into_iter()
            .flat_map(|c| c.data)
            .collect();
        Ok(
            rkyv::from_bytes::<Self, rkyv::rancor::Error>(&serialized_data)
                .map_err(|e| rewrite::results::NetabaseError::Serialization(
                    ::alloc::__export::must_use({
                        ::alloc::fmt::format(
                            format_args!("rkyv deserialization failed: {0:?}", e),
                        )
                    }),
                ))?,
        )
    }
    fn get_blob(&self) -> &Self::Chunk {
        {
            ::core::panicking::panic_fmt(
                format_args!(
                    "not implemented: {0}",
                    format_args!("get_blob() requires storing a chunk reference"),
                ),
            );
        }
    }
}
impl IntoIterator for NestedBlob {
    type Item = rewrite::results::NetabaseResult<NestedBlobChunk>;
    type IntoIter = std::vec::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        rewrite::traits::structural::blob::NetabaseBlobItem::into_chunks_iter(
            self,
            rewrite::traits::structural::blob::ChunkSize::Default,
        )
    }
}
struct ParentBlob {
    #[blob_field(chunk_size(128))]
    child: NestedBlob,
}
#[automatically_derived]
///An archived [`ParentBlob`]
#[bytecheck(crate = ::rkyv::bytecheck)]
#[repr(C)]
struct ArchivedParentBlob
where
    NestedBlob: ::rkyv::Archive,
{
    ///The archived counterpart of [`ParentBlob::child`]
    child: <NestedBlob as ::rkyv::Archive>::Archived,
}
#[automatically_derived]
unsafe impl<
    __C: ::rkyv::bytecheck::rancor::Fallible + ?::core::marker::Sized,
> ::rkyv::bytecheck::CheckBytes<__C> for ArchivedParentBlob
where
    NestedBlob: ::rkyv::Archive,
    <__C as ::rkyv::bytecheck::rancor::Fallible>::Error: ::rkyv::bytecheck::rancor::Trace,
    <NestedBlob as ::rkyv::Archive>::Archived: ::rkyv::bytecheck::CheckBytes<__C>,
{
    unsafe fn check_bytes(
        value: *const Self,
        context: &mut __C,
    ) -> ::core::result::Result<
        (),
        <__C as ::rkyv::bytecheck::rancor::Fallible>::Error,
    > {
        <<NestedBlob as ::rkyv::Archive>::Archived as ::rkyv::bytecheck::CheckBytes<
            __C,
        >>::check_bytes(&raw const (*value).child, context)
            .map_err(|e| {
                <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Trace>::trace(
                    e,
                    ::rkyv::bytecheck::StructCheckContext {
                        struct_name: "ArchivedParentBlob",
                        field_name: "child",
                    },
                )
            })?;
        ::core::result::Result::Ok(())
    }
}
#[automatically_derived]
///The resolver for an archived [`ParentBlob`]
struct ParentBlobResolver
where
    NestedBlob: ::rkyv::Archive,
{
    child: <NestedBlob as ::rkyv::Archive>::Resolver,
}
impl ::rkyv::Archive for ParentBlob
where
    NestedBlob: ::rkyv::Archive,
{
    type Archived = ArchivedParentBlob;
    type Resolver = ParentBlobResolver;
    const COPY_OPTIMIZATION: ::rkyv::traits::CopyOptimization<Self> = unsafe {
        ::rkyv::traits::CopyOptimization::enable_if(
            0 + ::core::mem::size_of::<NestedBlob>()
                == ::core::mem::size_of::<ParentBlob>()
                && <NestedBlob as ::rkyv::Archive>::COPY_OPTIMIZATION.is_enabled()
                && const { builtin # offset_of(ParentBlob, child) }
                    == const { builtin # offset_of(ArchivedParentBlob, child) },
        )
    };
    #[allow(clippy::unit_arg)]
    fn resolve(&self, resolver: Self::Resolver, out: ::rkyv::Place<Self::Archived>) {
        let field_ptr = unsafe { &raw mut (*out.ptr()).child };
        let field_out = unsafe { ::rkyv::Place::from_field_unchecked(out, field_ptr) };
        <NestedBlob as ::rkyv::Archive>::resolve(&self.child, resolver.child, field_out);
    }
}
unsafe impl ::rkyv::traits::Portable for ArchivedParentBlob
where
    NestedBlob: ::rkyv::Archive,
    <NestedBlob as ::rkyv::Archive>::Archived: ::rkyv::traits::Portable,
{}
#[automatically_derived]
impl<__S: ::rkyv::rancor::Fallible + ?Sized> ::rkyv::Serialize<__S> for ParentBlob
where
    NestedBlob: ::rkyv::Serialize<__S>,
{
    fn serialize(
        &self,
        serializer: &mut __S,
    ) -> ::core::result::Result<
        <Self as ::rkyv::Archive>::Resolver,
        <__S as ::rkyv::rancor::Fallible>::Error,
    > {
        let __this = self;
        ::core::result::Result::Ok(ParentBlobResolver {
            child: <NestedBlob as ::rkyv::Serialize<
                __S,
            >>::serialize(&__this.child, serializer)?,
        })
    }
}
#[automatically_derived]
impl<__D: ::rkyv::rancor::Fallible + ?Sized> ::rkyv::Deserialize<ParentBlob, __D>
for ::rkyv::Archived<ParentBlob>
where
    NestedBlob: ::rkyv::Archive,
    <NestedBlob as ::rkyv::Archive>::Archived: ::rkyv::Deserialize<NestedBlob, __D>,
{
    fn deserialize(
        &self,
        deserializer: &mut __D,
    ) -> ::core::result::Result<ParentBlob, <__D as ::rkyv::rancor::Fallible>::Error> {
        let __this = self;
        ::core::result::Result::Ok(ParentBlob {
            child: <<NestedBlob as ::rkyv::Archive>::Archived as ::rkyv::Deserialize<
                NestedBlob,
                __D,
            >>::deserialize(&__this.child, deserializer)?,
        })
    }
}
#[automatically_derived]
impl ::core::fmt::Debug for ParentBlob {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::debug_struct_field1_finish(
            f,
            "ParentBlob",
            "child",
            &&self.child,
        )
    }
}
#[automatically_derived]
impl ::core::clone::Clone for ParentBlob {
    #[inline]
    fn clone(&self) -> ParentBlob {
        ParentBlob {
            child: ::core::clone::Clone::clone(&self.child),
        }
    }
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for ParentBlob {}
#[automatically_derived]
impl ::core::cmp::PartialEq for ParentBlob {
    #[inline]
    fn eq(&self, other: &ParentBlob) -> bool {
        self.child == other.child
    }
}
#[automatically_derived]
impl ::core::cmp::Eq for ParentBlob {
    #[inline]
    #[doc(hidden)]
    #[coverage(off)]
    fn assert_fields_are_eq(&self) {
        let _: ::core::cmp::AssertParamIsEq<NestedBlob>;
    }
}
pub struct ParentBlobChildChunk {
    pub index: usize,
    pub data: Vec<u8>,
}
#[automatically_derived]
impl ::core::fmt::Debug for ParentBlobChildChunk {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::debug_struct_field2_finish(
            f,
            "ParentBlobChildChunk",
            "index",
            &self.index,
            "data",
            &&self.data,
        )
    }
}
#[automatically_derived]
impl ::core::clone::Clone for ParentBlobChildChunk {
    #[inline]
    fn clone(&self) -> ParentBlobChildChunk {
        ParentBlobChildChunk {
            index: ::core::clone::Clone::clone(&self.index),
            data: ::core::clone::Clone::clone(&self.data),
        }
    }
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for ParentBlobChildChunk {}
#[automatically_derived]
impl ::core::cmp::PartialEq for ParentBlobChildChunk {
    #[inline]
    fn eq(&self, other: &ParentBlobChildChunk) -> bool {
        self.index == other.index && self.data == other.data
    }
}
#[automatically_derived]
impl ::core::cmp::Eq for ParentBlobChildChunk {
    #[inline]
    #[doc(hidden)]
    #[coverage(off)]
    fn assert_fields_are_eq(&self) {
        let _: ::core::cmp::AssertParamIsEq<usize>;
        let _: ::core::cmp::AssertParamIsEq<Vec<u8>>;
    }
}
#[automatically_derived]
impl ::core::cmp::PartialOrd for ParentBlobChildChunk {
    #[inline]
    fn partial_cmp(
        &self,
        other: &ParentBlobChildChunk,
    ) -> ::core::option::Option<::core::cmp::Ordering> {
        match ::core::cmp::PartialOrd::partial_cmp(&self.index, &other.index) {
            ::core::option::Option::Some(::core::cmp::Ordering::Equal) => {
                ::core::cmp::PartialOrd::partial_cmp(&self.data, &other.data)
            }
            cmp => cmp,
        }
    }
}
#[automatically_derived]
impl ::core::cmp::Ord for ParentBlobChildChunk {
    #[inline]
    fn cmp(&self, other: &ParentBlobChildChunk) -> ::core::cmp::Ordering {
        match ::core::cmp::Ord::cmp(&self.index, &other.index) {
            ::core::cmp::Ordering::Equal => {
                ::core::cmp::Ord::cmp(&self.data, &other.data)
            }
            cmp => cmp,
        }
    }
}
#[automatically_derived]
///An archived [`ParentBlobChildChunk`]
#[bytecheck(crate = ::rkyv::bytecheck)]
#[repr(C)]
pub struct ArchivedParentBlobChildChunk
where
    usize: ::rkyv::Archive,
    Vec<u8>: ::rkyv::Archive,
{
    ///The archived counterpart of [`ParentBlobChildChunk::index`]
    pub index: <usize as ::rkyv::Archive>::Archived,
    ///The archived counterpart of [`ParentBlobChildChunk::data`]
    pub data: <Vec<u8> as ::rkyv::Archive>::Archived,
}
#[automatically_derived]
unsafe impl<
    __C: ::rkyv::bytecheck::rancor::Fallible + ?::core::marker::Sized,
> ::rkyv::bytecheck::CheckBytes<__C> for ArchivedParentBlobChildChunk
where
    usize: ::rkyv::Archive,
    Vec<u8>: ::rkyv::Archive,
    <__C as ::rkyv::bytecheck::rancor::Fallible>::Error: ::rkyv::bytecheck::rancor::Trace,
    <usize as ::rkyv::Archive>::Archived: ::rkyv::bytecheck::CheckBytes<__C>,
    <Vec<u8> as ::rkyv::Archive>::Archived: ::rkyv::bytecheck::CheckBytes<__C>,
{
    unsafe fn check_bytes(
        value: *const Self,
        context: &mut __C,
    ) -> ::core::result::Result<
        (),
        <__C as ::rkyv::bytecheck::rancor::Fallible>::Error,
    > {
        <<usize as ::rkyv::Archive>::Archived as ::rkyv::bytecheck::CheckBytes<
            __C,
        >>::check_bytes(&raw const (*value).index, context)
            .map_err(|e| {
                <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Trace>::trace(
                    e,
                    ::rkyv::bytecheck::StructCheckContext {
                        struct_name: "ArchivedParentBlobChildChunk",
                        field_name: "index",
                    },
                )
            })?;
        <<Vec<
            u8,
        > as ::rkyv::Archive>::Archived as ::rkyv::bytecheck::CheckBytes<
            __C,
        >>::check_bytes(&raw const (*value).data, context)
            .map_err(|e| {
                <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Trace>::trace(
                    e,
                    ::rkyv::bytecheck::StructCheckContext {
                        struct_name: "ArchivedParentBlobChildChunk",
                        field_name: "data",
                    },
                )
            })?;
        ::core::result::Result::Ok(())
    }
}
#[automatically_derived]
///The resolver for an archived [`ParentBlobChildChunk`]
pub struct ParentBlobChildChunkResolver
where
    usize: ::rkyv::Archive,
    Vec<u8>: ::rkyv::Archive,
{
    index: <usize as ::rkyv::Archive>::Resolver,
    data: <Vec<u8> as ::rkyv::Archive>::Resolver,
}
impl ::rkyv::Archive for ParentBlobChildChunk
where
    usize: ::rkyv::Archive,
    Vec<u8>: ::rkyv::Archive,
{
    type Archived = ArchivedParentBlobChildChunk;
    type Resolver = ParentBlobChildChunkResolver;
    const COPY_OPTIMIZATION: ::rkyv::traits::CopyOptimization<Self> = unsafe {
        ::rkyv::traits::CopyOptimization::enable_if(
            0 + ::core::mem::size_of::<usize>() + ::core::mem::size_of::<Vec<u8>>()
                == ::core::mem::size_of::<ParentBlobChildChunk>()
                && <usize as ::rkyv::Archive>::COPY_OPTIMIZATION.is_enabled()
                && const { builtin # offset_of(ParentBlobChildChunk, index) }
                    == const { builtin # offset_of(ArchivedParentBlobChildChunk, index) }
                && <Vec<u8> as ::rkyv::Archive>::COPY_OPTIMIZATION.is_enabled()
                && const { builtin # offset_of(ParentBlobChildChunk, data) }
                    == const { builtin # offset_of(ArchivedParentBlobChildChunk, data) },
        )
    };
    #[allow(clippy::unit_arg)]
    fn resolve(&self, resolver: Self::Resolver, out: ::rkyv::Place<Self::Archived>) {
        let field_ptr = unsafe { &raw mut (*out.ptr()).index };
        let field_out = unsafe { ::rkyv::Place::from_field_unchecked(out, field_ptr) };
        <usize as ::rkyv::Archive>::resolve(&self.index, resolver.index, field_out);
        let field_ptr = unsafe { &raw mut (*out.ptr()).data };
        let field_out = unsafe { ::rkyv::Place::from_field_unchecked(out, field_ptr) };
        <Vec<u8> as ::rkyv::Archive>::resolve(&self.data, resolver.data, field_out);
    }
}
unsafe impl ::rkyv::traits::Portable for ArchivedParentBlobChildChunk
where
    usize: ::rkyv::Archive,
    Vec<u8>: ::rkyv::Archive,
    <usize as ::rkyv::Archive>::Archived: ::rkyv::traits::Portable,
    <Vec<u8> as ::rkyv::Archive>::Archived: ::rkyv::traits::Portable,
{}
#[automatically_derived]
impl<__S: ::rkyv::rancor::Fallible + ?Sized> ::rkyv::Serialize<__S>
for ParentBlobChildChunk
where
    usize: ::rkyv::Serialize<__S>,
    Vec<u8>: ::rkyv::Serialize<__S>,
{
    fn serialize(
        &self,
        serializer: &mut __S,
    ) -> ::core::result::Result<
        <Self as ::rkyv::Archive>::Resolver,
        <__S as ::rkyv::rancor::Fallible>::Error,
    > {
        let __this = self;
        ::core::result::Result::Ok(ParentBlobChildChunkResolver {
            index: <usize as ::rkyv::Serialize<
                __S,
            >>::serialize(&__this.index, serializer)?,
            data: <Vec<
                u8,
            > as ::rkyv::Serialize<__S>>::serialize(&__this.data, serializer)?,
        })
    }
}
#[automatically_derived]
impl<
    __D: ::rkyv::rancor::Fallible + ?Sized,
> ::rkyv::Deserialize<ParentBlobChildChunk, __D>
for ::rkyv::Archived<ParentBlobChildChunk>
where
    usize: ::rkyv::Archive,
    <usize as ::rkyv::Archive>::Archived: ::rkyv::Deserialize<usize, __D>,
    Vec<u8>: ::rkyv::Archive,
    <Vec<u8> as ::rkyv::Archive>::Archived: ::rkyv::Deserialize<Vec<u8>, __D>,
{
    fn deserialize(
        &self,
        deserializer: &mut __D,
    ) -> ::core::result::Result<
        ParentBlobChildChunk,
        <__D as ::rkyv::rancor::Fallible>::Error,
    > {
        let __this = self;
        ::core::result::Result::Ok(ParentBlobChildChunk {
            index: <<usize as ::rkyv::Archive>::Archived as ::rkyv::Deserialize<
                usize,
                __D,
            >>::deserialize(&__this.index, deserializer)?,
            data: <<Vec<
                u8,
            > as ::rkyv::Archive>::Archived as ::rkyv::Deserialize<
                Vec<u8>,
                __D,
            >>::deserialize(&__this.data, deserializer)?,
        })
    }
}
impl ::rewrite::traits::structural::blob::BlobItemChunk for ParentBlobChildChunk {
    type Index = usize;
    fn get_index(&self) -> &Self::Index {
        &self.index
    }
}
pub enum ParentBlobChunk {
    Child(ParentBlobChildChunk),
    Missing,
}
#[automatically_derived]
impl ::core::fmt::Debug for ParentBlobChunk {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match self {
            ParentBlobChunk::Child(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "Child", &__self_0)
            }
            ParentBlobChunk::Missing => ::core::fmt::Formatter::write_str(f, "Missing"),
        }
    }
}
#[automatically_derived]
impl ::core::clone::Clone for ParentBlobChunk {
    #[inline]
    fn clone(&self) -> ParentBlobChunk {
        match self {
            ParentBlobChunk::Child(__self_0) => {
                ParentBlobChunk::Child(::core::clone::Clone::clone(__self_0))
            }
            ParentBlobChunk::Missing => ParentBlobChunk::Missing,
        }
    }
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for ParentBlobChunk {}
#[automatically_derived]
impl ::core::cmp::PartialEq for ParentBlobChunk {
    #[inline]
    fn eq(&self, other: &ParentBlobChunk) -> bool {
        let __self_discr = ::core::intrinsics::discriminant_value(self);
        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
        __self_discr == __arg1_discr
            && match (self, other) {
                (ParentBlobChunk::Child(__self_0), ParentBlobChunk::Child(__arg1_0)) => {
                    __self_0 == __arg1_0
                }
                _ => true,
            }
    }
}
#[automatically_derived]
impl ::core::cmp::Eq for ParentBlobChunk {
    #[inline]
    #[doc(hidden)]
    #[coverage(off)]
    fn assert_fields_are_eq(&self) {
        let _: ::core::cmp::AssertParamIsEq<ParentBlobChildChunk>;
    }
}
#[automatically_derived]
impl ::core::cmp::PartialOrd for ParentBlobChunk {
    #[inline]
    fn partial_cmp(
        &self,
        other: &ParentBlobChunk,
    ) -> ::core::option::Option<::core::cmp::Ordering> {
        let __self_discr = ::core::intrinsics::discriminant_value(self);
        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
        match (self, other) {
            (ParentBlobChunk::Child(__self_0), ParentBlobChunk::Child(__arg1_0)) => {
                ::core::cmp::PartialOrd::partial_cmp(__self_0, __arg1_0)
            }
            _ => ::core::cmp::PartialOrd::partial_cmp(&__self_discr, &__arg1_discr),
        }
    }
}
#[automatically_derived]
impl ::core::cmp::Ord for ParentBlobChunk {
    #[inline]
    fn cmp(&self, other: &ParentBlobChunk) -> ::core::cmp::Ordering {
        let __self_discr = ::core::intrinsics::discriminant_value(self);
        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
        match ::core::cmp::Ord::cmp(&__self_discr, &__arg1_discr) {
            ::core::cmp::Ordering::Equal => {
                match (self, other) {
                    (
                        ParentBlobChunk::Child(__self_0),
                        ParentBlobChunk::Child(__arg1_0),
                    ) => ::core::cmp::Ord::cmp(__self_0, __arg1_0),
                    _ => ::core::cmp::Ordering::Equal,
                }
            }
            cmp => cmp,
        }
    }
}
#[automatically_derived]
///An archived [`ParentBlobChunk`]
#[bytecheck(crate = ::rkyv::bytecheck)]
#[repr(u8)]
pub enum ArchivedParentBlobChunk
where
    ParentBlobChildChunk: ::rkyv::Archive,
{
    ///The archived counterpart of [`ParentBlobChunk::Child`]
    #[allow(dead_code)]
    Child(
        ///The archived counterpart of [`ParentBlobChunk::Child::0`]
        <ParentBlobChildChunk as ::rkyv::Archive>::Archived,
    ),
    ///The archived counterpart of [`ParentBlobChunk::Missing`]
    #[allow(dead_code)]
    Missing,
}
const _: () = {
    #[repr(u8)]
    enum Tag {
        Child,
        Missing,
    }
    struct Discriminant;
    #[automatically_derived]
    impl Discriminant {
        #[allow(non_upper_case_globals)]
        const Child: u8 = Tag::Child as u8;
        #[allow(non_upper_case_globals)]
        const Missing: u8 = Tag::Missing as u8;
    }
    #[repr(C)]
    struct VariantChild(
        Tag,
        <ParentBlobChildChunk as ::rkyv::Archive>::Archived,
        ::core::marker::PhantomData<ArchivedParentBlobChunk>,
    )
    where
        ParentBlobChildChunk: ::rkyv::Archive;
    #[automatically_derived]
    unsafe impl<
        __C: ::rkyv::bytecheck::rancor::Fallible + ?::core::marker::Sized,
    > ::rkyv::bytecheck::CheckBytes<__C> for ArchivedParentBlobChunk
    where
        ParentBlobChildChunk: ::rkyv::Archive,
        <__C as ::rkyv::bytecheck::rancor::Fallible>::Error: ::rkyv::bytecheck::rancor::Source,
        <ParentBlobChildChunk as ::rkyv::Archive>::Archived: ::rkyv::bytecheck::CheckBytes<
            __C,
        >,
    {
        unsafe fn check_bytes(
            value: *const Self,
            context: &mut __C,
        ) -> ::core::result::Result<
            (),
            <__C as ::rkyv::bytecheck::rancor::Fallible>::Error,
        > {
            let tag = *value.cast::<u8>();
            match tag {
                Discriminant::Child => {
                    let value = value.cast::<VariantChild>();
                    <<ParentBlobChildChunk as ::rkyv::Archive>::Archived as ::rkyv::bytecheck::CheckBytes<
                        __C,
                    >>::check_bytes(&raw const (*value).1, context)
                        .map_err(|e| {
                            <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Trace>::trace(
                                e,
                                ::rkyv::bytecheck::UnnamedEnumVariantCheckContext {
                                    enum_name: "ArchivedParentBlobChunk",
                                    variant_name: "Child",
                                    field_index: 1,
                                },
                            )
                        })?;
                }
                Discriminant::Missing => {}
                _ => {
                    return ::core::result::Result::Err(
                        <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Source>::new(::rkyv::bytecheck::InvalidEnumDiscriminantError {
                            enum_name: "ArchivedParentBlobChunk",
                            invalid_discriminant: tag,
                        }),
                    );
                }
            }
            ::core::result::Result::Ok(())
        }
    }
};
#[automatically_derived]
///The resolver for an archived [`ParentBlobChunk`]
pub enum ParentBlobChunkResolver
where
    ParentBlobChildChunk: ::rkyv::Archive,
{
    ///The resolver for [`ParentBlobChunk::Child`]
    #[allow(dead_code)]
    Child(<ParentBlobChildChunk as ::rkyv::Archive>::Resolver),
    ///The resolver for [`ParentBlobChunk::Missing`]
    #[allow(dead_code)]
    Missing,
}
const _: () = {
    #[repr(u8)]
    enum ArchivedTag {
        Child,
        Missing,
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for ArchivedTag {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for ArchivedTag {
        #[inline]
        fn eq(&self, other: &ArchivedTag) -> bool {
            let __self_discr = ::core::intrinsics::discriminant_value(self);
            let __arg1_discr = ::core::intrinsics::discriminant_value(other);
            __self_discr == __arg1_discr
        }
    }
    #[automatically_derived]
    impl ::core::cmp::PartialOrd for ArchivedTag {
        #[inline]
        fn partial_cmp(
            &self,
            other: &ArchivedTag,
        ) -> ::core::option::Option<::core::cmp::Ordering> {
            let __self_discr = ::core::intrinsics::discriminant_value(self);
            let __arg1_discr = ::core::intrinsics::discriminant_value(other);
            ::core::cmp::PartialOrd::partial_cmp(&__self_discr, &__arg1_discr)
        }
    }
    #[repr(C)]
    struct ArchivedVariantChild(
        ArchivedTag,
        <ParentBlobChildChunk as ::rkyv::Archive>::Archived,
        ::core::marker::PhantomData<ParentBlobChunk>,
    )
    where
        ParentBlobChildChunk: ::rkyv::Archive;
    impl ::rkyv::Archive for ParentBlobChunk
    where
        ParentBlobChildChunk: ::rkyv::Archive,
    {
        type Archived = ArchivedParentBlobChunk;
        type Resolver = ParentBlobChunkResolver;
        #[allow(clippy::unit_arg)]
        fn resolve(
            &self,
            resolver: <Self as ::rkyv::Archive>::Resolver,
            out: ::rkyv::Place<<Self as ::rkyv::Archive>::Archived>,
        ) {
            let __this = self;
            match resolver {
                ParentBlobChunkResolver::Child(resolver_0) => {
                    match __this {
                        ParentBlobChunk::Child(self_0, ..) => {
                            let out = unsafe {
                                out.cast_unchecked::<ArchivedVariantChild>()
                            };
                            let tag_ptr = unsafe { &raw mut (*out.ptr()).0 };
                            unsafe {
                                tag_ptr.write(ArchivedTag::Child);
                            }
                            let field_ptr = unsafe { &raw mut (*out.ptr()).1 };
                            let out_field = unsafe {
                                ::rkyv::Place::from_field_unchecked(out, field_ptr)
                            };
                            <ParentBlobChildChunk as ::rkyv::Archive>::resolve(
                                self_0,
                                resolver_0,
                                out_field,
                            );
                        }
                        #[allow(unreachable_patterns)]
                        _ => unsafe { ::core::hint::unreachable_unchecked() }
                    }
                }
                ParentBlobChunkResolver::Missing => {
                    let out = unsafe { out.cast_unchecked::<ArchivedTag>() };
                    unsafe {
                        out.write_unchecked(ArchivedTag::Missing);
                    }
                }
            }
        }
    }
};
unsafe impl ::rkyv::traits::Portable for ArchivedParentBlobChunk
where
    ParentBlobChildChunk: ::rkyv::Archive,
    <ParentBlobChildChunk as ::rkyv::Archive>::Archived: ::rkyv::traits::Portable,
{}
#[automatically_derived]
impl<__S: ::rkyv::rancor::Fallible + ?Sized> ::rkyv::Serialize<__S> for ParentBlobChunk
where
    ParentBlobChildChunk: ::rkyv::Serialize<__S>,
{
    fn serialize(
        &self,
        serializer: &mut __S,
    ) -> ::core::result::Result<
        <Self as ::rkyv::Archive>::Resolver,
        <__S as ::rkyv::rancor::Fallible>::Error,
    > {
        let __this = self;
        ::core::result::Result::Ok(
            match __this {
                ParentBlobChunk::Child(_0, ..) => {
                    ParentBlobChunkResolver::Child(
                        <ParentBlobChildChunk as ::rkyv::Serialize<
                            __S,
                        >>::serialize(_0, serializer)?,
                    )
                }
                ParentBlobChunk::Missing => ParentBlobChunkResolver::Missing,
            },
        )
    }
}
#[automatically_derived]
impl<__D: ::rkyv::rancor::Fallible + ?Sized> ::rkyv::Deserialize<ParentBlobChunk, __D>
for ::rkyv::Archived<ParentBlobChunk>
where
    ParentBlobChildChunk: ::rkyv::Archive,
    <ParentBlobChildChunk as ::rkyv::Archive>::Archived: ::rkyv::Deserialize<
        ParentBlobChildChunk,
        __D,
    >,
{
    fn deserialize(
        &self,
        deserializer: &mut __D,
    ) -> ::core::result::Result<
        ParentBlobChunk,
        <__D as ::rkyv::rancor::Fallible>::Error,
    > {
        let __this = self;
        ::core::result::Result::Ok(
            match __this {
                Self::Child(_0, ..) => {
                    ParentBlobChunk::Child(
                        <<ParentBlobChildChunk as ::rkyv::Archive>::Archived as ::rkyv::Deserialize<
                            ParentBlobChildChunk,
                            __D,
                        >>::deserialize(_0, deserializer)?,
                    )
                }
                Self::Missing => ParentBlobChunk::Missing,
            },
        )
    }
}
pub enum ParentBlobChunkFill {
    Full(usize),
    Partial(usize),
    Corrupted(usize),
}
#[automatically_derived]
impl ::core::fmt::Debug for ParentBlobChunkFill {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match self {
            ParentBlobChunkFill::Full(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "Full", &__self_0)
            }
            ParentBlobChunkFill::Partial(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(
                    f,
                    "Partial",
                    &__self_0,
                )
            }
            ParentBlobChunkFill::Corrupted(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(
                    f,
                    "Corrupted",
                    &__self_0,
                )
            }
        }
    }
}
#[automatically_derived]
#[doc(hidden)]
unsafe impl ::core::clone::TrivialClone for ParentBlobChunkFill {}
#[automatically_derived]
impl ::core::clone::Clone for ParentBlobChunkFill {
    #[inline]
    fn clone(&self) -> ParentBlobChunkFill {
        let _: ::core::clone::AssertParamIsClone<usize>;
        *self
    }
}
#[automatically_derived]
impl ::core::marker::Copy for ParentBlobChunkFill {}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for ParentBlobChunkFill {}
#[automatically_derived]
impl ::core::cmp::PartialEq for ParentBlobChunkFill {
    #[inline]
    fn eq(&self, other: &ParentBlobChunkFill) -> bool {
        let __self_discr = ::core::intrinsics::discriminant_value(self);
        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
        __self_discr == __arg1_discr
            && match (self, other) {
                (
                    ParentBlobChunkFill::Full(__self_0),
                    ParentBlobChunkFill::Full(__arg1_0),
                ) => __self_0 == __arg1_0,
                (
                    ParentBlobChunkFill::Partial(__self_0),
                    ParentBlobChunkFill::Partial(__arg1_0),
                ) => __self_0 == __arg1_0,
                (
                    ParentBlobChunkFill::Corrupted(__self_0),
                    ParentBlobChunkFill::Corrupted(__arg1_0),
                ) => __self_0 == __arg1_0,
                _ => unsafe { ::core::intrinsics::unreachable() }
            }
    }
}
#[automatically_derived]
impl ::core::cmp::Eq for ParentBlobChunkFill {
    #[inline]
    #[doc(hidden)]
    #[coverage(off)]
    fn assert_fields_are_eq(&self) {
        let _: ::core::cmp::AssertParamIsEq<usize>;
    }
}
#[automatically_derived]
impl ::core::cmp::PartialOrd for ParentBlobChunkFill {
    #[inline]
    fn partial_cmp(
        &self,
        other: &ParentBlobChunkFill,
    ) -> ::core::option::Option<::core::cmp::Ordering> {
        let __self_discr = ::core::intrinsics::discriminant_value(self);
        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
        match (self, other) {
            (
                ParentBlobChunkFill::Full(__self_0),
                ParentBlobChunkFill::Full(__arg1_0),
            ) => ::core::cmp::PartialOrd::partial_cmp(__self_0, __arg1_0),
            (
                ParentBlobChunkFill::Partial(__self_0),
                ParentBlobChunkFill::Partial(__arg1_0),
            ) => ::core::cmp::PartialOrd::partial_cmp(__self_0, __arg1_0),
            (
                ParentBlobChunkFill::Corrupted(__self_0),
                ParentBlobChunkFill::Corrupted(__arg1_0),
            ) => ::core::cmp::PartialOrd::partial_cmp(__self_0, __arg1_0),
            _ => ::core::cmp::PartialOrd::partial_cmp(&__self_discr, &__arg1_discr),
        }
    }
}
#[automatically_derived]
impl ::core::cmp::Ord for ParentBlobChunkFill {
    #[inline]
    fn cmp(&self, other: &ParentBlobChunkFill) -> ::core::cmp::Ordering {
        let __self_discr = ::core::intrinsics::discriminant_value(self);
        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
        match ::core::cmp::Ord::cmp(&__self_discr, &__arg1_discr) {
            ::core::cmp::Ordering::Equal => {
                match (self, other) {
                    (
                        ParentBlobChunkFill::Full(__self_0),
                        ParentBlobChunkFill::Full(__arg1_0),
                    ) => ::core::cmp::Ord::cmp(__self_0, __arg1_0),
                    (
                        ParentBlobChunkFill::Partial(__self_0),
                        ParentBlobChunkFill::Partial(__arg1_0),
                    ) => ::core::cmp::Ord::cmp(__self_0, __arg1_0),
                    (
                        ParentBlobChunkFill::Corrupted(__self_0),
                        ParentBlobChunkFill::Corrupted(__arg1_0),
                    ) => ::core::cmp::Ord::cmp(__self_0, __arg1_0),
                    _ => unsafe { ::core::intrinsics::unreachable() }
                }
            }
            cmp => cmp,
        }
    }
}
#[automatically_derived]
///An archived [`ParentBlobChunkFill`]
#[bytecheck(crate = ::rkyv::bytecheck)]
#[repr(u8)]
pub enum ArchivedParentBlobChunkFill
where
    usize: ::rkyv::Archive,
    usize: ::rkyv::Archive,
    usize: ::rkyv::Archive,
{
    ///The archived counterpart of [`ParentBlobChunkFill::Full`]
    #[allow(dead_code)]
    Full(
        ///The archived counterpart of [`ParentBlobChunkFill::Full::0`]
        <usize as ::rkyv::Archive>::Archived,
    ),
    ///The archived counterpart of [`ParentBlobChunkFill::Partial`]
    #[allow(dead_code)]
    Partial(
        ///The archived counterpart of [`ParentBlobChunkFill::Partial::0`]
        <usize as ::rkyv::Archive>::Archived,
    ),
    ///The archived counterpart of [`ParentBlobChunkFill::Corrupted`]
    #[allow(dead_code)]
    Corrupted(
        ///The archived counterpart of [`ParentBlobChunkFill::Corrupted::0`]
        <usize as ::rkyv::Archive>::Archived,
    ),
}
const _: () = {
    #[repr(u8)]
    enum Tag {
        Full,
        Partial,
        Corrupted,
    }
    struct Discriminant;
    #[automatically_derived]
    impl Discriminant {
        #[allow(non_upper_case_globals)]
        const Full: u8 = Tag::Full as u8;
        #[allow(non_upper_case_globals)]
        const Partial: u8 = Tag::Partial as u8;
        #[allow(non_upper_case_globals)]
        const Corrupted: u8 = Tag::Corrupted as u8;
    }
    #[repr(C)]
    struct VariantFull(
        Tag,
        <usize as ::rkyv::Archive>::Archived,
        ::core::marker::PhantomData<ArchivedParentBlobChunkFill>,
    )
    where
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive;
    #[repr(C)]
    struct VariantPartial(
        Tag,
        <usize as ::rkyv::Archive>::Archived,
        ::core::marker::PhantomData<ArchivedParentBlobChunkFill>,
    )
    where
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive;
    #[repr(C)]
    struct VariantCorrupted(
        Tag,
        <usize as ::rkyv::Archive>::Archived,
        ::core::marker::PhantomData<ArchivedParentBlobChunkFill>,
    )
    where
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive;
    #[automatically_derived]
    unsafe impl<
        __C: ::rkyv::bytecheck::rancor::Fallible + ?::core::marker::Sized,
    > ::rkyv::bytecheck::CheckBytes<__C> for ArchivedParentBlobChunkFill
    where
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive,
        <__C as ::rkyv::bytecheck::rancor::Fallible>::Error: ::rkyv::bytecheck::rancor::Source,
        <usize as ::rkyv::Archive>::Archived: ::rkyv::bytecheck::CheckBytes<__C>,
        <usize as ::rkyv::Archive>::Archived: ::rkyv::bytecheck::CheckBytes<__C>,
        <usize as ::rkyv::Archive>::Archived: ::rkyv::bytecheck::CheckBytes<__C>,
    {
        unsafe fn check_bytes(
            value: *const Self,
            context: &mut __C,
        ) -> ::core::result::Result<
            (),
            <__C as ::rkyv::bytecheck::rancor::Fallible>::Error,
        > {
            let tag = *value.cast::<u8>();
            match tag {
                Discriminant::Full => {
                    let value = value.cast::<VariantFull>();
                    <<usize as ::rkyv::Archive>::Archived as ::rkyv::bytecheck::CheckBytes<
                        __C,
                    >>::check_bytes(&raw const (*value).1, context)
                        .map_err(|e| {
                            <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Trace>::trace(
                                e,
                                ::rkyv::bytecheck::UnnamedEnumVariantCheckContext {
                                    enum_name: "ArchivedParentBlobChunkFill",
                                    variant_name: "Full",
                                    field_index: 1,
                                },
                            )
                        })?;
                }
                Discriminant::Partial => {
                    let value = value.cast::<VariantPartial>();
                    <<usize as ::rkyv::Archive>::Archived as ::rkyv::bytecheck::CheckBytes<
                        __C,
                    >>::check_bytes(&raw const (*value).1, context)
                        .map_err(|e| {
                            <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Trace>::trace(
                                e,
                                ::rkyv::bytecheck::UnnamedEnumVariantCheckContext {
                                    enum_name: "ArchivedParentBlobChunkFill",
                                    variant_name: "Partial",
                                    field_index: 1,
                                },
                            )
                        })?;
                }
                Discriminant::Corrupted => {
                    let value = value.cast::<VariantCorrupted>();
                    <<usize as ::rkyv::Archive>::Archived as ::rkyv::bytecheck::CheckBytes<
                        __C,
                    >>::check_bytes(&raw const (*value).1, context)
                        .map_err(|e| {
                            <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Trace>::trace(
                                e,
                                ::rkyv::bytecheck::UnnamedEnumVariantCheckContext {
                                    enum_name: "ArchivedParentBlobChunkFill",
                                    variant_name: "Corrupted",
                                    field_index: 1,
                                },
                            )
                        })?;
                }
                _ => {
                    return ::core::result::Result::Err(
                        <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Source>::new(::rkyv::bytecheck::InvalidEnumDiscriminantError {
                            enum_name: "ArchivedParentBlobChunkFill",
                            invalid_discriminant: tag,
                        }),
                    );
                }
            }
            ::core::result::Result::Ok(())
        }
    }
};
#[automatically_derived]
///The resolver for an archived [`ParentBlobChunkFill`]
pub enum ParentBlobChunkFillResolver
where
    usize: ::rkyv::Archive,
    usize: ::rkyv::Archive,
    usize: ::rkyv::Archive,
{
    ///The resolver for [`ParentBlobChunkFill::Full`]
    #[allow(dead_code)]
    Full(<usize as ::rkyv::Archive>::Resolver),
    ///The resolver for [`ParentBlobChunkFill::Partial`]
    #[allow(dead_code)]
    Partial(<usize as ::rkyv::Archive>::Resolver),
    ///The resolver for [`ParentBlobChunkFill::Corrupted`]
    #[allow(dead_code)]
    Corrupted(<usize as ::rkyv::Archive>::Resolver),
}
const _: () = {
    #[repr(u8)]
    enum ArchivedTag {
        Full,
        Partial,
        Corrupted,
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for ArchivedTag {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for ArchivedTag {
        #[inline]
        fn eq(&self, other: &ArchivedTag) -> bool {
            let __self_discr = ::core::intrinsics::discriminant_value(self);
            let __arg1_discr = ::core::intrinsics::discriminant_value(other);
            __self_discr == __arg1_discr
        }
    }
    #[automatically_derived]
    impl ::core::cmp::PartialOrd for ArchivedTag {
        #[inline]
        fn partial_cmp(
            &self,
            other: &ArchivedTag,
        ) -> ::core::option::Option<::core::cmp::Ordering> {
            let __self_discr = ::core::intrinsics::discriminant_value(self);
            let __arg1_discr = ::core::intrinsics::discriminant_value(other);
            ::core::cmp::PartialOrd::partial_cmp(&__self_discr, &__arg1_discr)
        }
    }
    #[repr(C)]
    struct ArchivedVariantFull(
        ArchivedTag,
        <usize as ::rkyv::Archive>::Archived,
        ::core::marker::PhantomData<ParentBlobChunkFill>,
    )
    where
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive;
    #[repr(C)]
    struct ArchivedVariantPartial(
        ArchivedTag,
        <usize as ::rkyv::Archive>::Archived,
        ::core::marker::PhantomData<ParentBlobChunkFill>,
    )
    where
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive;
    #[repr(C)]
    struct ArchivedVariantCorrupted(
        ArchivedTag,
        <usize as ::rkyv::Archive>::Archived,
        ::core::marker::PhantomData<ParentBlobChunkFill>,
    )
    where
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive;
    impl ::rkyv::Archive for ParentBlobChunkFill
    where
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive,
    {
        type Archived = ArchivedParentBlobChunkFill;
        type Resolver = ParentBlobChunkFillResolver;
        #[allow(clippy::unit_arg)]
        fn resolve(
            &self,
            resolver: <Self as ::rkyv::Archive>::Resolver,
            out: ::rkyv::Place<<Self as ::rkyv::Archive>::Archived>,
        ) {
            let __this = self;
            match resolver {
                ParentBlobChunkFillResolver::Full(resolver_0) => {
                    match __this {
                        ParentBlobChunkFill::Full(self_0, ..) => {
                            let out = unsafe {
                                out.cast_unchecked::<ArchivedVariantFull>()
                            };
                            let tag_ptr = unsafe { &raw mut (*out.ptr()).0 };
                            unsafe {
                                tag_ptr.write(ArchivedTag::Full);
                            }
                            let field_ptr = unsafe { &raw mut (*out.ptr()).1 };
                            let out_field = unsafe {
                                ::rkyv::Place::from_field_unchecked(out, field_ptr)
                            };
                            <usize as ::rkyv::Archive>::resolve(
                                self_0,
                                resolver_0,
                                out_field,
                            );
                        }
                        #[allow(unreachable_patterns)]
                        _ => unsafe { ::core::hint::unreachable_unchecked() }
                    }
                }
                ParentBlobChunkFillResolver::Partial(resolver_0) => {
                    match __this {
                        ParentBlobChunkFill::Partial(self_0, ..) => {
                            let out = unsafe {
                                out.cast_unchecked::<ArchivedVariantPartial>()
                            };
                            let tag_ptr = unsafe { &raw mut (*out.ptr()).0 };
                            unsafe {
                                tag_ptr.write(ArchivedTag::Partial);
                            }
                            let field_ptr = unsafe { &raw mut (*out.ptr()).1 };
                            let out_field = unsafe {
                                ::rkyv::Place::from_field_unchecked(out, field_ptr)
                            };
                            <usize as ::rkyv::Archive>::resolve(
                                self_0,
                                resolver_0,
                                out_field,
                            );
                        }
                        #[allow(unreachable_patterns)]
                        _ => unsafe { ::core::hint::unreachable_unchecked() }
                    }
                }
                ParentBlobChunkFillResolver::Corrupted(resolver_0) => {
                    match __this {
                        ParentBlobChunkFill::Corrupted(self_0, ..) => {
                            let out = unsafe {
                                out.cast_unchecked::<ArchivedVariantCorrupted>()
                            };
                            let tag_ptr = unsafe { &raw mut (*out.ptr()).0 };
                            unsafe {
                                tag_ptr.write(ArchivedTag::Corrupted);
                            }
                            let field_ptr = unsafe { &raw mut (*out.ptr()).1 };
                            let out_field = unsafe {
                                ::rkyv::Place::from_field_unchecked(out, field_ptr)
                            };
                            <usize as ::rkyv::Archive>::resolve(
                                self_0,
                                resolver_0,
                                out_field,
                            );
                        }
                        #[allow(unreachable_patterns)]
                        _ => unsafe { ::core::hint::unreachable_unchecked() }
                    }
                }
            }
        }
    }
};
unsafe impl ::rkyv::traits::Portable for ArchivedParentBlobChunkFill
where
    usize: ::rkyv::Archive,
    usize: ::rkyv::Archive,
    usize: ::rkyv::Archive,
    <usize as ::rkyv::Archive>::Archived: ::rkyv::traits::Portable,
    <usize as ::rkyv::Archive>::Archived: ::rkyv::traits::Portable,
    <usize as ::rkyv::Archive>::Archived: ::rkyv::traits::Portable,
{}
#[automatically_derived]
impl<__S: ::rkyv::rancor::Fallible + ?Sized> ::rkyv::Serialize<__S>
for ParentBlobChunkFill
where
    usize: ::rkyv::Serialize<__S>,
    usize: ::rkyv::Serialize<__S>,
    usize: ::rkyv::Serialize<__S>,
{
    fn serialize(
        &self,
        serializer: &mut __S,
    ) -> ::core::result::Result<
        <Self as ::rkyv::Archive>::Resolver,
        <__S as ::rkyv::rancor::Fallible>::Error,
    > {
        let __this = self;
        ::core::result::Result::Ok(
            match __this {
                ParentBlobChunkFill::Full(_0, ..) => {
                    ParentBlobChunkFillResolver::Full(
                        <usize as ::rkyv::Serialize<__S>>::serialize(_0, serializer)?,
                    )
                }
                ParentBlobChunkFill::Partial(_0, ..) => {
                    ParentBlobChunkFillResolver::Partial(
                        <usize as ::rkyv::Serialize<__S>>::serialize(_0, serializer)?,
                    )
                }
                ParentBlobChunkFill::Corrupted(_0, ..) => {
                    ParentBlobChunkFillResolver::Corrupted(
                        <usize as ::rkyv::Serialize<__S>>::serialize(_0, serializer)?,
                    )
                }
            },
        )
    }
}
#[automatically_derived]
impl<
    __D: ::rkyv::rancor::Fallible + ?Sized,
> ::rkyv::Deserialize<ParentBlobChunkFill, __D> for ::rkyv::Archived<ParentBlobChunkFill>
where
    usize: ::rkyv::Archive,
    <usize as ::rkyv::Archive>::Archived: ::rkyv::Deserialize<usize, __D>,
    usize: ::rkyv::Archive,
    <usize as ::rkyv::Archive>::Archived: ::rkyv::Deserialize<usize, __D>,
    usize: ::rkyv::Archive,
    <usize as ::rkyv::Archive>::Archived: ::rkyv::Deserialize<usize, __D>,
{
    fn deserialize(
        &self,
        deserializer: &mut __D,
    ) -> ::core::result::Result<
        ParentBlobChunkFill,
        <__D as ::rkyv::rancor::Fallible>::Error,
    > {
        let __this = self;
        ::core::result::Result::Ok(
            match __this {
                Self::Full(_0, ..) => {
                    ParentBlobChunkFill::Full(
                        <<usize as ::rkyv::Archive>::Archived as ::rkyv::Deserialize<
                            usize,
                            __D,
                        >>::deserialize(_0, deserializer)?,
                    )
                }
                Self::Partial(_0, ..) => {
                    ParentBlobChunkFill::Partial(
                        <<usize as ::rkyv::Archive>::Archived as ::rkyv::Deserialize<
                            usize,
                            __D,
                        >>::deserialize(_0, deserializer)?,
                    )
                }
                Self::Corrupted(_0, ..) => {
                    ParentBlobChunkFill::Corrupted(
                        <<usize as ::rkyv::Archive>::Archived as ::rkyv::Deserialize<
                            usize,
                            __D,
                        >>::deserialize(_0, deserializer)?,
                    )
                }
            },
        )
    }
}
impl ParentBlobChunkFill {
    pub fn from_size(actual: usize, expected: usize) -> Self {
        if actual == expected {
            Self::Full(actual)
        } else if actual < expected {
            Self::Partial(actual)
        } else {
            Self::Corrupted(actual)
        }
    }
}
impl rewrite::traits::structural::blob::BlobItemChunk for ParentBlobChunk {
    type Index = usize;
    fn get_index(&self) -> &Self::Index {
        match self {
            Self::Child(c) => c.get_index(),
            _ => {
                ::core::panicking::panic_fmt(
                    format_args!("get_index called on missing chunk"),
                );
            }
        }
    }
}
impl rewrite::traits::structural::blob::NetabaseBlobItem for ParentBlob {
    type Chunk = ParentBlobChunk;
    type BlobIter = std::vec::IntoIter<rewrite::results::NetabaseResult<Self::Chunk>>;
    const DEFAULT_CHUNK_SIZE: usize = 0usize;
    fn into_chunks(
        self,
        size: rewrite::traits::structural::blob::ChunkSize,
    ) -> Box<dyn Iterator<Item = Self::Chunk>> {
        Box::new(self.into_chunks_iter(size).filter_map(|r| r.ok()))
    }
    fn into_chunks_iter(
        self,
        size: rewrite::traits::structural::blob::ChunkSize,
    ) -> Self::BlobIter {
        let mut all_chunks = Vec::new();
        {
            let serialized_field: rewrite::results::NetabaseResult<Vec<u8>> = rkyv::to_bytes::<
                rkyv::rancor::Error,
            >(&self.child)
                .map_err(|e| rewrite::results::NetabaseError::Serialization(
                    ::alloc::__export::must_use({
                        ::alloc::fmt::format(
                            format_args!(
                                "rkyv serialization failed for field {0}: {1:?}",
                                "child",
                                e,
                            ),
                        )
                    }),
                ))
                .map(|d| d.to_vec());
            match serialized_field {
                Ok(data) => {
                    let chunk_size = match size {
                        rewrite::traits::structural::blob::ChunkSize::Default => {
                            let default = 128usize;
                            if default > 0 { default } else { 1024 }
                        }
                        rewrite::traits::structural::blob::ChunkSize::Size(n) => n,
                    };
                    all_chunks
                        .extend(
                            data
                                .chunks(chunk_size)
                                .enumerate()
                                .map(|(index, chunk_data)| {
                                    Ok(
                                        Self::Chunk::Child(ParentBlobChildChunk {
                                            index,
                                            data: chunk_data.to_vec(),
                                        }),
                                    )
                                }),
                        );
                }
                Err(e) => {
                    all_chunks.push(Err(e));
                }
            }
        }
        all_chunks.into_iter()
    }
    fn try_from_chunks(
        chunks: impl Iterator<Item = Self::Chunk>,
        size: rewrite::traits::structural::blob::ChunkSize,
    ) -> rewrite::results::NetabaseResult<Self> {
        let mut chunks_child = Vec::new();
        for chunk in chunks {
            match chunk {
                Self::Chunk::Child(c) => chunks_child.push(c),
                _ => {}
            }
        }
        let child = {
            if chunks_child.is_empty() {
                return Err(
                    rewrite::results::NetabaseError::BlobReconstruction(
                        rewrite::results::BlobReconstructionError::MissingChunks,
                    ),
                );
            }
            let mut sorted = chunks_child;
            sorted.sort_by_key(|c| c.index);
            let chunk_size = match size {
                rewrite::traits::structural::blob::ChunkSize::Default => {
                    let default = 128usize;
                    if default > 0 { default } else { 1024 }
                }
                rewrite::traits::structural::blob::ChunkSize::Size(n) => n,
            };
            let mut missing_details = Vec::new();
            let mut next_expected = 0;
            let max_idx = sorted.last().map(|c| c.index).unwrap_or(0);
            for chunk in &sorted {
                while chunk.index > next_expected {
                    missing_details
                        .push(
                            ::alloc::__export::must_use({
                                ::alloc::fmt::format(
                                    format_args!(
                                        "{0:?}({{ Index: {1}, Size: {2} }})",
                                        ParentBlobChunkFill::Full(chunk_size),
                                        next_expected,
                                        chunk_size,
                                    ),
                                )
                            }),
                        );
                    next_expected += 1;
                }
                let fill = ParentBlobChunkFill::from_size(chunk.data.len(), chunk_size);
                match fill {
                    ParentBlobChunkFill::Corrupted(size) => {
                        return Err(
                            rewrite::results::NetabaseError::BlobReconstruction(
                                rewrite::results::BlobReconstructionError::InvalidChunkData(
                                    ::alloc::__export::must_use({
                                        ::alloc::fmt::format(
                                            format_args!(
                                                "Corrupted chunk detected for field {0}: {1:?}({{ Index: {2}, Size: {3} }}). Max allowed size is {4}.",
                                                "child",
                                                fill,
                                                chunk.index,
                                                size,
                                                chunk_size,
                                            ),
                                        )
                                    }),
                                ),
                            ),
                        );
                    }
                    ParentBlobChunkFill::Partial(size) if chunk.index < max_idx => {
                        return Err(
                            rewrite::results::NetabaseError::BlobReconstruction(
                                rewrite::results::BlobReconstructionError::InvalidChunkData(
                                    ::alloc::__export::must_use({
                                        ::alloc::fmt::format(
                                            format_args!(
                                                "Unexpected partial chunk in middle of stream for field {0}: {1:?}({{ Index: {2}, Size: {3} }}). Expected {4} bytes.",
                                                "child",
                                                fill,
                                                chunk.index,
                                                size,
                                                chunk_size,
                                            ),
                                        )
                                    }),
                                ),
                            ),
                        );
                    }
                    _ => {}
                }
                if chunk.index == next_expected {
                    next_expected += 1;
                }
            }
            if !missing_details.is_empty() {
                if let Some(last) = sorted.last() {
                    let fill = ParentBlobChunkFill::from_size(
                        last.data.len(),
                        chunk_size,
                    );
                    if #[allow(non_exhaustive_omitted_patterns)]
                    match fill {
                        ParentBlobChunkFill::Full(_) => true,
                        _ => false,
                    } {
                        missing_details
                            .push(
                                ::alloc::__export::must_use({
                                    ::alloc::fmt::format(
                                        format_args!(
                                            "... (Stream truncated for field {0}: last chunk was Full, expected more data after Index {1})",
                                            "child",
                                            last.index,
                                        ),
                                    )
                                }),
                            );
                    }
                }
            }
            if !missing_details.is_empty() {
                return Err(
                    rewrite::results::NetabaseError::BlobReconstruction(
                        rewrite::results::BlobReconstructionError::InvalidChunkData(
                            ::alloc::__export::must_use({
                                ::alloc::fmt::format(
                                    format_args!(
                                        "Missing chunks for field {0}: [{1}]. Total chunks present: {2}",
                                        "child",
                                        missing_details.join(", "),
                                        sorted.len(),
                                    ),
                                )
                            }),
                        ),
                    ),
                );
            }
            let data: Vec<u8> = sorted.into_iter().flat_map(|c| c.data).collect();
            rkyv::from_bytes::<NestedBlob, rkyv::rancor::Error>(&data)
                .map_err(|e| rewrite::results::NetabaseError::Serialization(
                    ::alloc::__export::must_use({
                        ::alloc::fmt::format(
                            format_args!(
                                "rkyv deserialization failed for field {0}: {1:?}",
                                "child",
                                e,
                            ),
                        )
                    }),
                ))?
        };
        Ok(Self { child })
    }
    fn get_blob(&self) -> &Self::Chunk {
        {
            ::core::panicking::panic_fmt(
                format_args!(
                    "not implemented: {0}",
                    format_args!("get_blob() requires storing a chunk reference"),
                ),
            );
        }
    }
}
impl IntoIterator for ParentBlob {
    type Item = rewrite::results::NetabaseResult<ParentBlobChunk>;
    type IntoIter = std::vec::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        rewrite::traits::structural::blob::NetabaseBlobItem::into_chunks_iter(
            self,
            rewrite::traits::structural::blob::ChunkSize::Default,
        )
    }
}
enum EnumBlob {
    VariantA(String),
    VariantB { x: i32, y: i32 },
}
#[automatically_derived]
///An archived [`EnumBlob`]
#[bytecheck(crate = ::rkyv::bytecheck)]
#[repr(u8)]
enum ArchivedEnumBlob
where
    String: ::rkyv::Archive,
    i32: ::rkyv::Archive,
    i32: ::rkyv::Archive,
{
    ///The archived counterpart of [`EnumBlob::VariantA`]
    #[allow(dead_code)]
    VariantA(
        ///The archived counterpart of [`EnumBlob::VariantA::0`]
        <String as ::rkyv::Archive>::Archived,
    ),
    ///The archived counterpart of [`EnumBlob::VariantB`]
    #[allow(dead_code)]
    VariantB {
        ///The archived counterpart of [`EnumBlob::VariantB::x`]
        x: <i32 as ::rkyv::Archive>::Archived,
        ///The archived counterpart of [`EnumBlob::VariantB::y`]
        y: <i32 as ::rkyv::Archive>::Archived,
    },
}
const _: () = {
    #[repr(u8)]
    enum Tag {
        VariantA,
        VariantB,
    }
    struct Discriminant;
    #[automatically_derived]
    impl Discriminant {
        #[allow(non_upper_case_globals)]
        const VariantA: u8 = Tag::VariantA as u8;
        #[allow(non_upper_case_globals)]
        const VariantB: u8 = Tag::VariantB as u8;
    }
    #[repr(C)]
    struct VariantVariantA(
        Tag,
        <String as ::rkyv::Archive>::Archived,
        ::core::marker::PhantomData<ArchivedEnumBlob>,
    )
    where
        String: ::rkyv::Archive,
        i32: ::rkyv::Archive,
        i32: ::rkyv::Archive;
    #[repr(C)]
    struct VariantVariantB
    where
        String: ::rkyv::Archive,
        i32: ::rkyv::Archive,
        i32: ::rkyv::Archive,
    {
        __tag: Tag,
        x: <i32 as ::rkyv::Archive>::Archived,
        y: <i32 as ::rkyv::Archive>::Archived,
        __phantom: ::core::marker::PhantomData<ArchivedEnumBlob>,
    }
    #[automatically_derived]
    unsafe impl<
        __C: ::rkyv::bytecheck::rancor::Fallible + ?::core::marker::Sized,
    > ::rkyv::bytecheck::CheckBytes<__C> for ArchivedEnumBlob
    where
        String: ::rkyv::Archive,
        i32: ::rkyv::Archive,
        i32: ::rkyv::Archive,
        <__C as ::rkyv::bytecheck::rancor::Fallible>::Error: ::rkyv::bytecheck::rancor::Source,
        <String as ::rkyv::Archive>::Archived: ::rkyv::bytecheck::CheckBytes<__C>,
        <i32 as ::rkyv::Archive>::Archived: ::rkyv::bytecheck::CheckBytes<__C>,
        <i32 as ::rkyv::Archive>::Archived: ::rkyv::bytecheck::CheckBytes<__C>,
    {
        unsafe fn check_bytes(
            value: *const Self,
            context: &mut __C,
        ) -> ::core::result::Result<
            (),
            <__C as ::rkyv::bytecheck::rancor::Fallible>::Error,
        > {
            let tag = *value.cast::<u8>();
            match tag {
                Discriminant::VariantA => {
                    let value = value.cast::<VariantVariantA>();
                    <<String as ::rkyv::Archive>::Archived as ::rkyv::bytecheck::CheckBytes<
                        __C,
                    >>::check_bytes(&raw const (*value).1, context)
                        .map_err(|e| {
                            <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Trace>::trace(
                                e,
                                ::rkyv::bytecheck::UnnamedEnumVariantCheckContext {
                                    enum_name: "ArchivedEnumBlob",
                                    variant_name: "VariantA",
                                    field_index: 1,
                                },
                            )
                        })?;
                }
                Discriminant::VariantB => {
                    let value = value.cast::<VariantVariantB>();
                    <<i32 as ::rkyv::Archive>::Archived as ::rkyv::bytecheck::CheckBytes<
                        __C,
                    >>::check_bytes(&raw const (*value).x, context)
                        .map_err(|e| {
                            <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Trace>::trace(
                                e,
                                ::rkyv::bytecheck::NamedEnumVariantCheckContext {
                                    enum_name: "ArchivedEnumBlob",
                                    variant_name: "VariantB",
                                    field_name: "x",
                                },
                            )
                        })?;
                    <<i32 as ::rkyv::Archive>::Archived as ::rkyv::bytecheck::CheckBytes<
                        __C,
                    >>::check_bytes(&raw const (*value).y, context)
                        .map_err(|e| {
                            <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Trace>::trace(
                                e,
                                ::rkyv::bytecheck::NamedEnumVariantCheckContext {
                                    enum_name: "ArchivedEnumBlob",
                                    variant_name: "VariantB",
                                    field_name: "y",
                                },
                            )
                        })?;
                }
                _ => {
                    return ::core::result::Result::Err(
                        <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Source>::new(::rkyv::bytecheck::InvalidEnumDiscriminantError {
                            enum_name: "ArchivedEnumBlob",
                            invalid_discriminant: tag,
                        }),
                    );
                }
            }
            ::core::result::Result::Ok(())
        }
    }
};
#[automatically_derived]
///The resolver for an archived [`EnumBlob`]
enum EnumBlobResolver
where
    String: ::rkyv::Archive,
    i32: ::rkyv::Archive,
    i32: ::rkyv::Archive,
{
    ///The resolver for [`EnumBlob::VariantA`]
    #[allow(dead_code)]
    VariantA(<String as ::rkyv::Archive>::Resolver),
    ///The resolver for [`EnumBlob::VariantB`]
    #[allow(dead_code)]
    VariantB {
        x: <i32 as ::rkyv::Archive>::Resolver,
        y: <i32 as ::rkyv::Archive>::Resolver,
    },
}
const _: () = {
    #[repr(u8)]
    enum ArchivedTag {
        VariantA,
        VariantB,
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for ArchivedTag {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for ArchivedTag {
        #[inline]
        fn eq(&self, other: &ArchivedTag) -> bool {
            let __self_discr = ::core::intrinsics::discriminant_value(self);
            let __arg1_discr = ::core::intrinsics::discriminant_value(other);
            __self_discr == __arg1_discr
        }
    }
    #[automatically_derived]
    impl ::core::cmp::PartialOrd for ArchivedTag {
        #[inline]
        fn partial_cmp(
            &self,
            other: &ArchivedTag,
        ) -> ::core::option::Option<::core::cmp::Ordering> {
            let __self_discr = ::core::intrinsics::discriminant_value(self);
            let __arg1_discr = ::core::intrinsics::discriminant_value(other);
            ::core::cmp::PartialOrd::partial_cmp(&__self_discr, &__arg1_discr)
        }
    }
    #[repr(C)]
    struct ArchivedVariantVariantA(
        ArchivedTag,
        <String as ::rkyv::Archive>::Archived,
        ::core::marker::PhantomData<EnumBlob>,
    )
    where
        String: ::rkyv::Archive,
        i32: ::rkyv::Archive,
        i32: ::rkyv::Archive;
    #[repr(C)]
    struct ArchivedVariantVariantB
    where
        String: ::rkyv::Archive,
        i32: ::rkyv::Archive,
        i32: ::rkyv::Archive,
    {
        __tag: ArchivedTag,
        x: <i32 as ::rkyv::Archive>::Archived,
        y: <i32 as ::rkyv::Archive>::Archived,
        __phantom: ::core::marker::PhantomData<EnumBlob>,
    }
    impl ::rkyv::Archive for EnumBlob
    where
        String: ::rkyv::Archive,
        i32: ::rkyv::Archive,
        i32: ::rkyv::Archive,
    {
        type Archived = ArchivedEnumBlob;
        type Resolver = EnumBlobResolver;
        #[allow(clippy::unit_arg)]
        fn resolve(
            &self,
            resolver: <Self as ::rkyv::Archive>::Resolver,
            out: ::rkyv::Place<<Self as ::rkyv::Archive>::Archived>,
        ) {
            let __this = self;
            match resolver {
                EnumBlobResolver::VariantA(resolver_0) => {
                    match __this {
                        EnumBlob::VariantA(self_0, ..) => {
                            let out = unsafe {
                                out.cast_unchecked::<ArchivedVariantVariantA>()
                            };
                            let tag_ptr = unsafe { &raw mut (*out.ptr()).0 };
                            unsafe {
                                tag_ptr.write(ArchivedTag::VariantA);
                            }
                            let field_ptr = unsafe { &raw mut (*out.ptr()).1 };
                            let out_field = unsafe {
                                ::rkyv::Place::from_field_unchecked(out, field_ptr)
                            };
                            <String as ::rkyv::Archive>::resolve(
                                self_0,
                                resolver_0,
                                out_field,
                            );
                        }
                        #[allow(unreachable_patterns)]
                        _ => unsafe { ::core::hint::unreachable_unchecked() }
                    }
                }
                EnumBlobResolver::VariantB { x: resolver_0, y: resolver_1 } => {
                    match __this {
                        EnumBlob::VariantB { x: self_0, y: self_1, .. } => {
                            let out = unsafe {
                                out.cast_unchecked::<ArchivedVariantVariantB>()
                            };
                            let tag_ptr = unsafe { &raw mut (*out.ptr()).__tag };
                            unsafe {
                                tag_ptr.write(ArchivedTag::VariantB);
                            }
                            let field_ptr = unsafe { &raw mut (*out.ptr()).x };
                            let out_field = unsafe {
                                ::rkyv::Place::from_field_unchecked(out, field_ptr)
                            };
                            <i32 as ::rkyv::Archive>::resolve(
                                self_0,
                                resolver_0,
                                out_field,
                            );
                            let field_ptr = unsafe { &raw mut (*out.ptr()).y };
                            let out_field = unsafe {
                                ::rkyv::Place::from_field_unchecked(out, field_ptr)
                            };
                            <i32 as ::rkyv::Archive>::resolve(
                                self_1,
                                resolver_1,
                                out_field,
                            );
                        }
                        #[allow(unreachable_patterns)]
                        _ => unsafe { ::core::hint::unreachable_unchecked() }
                    }
                }
            }
        }
    }
};
unsafe impl ::rkyv::traits::Portable for ArchivedEnumBlob
where
    String: ::rkyv::Archive,
    i32: ::rkyv::Archive,
    i32: ::rkyv::Archive,
    <String as ::rkyv::Archive>::Archived: ::rkyv::traits::Portable,
    <i32 as ::rkyv::Archive>::Archived: ::rkyv::traits::Portable,
    <i32 as ::rkyv::Archive>::Archived: ::rkyv::traits::Portable,
{}
#[automatically_derived]
impl<__S: ::rkyv::rancor::Fallible + ?Sized> ::rkyv::Serialize<__S> for EnumBlob
where
    String: ::rkyv::Serialize<__S>,
    i32: ::rkyv::Serialize<__S>,
    i32: ::rkyv::Serialize<__S>,
{
    fn serialize(
        &self,
        serializer: &mut __S,
    ) -> ::core::result::Result<
        <Self as ::rkyv::Archive>::Resolver,
        <__S as ::rkyv::rancor::Fallible>::Error,
    > {
        let __this = self;
        ::core::result::Result::Ok(
            match __this {
                EnumBlob::VariantA(_0, ..) => {
                    EnumBlobResolver::VariantA(
                        <String as ::rkyv::Serialize<__S>>::serialize(_0, serializer)?,
                    )
                }
                EnumBlob::VariantB { x, y, .. } => {
                    EnumBlobResolver::VariantB {
                        x: <i32 as ::rkyv::Serialize<__S>>::serialize(x, serializer)?,
                        y: <i32 as ::rkyv::Serialize<__S>>::serialize(y, serializer)?,
                    }
                }
            },
        )
    }
}
#[automatically_derived]
impl<__D: ::rkyv::rancor::Fallible + ?Sized> ::rkyv::Deserialize<EnumBlob, __D>
for ::rkyv::Archived<EnumBlob>
where
    String: ::rkyv::Archive,
    <String as ::rkyv::Archive>::Archived: ::rkyv::Deserialize<String, __D>,
    i32: ::rkyv::Archive,
    <i32 as ::rkyv::Archive>::Archived: ::rkyv::Deserialize<i32, __D>,
    i32: ::rkyv::Archive,
    <i32 as ::rkyv::Archive>::Archived: ::rkyv::Deserialize<i32, __D>,
{
    fn deserialize(
        &self,
        deserializer: &mut __D,
    ) -> ::core::result::Result<EnumBlob, <__D as ::rkyv::rancor::Fallible>::Error> {
        let __this = self;
        ::core::result::Result::Ok(
            match __this {
                Self::VariantA(_0, ..) => {
                    EnumBlob::VariantA(
                        <<String as ::rkyv::Archive>::Archived as ::rkyv::Deserialize<
                            String,
                            __D,
                        >>::deserialize(_0, deserializer)?,
                    )
                }
                Self::VariantB { x, y, .. } => {
                    EnumBlob::VariantB {
                        x: <<i32 as ::rkyv::Archive>::Archived as ::rkyv::Deserialize<
                            i32,
                            __D,
                        >>::deserialize(x, deserializer)?,
                        y: <<i32 as ::rkyv::Archive>::Archived as ::rkyv::Deserialize<
                            i32,
                            __D,
                        >>::deserialize(y, deserializer)?,
                    }
                }
            },
        )
    }
}
#[automatically_derived]
impl ::core::fmt::Debug for EnumBlob {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match self {
            EnumBlob::VariantA(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(
                    f,
                    "VariantA",
                    &__self_0,
                )
            }
            EnumBlob::VariantB { x: __self_0, y: __self_1 } => {
                ::core::fmt::Formatter::debug_struct_field2_finish(
                    f,
                    "VariantB",
                    "x",
                    __self_0,
                    "y",
                    &__self_1,
                )
            }
        }
    }
}
#[automatically_derived]
impl ::core::clone::Clone for EnumBlob {
    #[inline]
    fn clone(&self) -> EnumBlob {
        match self {
            EnumBlob::VariantA(__self_0) => {
                EnumBlob::VariantA(::core::clone::Clone::clone(__self_0))
            }
            EnumBlob::VariantB { x: __self_0, y: __self_1 } => {
                EnumBlob::VariantB {
                    x: ::core::clone::Clone::clone(__self_0),
                    y: ::core::clone::Clone::clone(__self_1),
                }
            }
        }
    }
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for EnumBlob {}
#[automatically_derived]
impl ::core::cmp::PartialEq for EnumBlob {
    #[inline]
    fn eq(&self, other: &EnumBlob) -> bool {
        let __self_discr = ::core::intrinsics::discriminant_value(self);
        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
        __self_discr == __arg1_discr
            && match (self, other) {
                (EnumBlob::VariantA(__self_0), EnumBlob::VariantA(__arg1_0)) => {
                    __self_0 == __arg1_0
                }
                (
                    EnumBlob::VariantB { x: __self_0, y: __self_1 },
                    EnumBlob::VariantB { x: __arg1_0, y: __arg1_1 },
                ) => __self_0 == __arg1_0 && __self_1 == __arg1_1,
                _ => unsafe { ::core::intrinsics::unreachable() }
            }
    }
}
#[automatically_derived]
impl ::core::cmp::Eq for EnumBlob {
    #[inline]
    #[doc(hidden)]
    #[coverage(off)]
    fn assert_fields_are_eq(&self) {
        let _: ::core::cmp::AssertParamIsEq<String>;
        let _: ::core::cmp::AssertParamIsEq<i32>;
    }
}
pub struct EnumBlobChunk {
    pub index: usize,
    pub data: Vec<u8>,
}
#[automatically_derived]
impl ::core::fmt::Debug for EnumBlobChunk {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::debug_struct_field2_finish(
            f,
            "EnumBlobChunk",
            "index",
            &self.index,
            "data",
            &&self.data,
        )
    }
}
#[automatically_derived]
impl ::core::clone::Clone for EnumBlobChunk {
    #[inline]
    fn clone(&self) -> EnumBlobChunk {
        EnumBlobChunk {
            index: ::core::clone::Clone::clone(&self.index),
            data: ::core::clone::Clone::clone(&self.data),
        }
    }
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for EnumBlobChunk {}
#[automatically_derived]
impl ::core::cmp::PartialEq for EnumBlobChunk {
    #[inline]
    fn eq(&self, other: &EnumBlobChunk) -> bool {
        self.index == other.index && self.data == other.data
    }
}
#[automatically_derived]
impl ::core::cmp::Eq for EnumBlobChunk {
    #[inline]
    #[doc(hidden)]
    #[coverage(off)]
    fn assert_fields_are_eq(&self) {
        let _: ::core::cmp::AssertParamIsEq<usize>;
        let _: ::core::cmp::AssertParamIsEq<Vec<u8>>;
    }
}
#[automatically_derived]
impl ::core::cmp::PartialOrd for EnumBlobChunk {
    #[inline]
    fn partial_cmp(
        &self,
        other: &EnumBlobChunk,
    ) -> ::core::option::Option<::core::cmp::Ordering> {
        match ::core::cmp::PartialOrd::partial_cmp(&self.index, &other.index) {
            ::core::option::Option::Some(::core::cmp::Ordering::Equal) => {
                ::core::cmp::PartialOrd::partial_cmp(&self.data, &other.data)
            }
            cmp => cmp,
        }
    }
}
#[automatically_derived]
impl ::core::cmp::Ord for EnumBlobChunk {
    #[inline]
    fn cmp(&self, other: &EnumBlobChunk) -> ::core::cmp::Ordering {
        match ::core::cmp::Ord::cmp(&self.index, &other.index) {
            ::core::cmp::Ordering::Equal => {
                ::core::cmp::Ord::cmp(&self.data, &other.data)
            }
            cmp => cmp,
        }
    }
}
#[automatically_derived]
///An archived [`EnumBlobChunk`]
#[bytecheck(crate = ::rkyv::bytecheck)]
#[repr(C)]
pub struct ArchivedEnumBlobChunk
where
    usize: ::rkyv::Archive,
    Vec<u8>: ::rkyv::Archive,
{
    ///The archived counterpart of [`EnumBlobChunk::index`]
    pub index: <usize as ::rkyv::Archive>::Archived,
    ///The archived counterpart of [`EnumBlobChunk::data`]
    pub data: <Vec<u8> as ::rkyv::Archive>::Archived,
}
#[automatically_derived]
unsafe impl<
    __C: ::rkyv::bytecheck::rancor::Fallible + ?::core::marker::Sized,
> ::rkyv::bytecheck::CheckBytes<__C> for ArchivedEnumBlobChunk
where
    usize: ::rkyv::Archive,
    Vec<u8>: ::rkyv::Archive,
    <__C as ::rkyv::bytecheck::rancor::Fallible>::Error: ::rkyv::bytecheck::rancor::Trace,
    <usize as ::rkyv::Archive>::Archived: ::rkyv::bytecheck::CheckBytes<__C>,
    <Vec<u8> as ::rkyv::Archive>::Archived: ::rkyv::bytecheck::CheckBytes<__C>,
{
    unsafe fn check_bytes(
        value: *const Self,
        context: &mut __C,
    ) -> ::core::result::Result<
        (),
        <__C as ::rkyv::bytecheck::rancor::Fallible>::Error,
    > {
        <<usize as ::rkyv::Archive>::Archived as ::rkyv::bytecheck::CheckBytes<
            __C,
        >>::check_bytes(&raw const (*value).index, context)
            .map_err(|e| {
                <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Trace>::trace(
                    e,
                    ::rkyv::bytecheck::StructCheckContext {
                        struct_name: "ArchivedEnumBlobChunk",
                        field_name: "index",
                    },
                )
            })?;
        <<Vec<
            u8,
        > as ::rkyv::Archive>::Archived as ::rkyv::bytecheck::CheckBytes<
            __C,
        >>::check_bytes(&raw const (*value).data, context)
            .map_err(|e| {
                <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Trace>::trace(
                    e,
                    ::rkyv::bytecheck::StructCheckContext {
                        struct_name: "ArchivedEnumBlobChunk",
                        field_name: "data",
                    },
                )
            })?;
        ::core::result::Result::Ok(())
    }
}
#[automatically_derived]
///The resolver for an archived [`EnumBlobChunk`]
pub struct EnumBlobChunkResolver
where
    usize: ::rkyv::Archive,
    Vec<u8>: ::rkyv::Archive,
{
    index: <usize as ::rkyv::Archive>::Resolver,
    data: <Vec<u8> as ::rkyv::Archive>::Resolver,
}
impl ::rkyv::Archive for EnumBlobChunk
where
    usize: ::rkyv::Archive,
    Vec<u8>: ::rkyv::Archive,
{
    type Archived = ArchivedEnumBlobChunk;
    type Resolver = EnumBlobChunkResolver;
    const COPY_OPTIMIZATION: ::rkyv::traits::CopyOptimization<Self> = unsafe {
        ::rkyv::traits::CopyOptimization::enable_if(
            0 + ::core::mem::size_of::<usize>() + ::core::mem::size_of::<Vec<u8>>()
                == ::core::mem::size_of::<EnumBlobChunk>()
                && <usize as ::rkyv::Archive>::COPY_OPTIMIZATION.is_enabled()
                && const { builtin # offset_of(EnumBlobChunk, index) }
                    == const { builtin # offset_of(ArchivedEnumBlobChunk, index) }
                && <Vec<u8> as ::rkyv::Archive>::COPY_OPTIMIZATION.is_enabled()
                && const { builtin # offset_of(EnumBlobChunk, data) }
                    == const { builtin # offset_of(ArchivedEnumBlobChunk, data) },
        )
    };
    #[allow(clippy::unit_arg)]
    fn resolve(&self, resolver: Self::Resolver, out: ::rkyv::Place<Self::Archived>) {
        let field_ptr = unsafe { &raw mut (*out.ptr()).index };
        let field_out = unsafe { ::rkyv::Place::from_field_unchecked(out, field_ptr) };
        <usize as ::rkyv::Archive>::resolve(&self.index, resolver.index, field_out);
        let field_ptr = unsafe { &raw mut (*out.ptr()).data };
        let field_out = unsafe { ::rkyv::Place::from_field_unchecked(out, field_ptr) };
        <Vec<u8> as ::rkyv::Archive>::resolve(&self.data, resolver.data, field_out);
    }
}
unsafe impl ::rkyv::traits::Portable for ArchivedEnumBlobChunk
where
    usize: ::rkyv::Archive,
    Vec<u8>: ::rkyv::Archive,
    <usize as ::rkyv::Archive>::Archived: ::rkyv::traits::Portable,
    <Vec<u8> as ::rkyv::Archive>::Archived: ::rkyv::traits::Portable,
{}
#[automatically_derived]
impl<__S: ::rkyv::rancor::Fallible + ?Sized> ::rkyv::Serialize<__S> for EnumBlobChunk
where
    usize: ::rkyv::Serialize<__S>,
    Vec<u8>: ::rkyv::Serialize<__S>,
{
    fn serialize(
        &self,
        serializer: &mut __S,
    ) -> ::core::result::Result<
        <Self as ::rkyv::Archive>::Resolver,
        <__S as ::rkyv::rancor::Fallible>::Error,
    > {
        let __this = self;
        ::core::result::Result::Ok(EnumBlobChunkResolver {
            index: <usize as ::rkyv::Serialize<
                __S,
            >>::serialize(&__this.index, serializer)?,
            data: <Vec<
                u8,
            > as ::rkyv::Serialize<__S>>::serialize(&__this.data, serializer)?,
        })
    }
}
#[automatically_derived]
impl<__D: ::rkyv::rancor::Fallible + ?Sized> ::rkyv::Deserialize<EnumBlobChunk, __D>
for ::rkyv::Archived<EnumBlobChunk>
where
    usize: ::rkyv::Archive,
    <usize as ::rkyv::Archive>::Archived: ::rkyv::Deserialize<usize, __D>,
    Vec<u8>: ::rkyv::Archive,
    <Vec<u8> as ::rkyv::Archive>::Archived: ::rkyv::Deserialize<Vec<u8>, __D>,
{
    fn deserialize(
        &self,
        deserializer: &mut __D,
    ) -> ::core::result::Result<
        EnumBlobChunk,
        <__D as ::rkyv::rancor::Fallible>::Error,
    > {
        let __this = self;
        ::core::result::Result::Ok(EnumBlobChunk {
            index: <<usize as ::rkyv::Archive>::Archived as ::rkyv::Deserialize<
                usize,
                __D,
            >>::deserialize(&__this.index, deserializer)?,
            data: <<Vec<
                u8,
            > as ::rkyv::Archive>::Archived as ::rkyv::Deserialize<
                Vec<u8>,
                __D,
            >>::deserialize(&__this.data, deserializer)?,
        })
    }
}
impl ::rewrite::traits::structural::blob::BlobItemChunk for EnumBlobChunk {
    type Index = usize;
    fn get_index(&self) -> &Self::Index {
        &self.index
    }
}
pub enum EnumBlobChunkFill {
    Full(usize),
    Partial(usize),
    Corrupted(usize),
}
#[automatically_derived]
impl ::core::fmt::Debug for EnumBlobChunkFill {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match self {
            EnumBlobChunkFill::Full(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "Full", &__self_0)
            }
            EnumBlobChunkFill::Partial(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(
                    f,
                    "Partial",
                    &__self_0,
                )
            }
            EnumBlobChunkFill::Corrupted(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(
                    f,
                    "Corrupted",
                    &__self_0,
                )
            }
        }
    }
}
#[automatically_derived]
#[doc(hidden)]
unsafe impl ::core::clone::TrivialClone for EnumBlobChunkFill {}
#[automatically_derived]
impl ::core::clone::Clone for EnumBlobChunkFill {
    #[inline]
    fn clone(&self) -> EnumBlobChunkFill {
        let _: ::core::clone::AssertParamIsClone<usize>;
        *self
    }
}
#[automatically_derived]
impl ::core::marker::Copy for EnumBlobChunkFill {}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for EnumBlobChunkFill {}
#[automatically_derived]
impl ::core::cmp::PartialEq for EnumBlobChunkFill {
    #[inline]
    fn eq(&self, other: &EnumBlobChunkFill) -> bool {
        let __self_discr = ::core::intrinsics::discriminant_value(self);
        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
        __self_discr == __arg1_discr
            && match (self, other) {
                (
                    EnumBlobChunkFill::Full(__self_0),
                    EnumBlobChunkFill::Full(__arg1_0),
                ) => __self_0 == __arg1_0,
                (
                    EnumBlobChunkFill::Partial(__self_0),
                    EnumBlobChunkFill::Partial(__arg1_0),
                ) => __self_0 == __arg1_0,
                (
                    EnumBlobChunkFill::Corrupted(__self_0),
                    EnumBlobChunkFill::Corrupted(__arg1_0),
                ) => __self_0 == __arg1_0,
                _ => unsafe { ::core::intrinsics::unreachable() }
            }
    }
}
#[automatically_derived]
impl ::core::cmp::Eq for EnumBlobChunkFill {
    #[inline]
    #[doc(hidden)]
    #[coverage(off)]
    fn assert_fields_are_eq(&self) {
        let _: ::core::cmp::AssertParamIsEq<usize>;
    }
}
#[automatically_derived]
impl ::core::cmp::PartialOrd for EnumBlobChunkFill {
    #[inline]
    fn partial_cmp(
        &self,
        other: &EnumBlobChunkFill,
    ) -> ::core::option::Option<::core::cmp::Ordering> {
        let __self_discr = ::core::intrinsics::discriminant_value(self);
        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
        match (self, other) {
            (EnumBlobChunkFill::Full(__self_0), EnumBlobChunkFill::Full(__arg1_0)) => {
                ::core::cmp::PartialOrd::partial_cmp(__self_0, __arg1_0)
            }
            (
                EnumBlobChunkFill::Partial(__self_0),
                EnumBlobChunkFill::Partial(__arg1_0),
            ) => ::core::cmp::PartialOrd::partial_cmp(__self_0, __arg1_0),
            (
                EnumBlobChunkFill::Corrupted(__self_0),
                EnumBlobChunkFill::Corrupted(__arg1_0),
            ) => ::core::cmp::PartialOrd::partial_cmp(__self_0, __arg1_0),
            _ => ::core::cmp::PartialOrd::partial_cmp(&__self_discr, &__arg1_discr),
        }
    }
}
#[automatically_derived]
impl ::core::cmp::Ord for EnumBlobChunkFill {
    #[inline]
    fn cmp(&self, other: &EnumBlobChunkFill) -> ::core::cmp::Ordering {
        let __self_discr = ::core::intrinsics::discriminant_value(self);
        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
        match ::core::cmp::Ord::cmp(&__self_discr, &__arg1_discr) {
            ::core::cmp::Ordering::Equal => {
                match (self, other) {
                    (
                        EnumBlobChunkFill::Full(__self_0),
                        EnumBlobChunkFill::Full(__arg1_0),
                    ) => ::core::cmp::Ord::cmp(__self_0, __arg1_0),
                    (
                        EnumBlobChunkFill::Partial(__self_0),
                        EnumBlobChunkFill::Partial(__arg1_0),
                    ) => ::core::cmp::Ord::cmp(__self_0, __arg1_0),
                    (
                        EnumBlobChunkFill::Corrupted(__self_0),
                        EnumBlobChunkFill::Corrupted(__arg1_0),
                    ) => ::core::cmp::Ord::cmp(__self_0, __arg1_0),
                    _ => unsafe { ::core::intrinsics::unreachable() }
                }
            }
            cmp => cmp,
        }
    }
}
#[automatically_derived]
///An archived [`EnumBlobChunkFill`]
#[bytecheck(crate = ::rkyv::bytecheck)]
#[repr(u8)]
pub enum ArchivedEnumBlobChunkFill
where
    usize: ::rkyv::Archive,
    usize: ::rkyv::Archive,
    usize: ::rkyv::Archive,
{
    ///The archived counterpart of [`EnumBlobChunkFill::Full`]
    #[allow(dead_code)]
    Full(
        ///The archived counterpart of [`EnumBlobChunkFill::Full::0`]
        <usize as ::rkyv::Archive>::Archived,
    ),
    ///The archived counterpart of [`EnumBlobChunkFill::Partial`]
    #[allow(dead_code)]
    Partial(
        ///The archived counterpart of [`EnumBlobChunkFill::Partial::0`]
        <usize as ::rkyv::Archive>::Archived,
    ),
    ///The archived counterpart of [`EnumBlobChunkFill::Corrupted`]
    #[allow(dead_code)]
    Corrupted(
        ///The archived counterpart of [`EnumBlobChunkFill::Corrupted::0`]
        <usize as ::rkyv::Archive>::Archived,
    ),
}
const _: () = {
    #[repr(u8)]
    enum Tag {
        Full,
        Partial,
        Corrupted,
    }
    struct Discriminant;
    #[automatically_derived]
    impl Discriminant {
        #[allow(non_upper_case_globals)]
        const Full: u8 = Tag::Full as u8;
        #[allow(non_upper_case_globals)]
        const Partial: u8 = Tag::Partial as u8;
        #[allow(non_upper_case_globals)]
        const Corrupted: u8 = Tag::Corrupted as u8;
    }
    #[repr(C)]
    struct VariantFull(
        Tag,
        <usize as ::rkyv::Archive>::Archived,
        ::core::marker::PhantomData<ArchivedEnumBlobChunkFill>,
    )
    where
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive;
    #[repr(C)]
    struct VariantPartial(
        Tag,
        <usize as ::rkyv::Archive>::Archived,
        ::core::marker::PhantomData<ArchivedEnumBlobChunkFill>,
    )
    where
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive;
    #[repr(C)]
    struct VariantCorrupted(
        Tag,
        <usize as ::rkyv::Archive>::Archived,
        ::core::marker::PhantomData<ArchivedEnumBlobChunkFill>,
    )
    where
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive;
    #[automatically_derived]
    unsafe impl<
        __C: ::rkyv::bytecheck::rancor::Fallible + ?::core::marker::Sized,
    > ::rkyv::bytecheck::CheckBytes<__C> for ArchivedEnumBlobChunkFill
    where
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive,
        <__C as ::rkyv::bytecheck::rancor::Fallible>::Error: ::rkyv::bytecheck::rancor::Source,
        <usize as ::rkyv::Archive>::Archived: ::rkyv::bytecheck::CheckBytes<__C>,
        <usize as ::rkyv::Archive>::Archived: ::rkyv::bytecheck::CheckBytes<__C>,
        <usize as ::rkyv::Archive>::Archived: ::rkyv::bytecheck::CheckBytes<__C>,
    {
        unsafe fn check_bytes(
            value: *const Self,
            context: &mut __C,
        ) -> ::core::result::Result<
            (),
            <__C as ::rkyv::bytecheck::rancor::Fallible>::Error,
        > {
            let tag = *value.cast::<u8>();
            match tag {
                Discriminant::Full => {
                    let value = value.cast::<VariantFull>();
                    <<usize as ::rkyv::Archive>::Archived as ::rkyv::bytecheck::CheckBytes<
                        __C,
                    >>::check_bytes(&raw const (*value).1, context)
                        .map_err(|e| {
                            <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Trace>::trace(
                                e,
                                ::rkyv::bytecheck::UnnamedEnumVariantCheckContext {
                                    enum_name: "ArchivedEnumBlobChunkFill",
                                    variant_name: "Full",
                                    field_index: 1,
                                },
                            )
                        })?;
                }
                Discriminant::Partial => {
                    let value = value.cast::<VariantPartial>();
                    <<usize as ::rkyv::Archive>::Archived as ::rkyv::bytecheck::CheckBytes<
                        __C,
                    >>::check_bytes(&raw const (*value).1, context)
                        .map_err(|e| {
                            <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Trace>::trace(
                                e,
                                ::rkyv::bytecheck::UnnamedEnumVariantCheckContext {
                                    enum_name: "ArchivedEnumBlobChunkFill",
                                    variant_name: "Partial",
                                    field_index: 1,
                                },
                            )
                        })?;
                }
                Discriminant::Corrupted => {
                    let value = value.cast::<VariantCorrupted>();
                    <<usize as ::rkyv::Archive>::Archived as ::rkyv::bytecheck::CheckBytes<
                        __C,
                    >>::check_bytes(&raw const (*value).1, context)
                        .map_err(|e| {
                            <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Trace>::trace(
                                e,
                                ::rkyv::bytecheck::UnnamedEnumVariantCheckContext {
                                    enum_name: "ArchivedEnumBlobChunkFill",
                                    variant_name: "Corrupted",
                                    field_index: 1,
                                },
                            )
                        })?;
                }
                _ => {
                    return ::core::result::Result::Err(
                        <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Source>::new(::rkyv::bytecheck::InvalidEnumDiscriminantError {
                            enum_name: "ArchivedEnumBlobChunkFill",
                            invalid_discriminant: tag,
                        }),
                    );
                }
            }
            ::core::result::Result::Ok(())
        }
    }
};
#[automatically_derived]
///The resolver for an archived [`EnumBlobChunkFill`]
pub enum EnumBlobChunkFillResolver
where
    usize: ::rkyv::Archive,
    usize: ::rkyv::Archive,
    usize: ::rkyv::Archive,
{
    ///The resolver for [`EnumBlobChunkFill::Full`]
    #[allow(dead_code)]
    Full(<usize as ::rkyv::Archive>::Resolver),
    ///The resolver for [`EnumBlobChunkFill::Partial`]
    #[allow(dead_code)]
    Partial(<usize as ::rkyv::Archive>::Resolver),
    ///The resolver for [`EnumBlobChunkFill::Corrupted`]
    #[allow(dead_code)]
    Corrupted(<usize as ::rkyv::Archive>::Resolver),
}
const _: () = {
    #[repr(u8)]
    enum ArchivedTag {
        Full,
        Partial,
        Corrupted,
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for ArchivedTag {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for ArchivedTag {
        #[inline]
        fn eq(&self, other: &ArchivedTag) -> bool {
            let __self_discr = ::core::intrinsics::discriminant_value(self);
            let __arg1_discr = ::core::intrinsics::discriminant_value(other);
            __self_discr == __arg1_discr
        }
    }
    #[automatically_derived]
    impl ::core::cmp::PartialOrd for ArchivedTag {
        #[inline]
        fn partial_cmp(
            &self,
            other: &ArchivedTag,
        ) -> ::core::option::Option<::core::cmp::Ordering> {
            let __self_discr = ::core::intrinsics::discriminant_value(self);
            let __arg1_discr = ::core::intrinsics::discriminant_value(other);
            ::core::cmp::PartialOrd::partial_cmp(&__self_discr, &__arg1_discr)
        }
    }
    #[repr(C)]
    struct ArchivedVariantFull(
        ArchivedTag,
        <usize as ::rkyv::Archive>::Archived,
        ::core::marker::PhantomData<EnumBlobChunkFill>,
    )
    where
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive;
    #[repr(C)]
    struct ArchivedVariantPartial(
        ArchivedTag,
        <usize as ::rkyv::Archive>::Archived,
        ::core::marker::PhantomData<EnumBlobChunkFill>,
    )
    where
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive;
    #[repr(C)]
    struct ArchivedVariantCorrupted(
        ArchivedTag,
        <usize as ::rkyv::Archive>::Archived,
        ::core::marker::PhantomData<EnumBlobChunkFill>,
    )
    where
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive;
    impl ::rkyv::Archive for EnumBlobChunkFill
    where
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive,
    {
        type Archived = ArchivedEnumBlobChunkFill;
        type Resolver = EnumBlobChunkFillResolver;
        #[allow(clippy::unit_arg)]
        fn resolve(
            &self,
            resolver: <Self as ::rkyv::Archive>::Resolver,
            out: ::rkyv::Place<<Self as ::rkyv::Archive>::Archived>,
        ) {
            let __this = self;
            match resolver {
                EnumBlobChunkFillResolver::Full(resolver_0) => {
                    match __this {
                        EnumBlobChunkFill::Full(self_0, ..) => {
                            let out = unsafe {
                                out.cast_unchecked::<ArchivedVariantFull>()
                            };
                            let tag_ptr = unsafe { &raw mut (*out.ptr()).0 };
                            unsafe {
                                tag_ptr.write(ArchivedTag::Full);
                            }
                            let field_ptr = unsafe { &raw mut (*out.ptr()).1 };
                            let out_field = unsafe {
                                ::rkyv::Place::from_field_unchecked(out, field_ptr)
                            };
                            <usize as ::rkyv::Archive>::resolve(
                                self_0,
                                resolver_0,
                                out_field,
                            );
                        }
                        #[allow(unreachable_patterns)]
                        _ => unsafe { ::core::hint::unreachable_unchecked() }
                    }
                }
                EnumBlobChunkFillResolver::Partial(resolver_0) => {
                    match __this {
                        EnumBlobChunkFill::Partial(self_0, ..) => {
                            let out = unsafe {
                                out.cast_unchecked::<ArchivedVariantPartial>()
                            };
                            let tag_ptr = unsafe { &raw mut (*out.ptr()).0 };
                            unsafe {
                                tag_ptr.write(ArchivedTag::Partial);
                            }
                            let field_ptr = unsafe { &raw mut (*out.ptr()).1 };
                            let out_field = unsafe {
                                ::rkyv::Place::from_field_unchecked(out, field_ptr)
                            };
                            <usize as ::rkyv::Archive>::resolve(
                                self_0,
                                resolver_0,
                                out_field,
                            );
                        }
                        #[allow(unreachable_patterns)]
                        _ => unsafe { ::core::hint::unreachable_unchecked() }
                    }
                }
                EnumBlobChunkFillResolver::Corrupted(resolver_0) => {
                    match __this {
                        EnumBlobChunkFill::Corrupted(self_0, ..) => {
                            let out = unsafe {
                                out.cast_unchecked::<ArchivedVariantCorrupted>()
                            };
                            let tag_ptr = unsafe { &raw mut (*out.ptr()).0 };
                            unsafe {
                                tag_ptr.write(ArchivedTag::Corrupted);
                            }
                            let field_ptr = unsafe { &raw mut (*out.ptr()).1 };
                            let out_field = unsafe {
                                ::rkyv::Place::from_field_unchecked(out, field_ptr)
                            };
                            <usize as ::rkyv::Archive>::resolve(
                                self_0,
                                resolver_0,
                                out_field,
                            );
                        }
                        #[allow(unreachable_patterns)]
                        _ => unsafe { ::core::hint::unreachable_unchecked() }
                    }
                }
            }
        }
    }
};
unsafe impl ::rkyv::traits::Portable for ArchivedEnumBlobChunkFill
where
    usize: ::rkyv::Archive,
    usize: ::rkyv::Archive,
    usize: ::rkyv::Archive,
    <usize as ::rkyv::Archive>::Archived: ::rkyv::traits::Portable,
    <usize as ::rkyv::Archive>::Archived: ::rkyv::traits::Portable,
    <usize as ::rkyv::Archive>::Archived: ::rkyv::traits::Portable,
{}
#[automatically_derived]
impl<__S: ::rkyv::rancor::Fallible + ?Sized> ::rkyv::Serialize<__S> for EnumBlobChunkFill
where
    usize: ::rkyv::Serialize<__S>,
    usize: ::rkyv::Serialize<__S>,
    usize: ::rkyv::Serialize<__S>,
{
    fn serialize(
        &self,
        serializer: &mut __S,
    ) -> ::core::result::Result<
        <Self as ::rkyv::Archive>::Resolver,
        <__S as ::rkyv::rancor::Fallible>::Error,
    > {
        let __this = self;
        ::core::result::Result::Ok(
            match __this {
                EnumBlobChunkFill::Full(_0, ..) => {
                    EnumBlobChunkFillResolver::Full(
                        <usize as ::rkyv::Serialize<__S>>::serialize(_0, serializer)?,
                    )
                }
                EnumBlobChunkFill::Partial(_0, ..) => {
                    EnumBlobChunkFillResolver::Partial(
                        <usize as ::rkyv::Serialize<__S>>::serialize(_0, serializer)?,
                    )
                }
                EnumBlobChunkFill::Corrupted(_0, ..) => {
                    EnumBlobChunkFillResolver::Corrupted(
                        <usize as ::rkyv::Serialize<__S>>::serialize(_0, serializer)?,
                    )
                }
            },
        )
    }
}
#[automatically_derived]
impl<__D: ::rkyv::rancor::Fallible + ?Sized> ::rkyv::Deserialize<EnumBlobChunkFill, __D>
for ::rkyv::Archived<EnumBlobChunkFill>
where
    usize: ::rkyv::Archive,
    <usize as ::rkyv::Archive>::Archived: ::rkyv::Deserialize<usize, __D>,
    usize: ::rkyv::Archive,
    <usize as ::rkyv::Archive>::Archived: ::rkyv::Deserialize<usize, __D>,
    usize: ::rkyv::Archive,
    <usize as ::rkyv::Archive>::Archived: ::rkyv::Deserialize<usize, __D>,
{
    fn deserialize(
        &self,
        deserializer: &mut __D,
    ) -> ::core::result::Result<
        EnumBlobChunkFill,
        <__D as ::rkyv::rancor::Fallible>::Error,
    > {
        let __this = self;
        ::core::result::Result::Ok(
            match __this {
                Self::Full(_0, ..) => {
                    EnumBlobChunkFill::Full(
                        <<usize as ::rkyv::Archive>::Archived as ::rkyv::Deserialize<
                            usize,
                            __D,
                        >>::deserialize(_0, deserializer)?,
                    )
                }
                Self::Partial(_0, ..) => {
                    EnumBlobChunkFill::Partial(
                        <<usize as ::rkyv::Archive>::Archived as ::rkyv::Deserialize<
                            usize,
                            __D,
                        >>::deserialize(_0, deserializer)?,
                    )
                }
                Self::Corrupted(_0, ..) => {
                    EnumBlobChunkFill::Corrupted(
                        <<usize as ::rkyv::Archive>::Archived as ::rkyv::Deserialize<
                            usize,
                            __D,
                        >>::deserialize(_0, deserializer)?,
                    )
                }
            },
        )
    }
}
impl EnumBlobChunkFill {
    pub fn from_size(actual: usize, expected: usize) -> Self {
        if actual == expected {
            Self::Full(actual)
        } else if actual < expected {
            Self::Partial(actual)
        } else {
            Self::Corrupted(actual)
        }
    }
}
impl rewrite::traits::structural::blob::NetabaseBlobItem for EnumBlob {
    type Chunk = EnumBlobChunk;
    type BlobIter = std::vec::IntoIter<rewrite::results::NetabaseResult<Self::Chunk>>;
    const DEFAULT_CHUNK_SIZE: usize = 0usize;
    fn into_chunks(
        self,
        size: rewrite::traits::structural::blob::ChunkSize,
    ) -> Box<dyn Iterator<Item = Self::Chunk>> {
        Box::new(self.into_chunks_iter(size).filter_map(|r| r.ok()))
    }
    fn into_chunks_iter(
        self,
        size: rewrite::traits::structural::blob::ChunkSize,
    ) -> Self::BlobIter {
        let serialized_data: rewrite::results::NetabaseResult<Vec<u8>> = (|| -> rewrite::results::NetabaseResult<
            Vec<u8>,
        > {
            Ok(
                rkyv::to_bytes::<rkyv::rancor::Error>(&self)
                    .map_err(|e| rewrite::results::NetabaseError::Serialization(
                        ::alloc::__export::must_use({
                            ::alloc::fmt::format(
                                format_args!("rkyv serialization failed: {0:?}", e),
                            )
                        }),
                    ))?
                    .to_vec(),
            )
        })();
        let chunk_size = match size {
            rewrite::traits::structural::blob::ChunkSize::Default => {
                if Self::DEFAULT_CHUNK_SIZE > 0 {
                    Self::DEFAULT_CHUNK_SIZE
                } else {
                    1024
                }
            }
            rewrite::traits::structural::blob::ChunkSize::Size(n) => n,
        };
        match serialized_data {
            Ok(data) => {
                data.chunks(chunk_size)
                    .enumerate()
                    .map(|(index, chunk_data)| {
                        Ok(Self::Chunk {
                            index,
                            data: chunk_data.to_vec(),
                        })
                    })
                    .collect::<Vec<_>>()
                    .into_iter()
            }
            Err(e) => {
                ::alloc::boxed::box_assume_init_into_vec_unsafe(
                        ::alloc::intrinsics::write_box_via_move(
                            ::alloc::boxed::Box::new_uninit(),
                            [Err(e)],
                        ),
                    )
                    .into_iter()
            }
        }
    }
    fn try_from_chunks(
        chunks: impl Iterator<Item = Self::Chunk>,
        size: rewrite::traits::structural::blob::ChunkSize,
    ) -> rewrite::results::NetabaseResult<Self> {
        let mut sorted_chunks: Vec<_> = chunks.collect();
        sorted_chunks.sort_by_key(|c| c.index);
        if sorted_chunks.is_empty() {
            return Err(
                rewrite::results::NetabaseError::BlobReconstruction(
                    rewrite::results::BlobReconstructionError::MissingChunks,
                ),
            );
        }
        let chunk_size = match size {
            rewrite::traits::structural::blob::ChunkSize::Default => {
                if Self::DEFAULT_CHUNK_SIZE > 0 {
                    Self::DEFAULT_CHUNK_SIZE
                } else {
                    1024
                }
            }
            rewrite::traits::structural::blob::ChunkSize::Size(n) => n,
        };
        let mut missing_details = Vec::new();
        let mut next_expected = 0;
        let max_idx = sorted_chunks.last().map(|c| c.index).unwrap_or(0);
        for chunk in &sorted_chunks {
            while chunk.index > next_expected {
                missing_details
                    .push(
                        ::alloc::__export::must_use({
                            ::alloc::fmt::format(
                                format_args!(
                                    "{0:?}({{ Index: {1}, Size: {2} }})",
                                    EnumBlobChunkFill::Full(chunk_size),
                                    next_expected,
                                    chunk_size,
                                ),
                            )
                        }),
                    );
                next_expected += 1;
            }
            let fill = EnumBlobChunkFill::from_size(chunk.data.len(), chunk_size);
            match fill {
                EnumBlobChunkFill::Corrupted(size) => {
                    return Err(
                        rewrite::results::NetabaseError::BlobReconstruction(
                            rewrite::results::BlobReconstructionError::InvalidChunkData(
                                ::alloc::__export::must_use({
                                    ::alloc::fmt::format(
                                        format_args!(
                                            "Corrupted chunk detected: {0:?}({{ Index: {1}, Size: {2} }}). Max allowed size is {3}.",
                                            fill,
                                            chunk.index,
                                            size,
                                            chunk_size,
                                        ),
                                    )
                                }),
                            ),
                        ),
                    );
                }
                EnumBlobChunkFill::Partial(size) if chunk.index < max_idx => {
                    return Err(
                        rewrite::results::NetabaseError::BlobReconstruction(
                            rewrite::results::BlobReconstructionError::InvalidChunkData(
                                ::alloc::__export::must_use({
                                    ::alloc::fmt::format(
                                        format_args!(
                                            "Unexpected partial chunk in middle of stream: {0:?}({{ Index: {1}, Size: {2} }}). Expected {3} bytes.",
                                            fill,
                                            chunk.index,
                                            size,
                                            chunk_size,
                                        ),
                                    )
                                }),
                            ),
                        ),
                    );
                }
                _ => {}
            }
            if chunk.index == next_expected {
                next_expected += 1;
            }
        }
        if !missing_details.is_empty() {
            if let Some(last) = sorted_chunks.last() {
                let fill = EnumBlobChunkFill::from_size(last.data.len(), chunk_size);
                if #[allow(non_exhaustive_omitted_patterns)]
                match fill {
                    EnumBlobChunkFill::Full(_) => true,
                    _ => false,
                } {
                    missing_details
                        .push(
                            ::alloc::__export::must_use({
                                ::alloc::fmt::format(
                                    format_args!(
                                        "... (Stream truncated: last chunk was Full, expected more data after Index {0})",
                                        last.index,
                                    ),
                                )
                            }),
                        );
                }
            }
        }
        if !missing_details.is_empty() {
            return Err(
                rewrite::results::NetabaseError::BlobReconstruction(
                    rewrite::results::BlobReconstructionError::InvalidChunkData(
                        ::alloc::__export::must_use({
                            ::alloc::fmt::format(
                                format_args!(
                                    "Missing chunks: [{0}]. Total chunks present: {1}",
                                    missing_details.join(", "),
                                    sorted_chunks.len(),
                                ),
                            )
                        }),
                    ),
                ),
            );
        }
        let serialized_data: Vec<u8> = sorted_chunks
            .into_iter()
            .flat_map(|c| c.data)
            .collect();
        Ok(
            rkyv::from_bytes::<Self, rkyv::rancor::Error>(&serialized_data)
                .map_err(|e| rewrite::results::NetabaseError::Serialization(
                    ::alloc::__export::must_use({
                        ::alloc::fmt::format(
                            format_args!("rkyv deserialization failed: {0:?}", e),
                        )
                    }),
                ))?,
        )
    }
    fn get_blob(&self) -> &Self::Chunk {
        {
            ::core::panicking::panic_fmt(
                format_args!(
                    "not implemented: {0}",
                    format_args!("get_blob() requires storing a chunk reference"),
                ),
            );
        }
    }
}
impl IntoIterator for EnumBlob {
    type Item = rewrite::results::NetabaseResult<EnumBlobChunk>;
    type IntoIter = std::vec::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        rewrite::traits::structural::blob::NetabaseBlobItem::into_chunks_iter(
            self,
            rewrite::traits::structural::blob::ChunkSize::Default,
        )
    }
}
struct SimpleStreamingBlob {
    data: Vec<u8>,
}
#[automatically_derived]
///An archived [`SimpleStreamingBlob`]
#[bytecheck(crate = ::rkyv::bytecheck)]
#[repr(C)]
struct ArchivedSimpleStreamingBlob
where
    Vec<u8>: ::rkyv::Archive,
{
    ///The archived counterpart of [`SimpleStreamingBlob::data`]
    data: <Vec<u8> as ::rkyv::Archive>::Archived,
}
#[automatically_derived]
unsafe impl<
    __C: ::rkyv::bytecheck::rancor::Fallible + ?::core::marker::Sized,
> ::rkyv::bytecheck::CheckBytes<__C> for ArchivedSimpleStreamingBlob
where
    Vec<u8>: ::rkyv::Archive,
    <__C as ::rkyv::bytecheck::rancor::Fallible>::Error: ::rkyv::bytecheck::rancor::Trace,
    <Vec<u8> as ::rkyv::Archive>::Archived: ::rkyv::bytecheck::CheckBytes<__C>,
{
    unsafe fn check_bytes(
        value: *const Self,
        context: &mut __C,
    ) -> ::core::result::Result<
        (),
        <__C as ::rkyv::bytecheck::rancor::Fallible>::Error,
    > {
        <<Vec<
            u8,
        > as ::rkyv::Archive>::Archived as ::rkyv::bytecheck::CheckBytes<
            __C,
        >>::check_bytes(&raw const (*value).data, context)
            .map_err(|e| {
                <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Trace>::trace(
                    e,
                    ::rkyv::bytecheck::StructCheckContext {
                        struct_name: "ArchivedSimpleStreamingBlob",
                        field_name: "data",
                    },
                )
            })?;
        ::core::result::Result::Ok(())
    }
}
#[automatically_derived]
///The resolver for an archived [`SimpleStreamingBlob`]
struct SimpleStreamingBlobResolver
where
    Vec<u8>: ::rkyv::Archive,
{
    data: <Vec<u8> as ::rkyv::Archive>::Resolver,
}
impl ::rkyv::Archive for SimpleStreamingBlob
where
    Vec<u8>: ::rkyv::Archive,
{
    type Archived = ArchivedSimpleStreamingBlob;
    type Resolver = SimpleStreamingBlobResolver;
    const COPY_OPTIMIZATION: ::rkyv::traits::CopyOptimization<Self> = unsafe {
        ::rkyv::traits::CopyOptimization::enable_if(
            0 + ::core::mem::size_of::<Vec<u8>>()
                == ::core::mem::size_of::<SimpleStreamingBlob>()
                && <Vec<u8> as ::rkyv::Archive>::COPY_OPTIMIZATION.is_enabled()
                && const { builtin # offset_of(SimpleStreamingBlob, data) }
                    == const { builtin # offset_of(ArchivedSimpleStreamingBlob, data) },
        )
    };
    #[allow(clippy::unit_arg)]
    fn resolve(&self, resolver: Self::Resolver, out: ::rkyv::Place<Self::Archived>) {
        let field_ptr = unsafe { &raw mut (*out.ptr()).data };
        let field_out = unsafe { ::rkyv::Place::from_field_unchecked(out, field_ptr) };
        <Vec<u8> as ::rkyv::Archive>::resolve(&self.data, resolver.data, field_out);
    }
}
unsafe impl ::rkyv::traits::Portable for ArchivedSimpleStreamingBlob
where
    Vec<u8>: ::rkyv::Archive,
    <Vec<u8> as ::rkyv::Archive>::Archived: ::rkyv::traits::Portable,
{}
#[automatically_derived]
impl<__S: ::rkyv::rancor::Fallible + ?Sized> ::rkyv::Serialize<__S>
for SimpleStreamingBlob
where
    Vec<u8>: ::rkyv::Serialize<__S>,
{
    fn serialize(
        &self,
        serializer: &mut __S,
    ) -> ::core::result::Result<
        <Self as ::rkyv::Archive>::Resolver,
        <__S as ::rkyv::rancor::Fallible>::Error,
    > {
        let __this = self;
        ::core::result::Result::Ok(SimpleStreamingBlobResolver {
            data: <Vec<
                u8,
            > as ::rkyv::Serialize<__S>>::serialize(&__this.data, serializer)?,
        })
    }
}
#[automatically_derived]
impl<
    __D: ::rkyv::rancor::Fallible + ?Sized,
> ::rkyv::Deserialize<SimpleStreamingBlob, __D> for ::rkyv::Archived<SimpleStreamingBlob>
where
    Vec<u8>: ::rkyv::Archive,
    <Vec<u8> as ::rkyv::Archive>::Archived: ::rkyv::Deserialize<Vec<u8>, __D>,
{
    fn deserialize(
        &self,
        deserializer: &mut __D,
    ) -> ::core::result::Result<
        SimpleStreamingBlob,
        <__D as ::rkyv::rancor::Fallible>::Error,
    > {
        let __this = self;
        ::core::result::Result::Ok(SimpleStreamingBlob {
            data: <<Vec<
                u8,
            > as ::rkyv::Archive>::Archived as ::rkyv::Deserialize<
                Vec<u8>,
                __D,
            >>::deserialize(&__this.data, deserializer)?,
        })
    }
}
#[automatically_derived]
impl ::core::fmt::Debug for SimpleStreamingBlob {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::debug_struct_field1_finish(
            f,
            "SimpleStreamingBlob",
            "data",
            &&self.data,
        )
    }
}
#[automatically_derived]
impl ::core::clone::Clone for SimpleStreamingBlob {
    #[inline]
    fn clone(&self) -> SimpleStreamingBlob {
        SimpleStreamingBlob {
            data: ::core::clone::Clone::clone(&self.data),
        }
    }
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for SimpleStreamingBlob {}
#[automatically_derived]
impl ::core::cmp::PartialEq for SimpleStreamingBlob {
    #[inline]
    fn eq(&self, other: &SimpleStreamingBlob) -> bool {
        self.data == other.data
    }
}
#[automatically_derived]
impl ::core::cmp::Eq for SimpleStreamingBlob {
    #[inline]
    #[doc(hidden)]
    #[coverage(off)]
    fn assert_fields_are_eq(&self) {
        let _: ::core::cmp::AssertParamIsEq<Vec<u8>>;
    }
}
pub struct SimpleStreamingBlobChunk {
    pub index: usize,
    pub data: Vec<u8>,
}
#[automatically_derived]
impl ::core::fmt::Debug for SimpleStreamingBlobChunk {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::debug_struct_field2_finish(
            f,
            "SimpleStreamingBlobChunk",
            "index",
            &self.index,
            "data",
            &&self.data,
        )
    }
}
#[automatically_derived]
impl ::core::clone::Clone for SimpleStreamingBlobChunk {
    #[inline]
    fn clone(&self) -> SimpleStreamingBlobChunk {
        SimpleStreamingBlobChunk {
            index: ::core::clone::Clone::clone(&self.index),
            data: ::core::clone::Clone::clone(&self.data),
        }
    }
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for SimpleStreamingBlobChunk {}
#[automatically_derived]
impl ::core::cmp::PartialEq for SimpleStreamingBlobChunk {
    #[inline]
    fn eq(&self, other: &SimpleStreamingBlobChunk) -> bool {
        self.index == other.index && self.data == other.data
    }
}
#[automatically_derived]
impl ::core::cmp::Eq for SimpleStreamingBlobChunk {
    #[inline]
    #[doc(hidden)]
    #[coverage(off)]
    fn assert_fields_are_eq(&self) {
        let _: ::core::cmp::AssertParamIsEq<usize>;
        let _: ::core::cmp::AssertParamIsEq<Vec<u8>>;
    }
}
#[automatically_derived]
impl ::core::cmp::PartialOrd for SimpleStreamingBlobChunk {
    #[inline]
    fn partial_cmp(
        &self,
        other: &SimpleStreamingBlobChunk,
    ) -> ::core::option::Option<::core::cmp::Ordering> {
        match ::core::cmp::PartialOrd::partial_cmp(&self.index, &other.index) {
            ::core::option::Option::Some(::core::cmp::Ordering::Equal) => {
                ::core::cmp::PartialOrd::partial_cmp(&self.data, &other.data)
            }
            cmp => cmp,
        }
    }
}
#[automatically_derived]
impl ::core::cmp::Ord for SimpleStreamingBlobChunk {
    #[inline]
    fn cmp(&self, other: &SimpleStreamingBlobChunk) -> ::core::cmp::Ordering {
        match ::core::cmp::Ord::cmp(&self.index, &other.index) {
            ::core::cmp::Ordering::Equal => {
                ::core::cmp::Ord::cmp(&self.data, &other.data)
            }
            cmp => cmp,
        }
    }
}
#[automatically_derived]
///An archived [`SimpleStreamingBlobChunk`]
#[bytecheck(crate = ::rkyv::bytecheck)]
#[repr(C)]
pub struct ArchivedSimpleStreamingBlobChunk
where
    usize: ::rkyv::Archive,
    Vec<u8>: ::rkyv::Archive,
{
    ///The archived counterpart of [`SimpleStreamingBlobChunk::index`]
    pub index: <usize as ::rkyv::Archive>::Archived,
    ///The archived counterpart of [`SimpleStreamingBlobChunk::data`]
    pub data: <Vec<u8> as ::rkyv::Archive>::Archived,
}
#[automatically_derived]
unsafe impl<
    __C: ::rkyv::bytecheck::rancor::Fallible + ?::core::marker::Sized,
> ::rkyv::bytecheck::CheckBytes<__C> for ArchivedSimpleStreamingBlobChunk
where
    usize: ::rkyv::Archive,
    Vec<u8>: ::rkyv::Archive,
    <__C as ::rkyv::bytecheck::rancor::Fallible>::Error: ::rkyv::bytecheck::rancor::Trace,
    <usize as ::rkyv::Archive>::Archived: ::rkyv::bytecheck::CheckBytes<__C>,
    <Vec<u8> as ::rkyv::Archive>::Archived: ::rkyv::bytecheck::CheckBytes<__C>,
{
    unsafe fn check_bytes(
        value: *const Self,
        context: &mut __C,
    ) -> ::core::result::Result<
        (),
        <__C as ::rkyv::bytecheck::rancor::Fallible>::Error,
    > {
        <<usize as ::rkyv::Archive>::Archived as ::rkyv::bytecheck::CheckBytes<
            __C,
        >>::check_bytes(&raw const (*value).index, context)
            .map_err(|e| {
                <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Trace>::trace(
                    e,
                    ::rkyv::bytecheck::StructCheckContext {
                        struct_name: "ArchivedSimpleStreamingBlobChunk",
                        field_name: "index",
                    },
                )
            })?;
        <<Vec<
            u8,
        > as ::rkyv::Archive>::Archived as ::rkyv::bytecheck::CheckBytes<
            __C,
        >>::check_bytes(&raw const (*value).data, context)
            .map_err(|e| {
                <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Trace>::trace(
                    e,
                    ::rkyv::bytecheck::StructCheckContext {
                        struct_name: "ArchivedSimpleStreamingBlobChunk",
                        field_name: "data",
                    },
                )
            })?;
        ::core::result::Result::Ok(())
    }
}
#[automatically_derived]
///The resolver for an archived [`SimpleStreamingBlobChunk`]
pub struct SimpleStreamingBlobChunkResolver
where
    usize: ::rkyv::Archive,
    Vec<u8>: ::rkyv::Archive,
{
    index: <usize as ::rkyv::Archive>::Resolver,
    data: <Vec<u8> as ::rkyv::Archive>::Resolver,
}
impl ::rkyv::Archive for SimpleStreamingBlobChunk
where
    usize: ::rkyv::Archive,
    Vec<u8>: ::rkyv::Archive,
{
    type Archived = ArchivedSimpleStreamingBlobChunk;
    type Resolver = SimpleStreamingBlobChunkResolver;
    const COPY_OPTIMIZATION: ::rkyv::traits::CopyOptimization<Self> = unsafe {
        ::rkyv::traits::CopyOptimization::enable_if(
            0 + ::core::mem::size_of::<usize>() + ::core::mem::size_of::<Vec<u8>>()
                == ::core::mem::size_of::<SimpleStreamingBlobChunk>()
                && <usize as ::rkyv::Archive>::COPY_OPTIMIZATION.is_enabled()
                && const { builtin # offset_of(SimpleStreamingBlobChunk, index) }
                    == const {
                        builtin # offset_of(ArchivedSimpleStreamingBlobChunk, index)
                    } && <Vec<u8> as ::rkyv::Archive>::COPY_OPTIMIZATION.is_enabled()
                && const { builtin # offset_of(SimpleStreamingBlobChunk, data) }
                    == const {
                        builtin # offset_of(ArchivedSimpleStreamingBlobChunk, data)
                    },
        )
    };
    #[allow(clippy::unit_arg)]
    fn resolve(&self, resolver: Self::Resolver, out: ::rkyv::Place<Self::Archived>) {
        let field_ptr = unsafe { &raw mut (*out.ptr()).index };
        let field_out = unsafe { ::rkyv::Place::from_field_unchecked(out, field_ptr) };
        <usize as ::rkyv::Archive>::resolve(&self.index, resolver.index, field_out);
        let field_ptr = unsafe { &raw mut (*out.ptr()).data };
        let field_out = unsafe { ::rkyv::Place::from_field_unchecked(out, field_ptr) };
        <Vec<u8> as ::rkyv::Archive>::resolve(&self.data, resolver.data, field_out);
    }
}
unsafe impl ::rkyv::traits::Portable for ArchivedSimpleStreamingBlobChunk
where
    usize: ::rkyv::Archive,
    Vec<u8>: ::rkyv::Archive,
    <usize as ::rkyv::Archive>::Archived: ::rkyv::traits::Portable,
    <Vec<u8> as ::rkyv::Archive>::Archived: ::rkyv::traits::Portable,
{}
#[automatically_derived]
impl<__S: ::rkyv::rancor::Fallible + ?Sized> ::rkyv::Serialize<__S>
for SimpleStreamingBlobChunk
where
    usize: ::rkyv::Serialize<__S>,
    Vec<u8>: ::rkyv::Serialize<__S>,
{
    fn serialize(
        &self,
        serializer: &mut __S,
    ) -> ::core::result::Result<
        <Self as ::rkyv::Archive>::Resolver,
        <__S as ::rkyv::rancor::Fallible>::Error,
    > {
        let __this = self;
        ::core::result::Result::Ok(SimpleStreamingBlobChunkResolver {
            index: <usize as ::rkyv::Serialize<
                __S,
            >>::serialize(&__this.index, serializer)?,
            data: <Vec<
                u8,
            > as ::rkyv::Serialize<__S>>::serialize(&__this.data, serializer)?,
        })
    }
}
#[automatically_derived]
impl<
    __D: ::rkyv::rancor::Fallible + ?Sized,
> ::rkyv::Deserialize<SimpleStreamingBlobChunk, __D>
for ::rkyv::Archived<SimpleStreamingBlobChunk>
where
    usize: ::rkyv::Archive,
    <usize as ::rkyv::Archive>::Archived: ::rkyv::Deserialize<usize, __D>,
    Vec<u8>: ::rkyv::Archive,
    <Vec<u8> as ::rkyv::Archive>::Archived: ::rkyv::Deserialize<Vec<u8>, __D>,
{
    fn deserialize(
        &self,
        deserializer: &mut __D,
    ) -> ::core::result::Result<
        SimpleStreamingBlobChunk,
        <__D as ::rkyv::rancor::Fallible>::Error,
    > {
        let __this = self;
        ::core::result::Result::Ok(SimpleStreamingBlobChunk {
            index: <<usize as ::rkyv::Archive>::Archived as ::rkyv::Deserialize<
                usize,
                __D,
            >>::deserialize(&__this.index, deserializer)?,
            data: <<Vec<
                u8,
            > as ::rkyv::Archive>::Archived as ::rkyv::Deserialize<
                Vec<u8>,
                __D,
            >>::deserialize(&__this.data, deserializer)?,
        })
    }
}
impl ::rewrite::traits::structural::blob::BlobItemChunk for SimpleStreamingBlobChunk {
    type Index = usize;
    fn get_index(&self) -> &Self::Index {
        &self.index
    }
}
pub enum SimpleStreamingBlobChunkFill {
    Full(usize),
    Partial(usize),
    Corrupted(usize),
}
#[automatically_derived]
impl ::core::fmt::Debug for SimpleStreamingBlobChunkFill {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match self {
            SimpleStreamingBlobChunkFill::Full(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "Full", &__self_0)
            }
            SimpleStreamingBlobChunkFill::Partial(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(
                    f,
                    "Partial",
                    &__self_0,
                )
            }
            SimpleStreamingBlobChunkFill::Corrupted(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(
                    f,
                    "Corrupted",
                    &__self_0,
                )
            }
        }
    }
}
#[automatically_derived]
#[doc(hidden)]
unsafe impl ::core::clone::TrivialClone for SimpleStreamingBlobChunkFill {}
#[automatically_derived]
impl ::core::clone::Clone for SimpleStreamingBlobChunkFill {
    #[inline]
    fn clone(&self) -> SimpleStreamingBlobChunkFill {
        let _: ::core::clone::AssertParamIsClone<usize>;
        *self
    }
}
#[automatically_derived]
impl ::core::marker::Copy for SimpleStreamingBlobChunkFill {}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for SimpleStreamingBlobChunkFill {}
#[automatically_derived]
impl ::core::cmp::PartialEq for SimpleStreamingBlobChunkFill {
    #[inline]
    fn eq(&self, other: &SimpleStreamingBlobChunkFill) -> bool {
        let __self_discr = ::core::intrinsics::discriminant_value(self);
        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
        __self_discr == __arg1_discr
            && match (self, other) {
                (
                    SimpleStreamingBlobChunkFill::Full(__self_0),
                    SimpleStreamingBlobChunkFill::Full(__arg1_0),
                ) => __self_0 == __arg1_0,
                (
                    SimpleStreamingBlobChunkFill::Partial(__self_0),
                    SimpleStreamingBlobChunkFill::Partial(__arg1_0),
                ) => __self_0 == __arg1_0,
                (
                    SimpleStreamingBlobChunkFill::Corrupted(__self_0),
                    SimpleStreamingBlobChunkFill::Corrupted(__arg1_0),
                ) => __self_0 == __arg1_0,
                _ => unsafe { ::core::intrinsics::unreachable() }
            }
    }
}
#[automatically_derived]
impl ::core::cmp::Eq for SimpleStreamingBlobChunkFill {
    #[inline]
    #[doc(hidden)]
    #[coverage(off)]
    fn assert_fields_are_eq(&self) {
        let _: ::core::cmp::AssertParamIsEq<usize>;
    }
}
#[automatically_derived]
impl ::core::cmp::PartialOrd for SimpleStreamingBlobChunkFill {
    #[inline]
    fn partial_cmp(
        &self,
        other: &SimpleStreamingBlobChunkFill,
    ) -> ::core::option::Option<::core::cmp::Ordering> {
        let __self_discr = ::core::intrinsics::discriminant_value(self);
        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
        match (self, other) {
            (
                SimpleStreamingBlobChunkFill::Full(__self_0),
                SimpleStreamingBlobChunkFill::Full(__arg1_0),
            ) => ::core::cmp::PartialOrd::partial_cmp(__self_0, __arg1_0),
            (
                SimpleStreamingBlobChunkFill::Partial(__self_0),
                SimpleStreamingBlobChunkFill::Partial(__arg1_0),
            ) => ::core::cmp::PartialOrd::partial_cmp(__self_0, __arg1_0),
            (
                SimpleStreamingBlobChunkFill::Corrupted(__self_0),
                SimpleStreamingBlobChunkFill::Corrupted(__arg1_0),
            ) => ::core::cmp::PartialOrd::partial_cmp(__self_0, __arg1_0),
            _ => ::core::cmp::PartialOrd::partial_cmp(&__self_discr, &__arg1_discr),
        }
    }
}
#[automatically_derived]
impl ::core::cmp::Ord for SimpleStreamingBlobChunkFill {
    #[inline]
    fn cmp(&self, other: &SimpleStreamingBlobChunkFill) -> ::core::cmp::Ordering {
        let __self_discr = ::core::intrinsics::discriminant_value(self);
        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
        match ::core::cmp::Ord::cmp(&__self_discr, &__arg1_discr) {
            ::core::cmp::Ordering::Equal => {
                match (self, other) {
                    (
                        SimpleStreamingBlobChunkFill::Full(__self_0),
                        SimpleStreamingBlobChunkFill::Full(__arg1_0),
                    ) => ::core::cmp::Ord::cmp(__self_0, __arg1_0),
                    (
                        SimpleStreamingBlobChunkFill::Partial(__self_0),
                        SimpleStreamingBlobChunkFill::Partial(__arg1_0),
                    ) => ::core::cmp::Ord::cmp(__self_0, __arg1_0),
                    (
                        SimpleStreamingBlobChunkFill::Corrupted(__self_0),
                        SimpleStreamingBlobChunkFill::Corrupted(__arg1_0),
                    ) => ::core::cmp::Ord::cmp(__self_0, __arg1_0),
                    _ => unsafe { ::core::intrinsics::unreachable() }
                }
            }
            cmp => cmp,
        }
    }
}
#[automatically_derived]
///An archived [`SimpleStreamingBlobChunkFill`]
#[bytecheck(crate = ::rkyv::bytecheck)]
#[repr(u8)]
pub enum ArchivedSimpleStreamingBlobChunkFill
where
    usize: ::rkyv::Archive,
    usize: ::rkyv::Archive,
    usize: ::rkyv::Archive,
{
    ///The archived counterpart of [`SimpleStreamingBlobChunkFill::Full`]
    #[allow(dead_code)]
    Full(
        ///The archived counterpart of [`SimpleStreamingBlobChunkFill::Full::0`]
        <usize as ::rkyv::Archive>::Archived,
    ),
    ///The archived counterpart of [`SimpleStreamingBlobChunkFill::Partial`]
    #[allow(dead_code)]
    Partial(
        ///The archived counterpart of [`SimpleStreamingBlobChunkFill::Partial::0`]
        <usize as ::rkyv::Archive>::Archived,
    ),
    ///The archived counterpart of [`SimpleStreamingBlobChunkFill::Corrupted`]
    #[allow(dead_code)]
    Corrupted(
        ///The archived counterpart of [`SimpleStreamingBlobChunkFill::Corrupted::0`]
        <usize as ::rkyv::Archive>::Archived,
    ),
}
const _: () = {
    #[repr(u8)]
    enum Tag {
        Full,
        Partial,
        Corrupted,
    }
    struct Discriminant;
    #[automatically_derived]
    impl Discriminant {
        #[allow(non_upper_case_globals)]
        const Full: u8 = Tag::Full as u8;
        #[allow(non_upper_case_globals)]
        const Partial: u8 = Tag::Partial as u8;
        #[allow(non_upper_case_globals)]
        const Corrupted: u8 = Tag::Corrupted as u8;
    }
    #[repr(C)]
    struct VariantFull(
        Tag,
        <usize as ::rkyv::Archive>::Archived,
        ::core::marker::PhantomData<ArchivedSimpleStreamingBlobChunkFill>,
    )
    where
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive;
    #[repr(C)]
    struct VariantPartial(
        Tag,
        <usize as ::rkyv::Archive>::Archived,
        ::core::marker::PhantomData<ArchivedSimpleStreamingBlobChunkFill>,
    )
    where
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive;
    #[repr(C)]
    struct VariantCorrupted(
        Tag,
        <usize as ::rkyv::Archive>::Archived,
        ::core::marker::PhantomData<ArchivedSimpleStreamingBlobChunkFill>,
    )
    where
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive;
    #[automatically_derived]
    unsafe impl<
        __C: ::rkyv::bytecheck::rancor::Fallible + ?::core::marker::Sized,
    > ::rkyv::bytecheck::CheckBytes<__C> for ArchivedSimpleStreamingBlobChunkFill
    where
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive,
        <__C as ::rkyv::bytecheck::rancor::Fallible>::Error: ::rkyv::bytecheck::rancor::Source,
        <usize as ::rkyv::Archive>::Archived: ::rkyv::bytecheck::CheckBytes<__C>,
        <usize as ::rkyv::Archive>::Archived: ::rkyv::bytecheck::CheckBytes<__C>,
        <usize as ::rkyv::Archive>::Archived: ::rkyv::bytecheck::CheckBytes<__C>,
    {
        unsafe fn check_bytes(
            value: *const Self,
            context: &mut __C,
        ) -> ::core::result::Result<
            (),
            <__C as ::rkyv::bytecheck::rancor::Fallible>::Error,
        > {
            let tag = *value.cast::<u8>();
            match tag {
                Discriminant::Full => {
                    let value = value.cast::<VariantFull>();
                    <<usize as ::rkyv::Archive>::Archived as ::rkyv::bytecheck::CheckBytes<
                        __C,
                    >>::check_bytes(&raw const (*value).1, context)
                        .map_err(|e| {
                            <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Trace>::trace(
                                e,
                                ::rkyv::bytecheck::UnnamedEnumVariantCheckContext {
                                    enum_name: "ArchivedSimpleStreamingBlobChunkFill",
                                    variant_name: "Full",
                                    field_index: 1,
                                },
                            )
                        })?;
                }
                Discriminant::Partial => {
                    let value = value.cast::<VariantPartial>();
                    <<usize as ::rkyv::Archive>::Archived as ::rkyv::bytecheck::CheckBytes<
                        __C,
                    >>::check_bytes(&raw const (*value).1, context)
                        .map_err(|e| {
                            <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Trace>::trace(
                                e,
                                ::rkyv::bytecheck::UnnamedEnumVariantCheckContext {
                                    enum_name: "ArchivedSimpleStreamingBlobChunkFill",
                                    variant_name: "Partial",
                                    field_index: 1,
                                },
                            )
                        })?;
                }
                Discriminant::Corrupted => {
                    let value = value.cast::<VariantCorrupted>();
                    <<usize as ::rkyv::Archive>::Archived as ::rkyv::bytecheck::CheckBytes<
                        __C,
                    >>::check_bytes(&raw const (*value).1, context)
                        .map_err(|e| {
                            <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Trace>::trace(
                                e,
                                ::rkyv::bytecheck::UnnamedEnumVariantCheckContext {
                                    enum_name: "ArchivedSimpleStreamingBlobChunkFill",
                                    variant_name: "Corrupted",
                                    field_index: 1,
                                },
                            )
                        })?;
                }
                _ => {
                    return ::core::result::Result::Err(
                        <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Source>::new(::rkyv::bytecheck::InvalidEnumDiscriminantError {
                            enum_name: "ArchivedSimpleStreamingBlobChunkFill",
                            invalid_discriminant: tag,
                        }),
                    );
                }
            }
            ::core::result::Result::Ok(())
        }
    }
};
#[automatically_derived]
///The resolver for an archived [`SimpleStreamingBlobChunkFill`]
pub enum SimpleStreamingBlobChunkFillResolver
where
    usize: ::rkyv::Archive,
    usize: ::rkyv::Archive,
    usize: ::rkyv::Archive,
{
    ///The resolver for [`SimpleStreamingBlobChunkFill::Full`]
    #[allow(dead_code)]
    Full(<usize as ::rkyv::Archive>::Resolver),
    ///The resolver for [`SimpleStreamingBlobChunkFill::Partial`]
    #[allow(dead_code)]
    Partial(<usize as ::rkyv::Archive>::Resolver),
    ///The resolver for [`SimpleStreamingBlobChunkFill::Corrupted`]
    #[allow(dead_code)]
    Corrupted(<usize as ::rkyv::Archive>::Resolver),
}
const _: () = {
    #[repr(u8)]
    enum ArchivedTag {
        Full,
        Partial,
        Corrupted,
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for ArchivedTag {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for ArchivedTag {
        #[inline]
        fn eq(&self, other: &ArchivedTag) -> bool {
            let __self_discr = ::core::intrinsics::discriminant_value(self);
            let __arg1_discr = ::core::intrinsics::discriminant_value(other);
            __self_discr == __arg1_discr
        }
    }
    #[automatically_derived]
    impl ::core::cmp::PartialOrd for ArchivedTag {
        #[inline]
        fn partial_cmp(
            &self,
            other: &ArchivedTag,
        ) -> ::core::option::Option<::core::cmp::Ordering> {
            let __self_discr = ::core::intrinsics::discriminant_value(self);
            let __arg1_discr = ::core::intrinsics::discriminant_value(other);
            ::core::cmp::PartialOrd::partial_cmp(&__self_discr, &__arg1_discr)
        }
    }
    #[repr(C)]
    struct ArchivedVariantFull(
        ArchivedTag,
        <usize as ::rkyv::Archive>::Archived,
        ::core::marker::PhantomData<SimpleStreamingBlobChunkFill>,
    )
    where
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive;
    #[repr(C)]
    struct ArchivedVariantPartial(
        ArchivedTag,
        <usize as ::rkyv::Archive>::Archived,
        ::core::marker::PhantomData<SimpleStreamingBlobChunkFill>,
    )
    where
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive;
    #[repr(C)]
    struct ArchivedVariantCorrupted(
        ArchivedTag,
        <usize as ::rkyv::Archive>::Archived,
        ::core::marker::PhantomData<SimpleStreamingBlobChunkFill>,
    )
    where
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive;
    impl ::rkyv::Archive for SimpleStreamingBlobChunkFill
    where
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive,
    {
        type Archived = ArchivedSimpleStreamingBlobChunkFill;
        type Resolver = SimpleStreamingBlobChunkFillResolver;
        #[allow(clippy::unit_arg)]
        fn resolve(
            &self,
            resolver: <Self as ::rkyv::Archive>::Resolver,
            out: ::rkyv::Place<<Self as ::rkyv::Archive>::Archived>,
        ) {
            let __this = self;
            match resolver {
                SimpleStreamingBlobChunkFillResolver::Full(resolver_0) => {
                    match __this {
                        SimpleStreamingBlobChunkFill::Full(self_0, ..) => {
                            let out = unsafe {
                                out.cast_unchecked::<ArchivedVariantFull>()
                            };
                            let tag_ptr = unsafe { &raw mut (*out.ptr()).0 };
                            unsafe {
                                tag_ptr.write(ArchivedTag::Full);
                            }
                            let field_ptr = unsafe { &raw mut (*out.ptr()).1 };
                            let out_field = unsafe {
                                ::rkyv::Place::from_field_unchecked(out, field_ptr)
                            };
                            <usize as ::rkyv::Archive>::resolve(
                                self_0,
                                resolver_0,
                                out_field,
                            );
                        }
                        #[allow(unreachable_patterns)]
                        _ => unsafe { ::core::hint::unreachable_unchecked() }
                    }
                }
                SimpleStreamingBlobChunkFillResolver::Partial(resolver_0) => {
                    match __this {
                        SimpleStreamingBlobChunkFill::Partial(self_0, ..) => {
                            let out = unsafe {
                                out.cast_unchecked::<ArchivedVariantPartial>()
                            };
                            let tag_ptr = unsafe { &raw mut (*out.ptr()).0 };
                            unsafe {
                                tag_ptr.write(ArchivedTag::Partial);
                            }
                            let field_ptr = unsafe { &raw mut (*out.ptr()).1 };
                            let out_field = unsafe {
                                ::rkyv::Place::from_field_unchecked(out, field_ptr)
                            };
                            <usize as ::rkyv::Archive>::resolve(
                                self_0,
                                resolver_0,
                                out_field,
                            );
                        }
                        #[allow(unreachable_patterns)]
                        _ => unsafe { ::core::hint::unreachable_unchecked() }
                    }
                }
                SimpleStreamingBlobChunkFillResolver::Corrupted(resolver_0) => {
                    match __this {
                        SimpleStreamingBlobChunkFill::Corrupted(self_0, ..) => {
                            let out = unsafe {
                                out.cast_unchecked::<ArchivedVariantCorrupted>()
                            };
                            let tag_ptr = unsafe { &raw mut (*out.ptr()).0 };
                            unsafe {
                                tag_ptr.write(ArchivedTag::Corrupted);
                            }
                            let field_ptr = unsafe { &raw mut (*out.ptr()).1 };
                            let out_field = unsafe {
                                ::rkyv::Place::from_field_unchecked(out, field_ptr)
                            };
                            <usize as ::rkyv::Archive>::resolve(
                                self_0,
                                resolver_0,
                                out_field,
                            );
                        }
                        #[allow(unreachable_patterns)]
                        _ => unsafe { ::core::hint::unreachable_unchecked() }
                    }
                }
            }
        }
    }
};
unsafe impl ::rkyv::traits::Portable for ArchivedSimpleStreamingBlobChunkFill
where
    usize: ::rkyv::Archive,
    usize: ::rkyv::Archive,
    usize: ::rkyv::Archive,
    <usize as ::rkyv::Archive>::Archived: ::rkyv::traits::Portable,
    <usize as ::rkyv::Archive>::Archived: ::rkyv::traits::Portable,
    <usize as ::rkyv::Archive>::Archived: ::rkyv::traits::Portable,
{}
#[automatically_derived]
impl<__S: ::rkyv::rancor::Fallible + ?Sized> ::rkyv::Serialize<__S>
for SimpleStreamingBlobChunkFill
where
    usize: ::rkyv::Serialize<__S>,
    usize: ::rkyv::Serialize<__S>,
    usize: ::rkyv::Serialize<__S>,
{
    fn serialize(
        &self,
        serializer: &mut __S,
    ) -> ::core::result::Result<
        <Self as ::rkyv::Archive>::Resolver,
        <__S as ::rkyv::rancor::Fallible>::Error,
    > {
        let __this = self;
        ::core::result::Result::Ok(
            match __this {
                SimpleStreamingBlobChunkFill::Full(_0, ..) => {
                    SimpleStreamingBlobChunkFillResolver::Full(
                        <usize as ::rkyv::Serialize<__S>>::serialize(_0, serializer)?,
                    )
                }
                SimpleStreamingBlobChunkFill::Partial(_0, ..) => {
                    SimpleStreamingBlobChunkFillResolver::Partial(
                        <usize as ::rkyv::Serialize<__S>>::serialize(_0, serializer)?,
                    )
                }
                SimpleStreamingBlobChunkFill::Corrupted(_0, ..) => {
                    SimpleStreamingBlobChunkFillResolver::Corrupted(
                        <usize as ::rkyv::Serialize<__S>>::serialize(_0, serializer)?,
                    )
                }
            },
        )
    }
}
#[automatically_derived]
impl<
    __D: ::rkyv::rancor::Fallible + ?Sized,
> ::rkyv::Deserialize<SimpleStreamingBlobChunkFill, __D>
for ::rkyv::Archived<SimpleStreamingBlobChunkFill>
where
    usize: ::rkyv::Archive,
    <usize as ::rkyv::Archive>::Archived: ::rkyv::Deserialize<usize, __D>,
    usize: ::rkyv::Archive,
    <usize as ::rkyv::Archive>::Archived: ::rkyv::Deserialize<usize, __D>,
    usize: ::rkyv::Archive,
    <usize as ::rkyv::Archive>::Archived: ::rkyv::Deserialize<usize, __D>,
{
    fn deserialize(
        &self,
        deserializer: &mut __D,
    ) -> ::core::result::Result<
        SimpleStreamingBlobChunkFill,
        <__D as ::rkyv::rancor::Fallible>::Error,
    > {
        let __this = self;
        ::core::result::Result::Ok(
            match __this {
                Self::Full(_0, ..) => {
                    SimpleStreamingBlobChunkFill::Full(
                        <<usize as ::rkyv::Archive>::Archived as ::rkyv::Deserialize<
                            usize,
                            __D,
                        >>::deserialize(_0, deserializer)?,
                    )
                }
                Self::Partial(_0, ..) => {
                    SimpleStreamingBlobChunkFill::Partial(
                        <<usize as ::rkyv::Archive>::Archived as ::rkyv::Deserialize<
                            usize,
                            __D,
                        >>::deserialize(_0, deserializer)?,
                    )
                }
                Self::Corrupted(_0, ..) => {
                    SimpleStreamingBlobChunkFill::Corrupted(
                        <<usize as ::rkyv::Archive>::Archived as ::rkyv::Deserialize<
                            usize,
                            __D,
                        >>::deserialize(_0, deserializer)?,
                    )
                }
            },
        )
    }
}
impl SimpleStreamingBlobChunkFill {
    pub fn from_size(actual: usize, expected: usize) -> Self {
        if actual == expected {
            Self::Full(actual)
        } else if actual < expected {
            Self::Partial(actual)
        } else {
            Self::Corrupted(actual)
        }
    }
}
impl rewrite::traits::structural::blob::NetabaseBlobItem for SimpleStreamingBlob {
    type Chunk = SimpleStreamingBlobChunk;
    type BlobIter = std::vec::IntoIter<rewrite::results::NetabaseResult<Self::Chunk>>;
    const DEFAULT_CHUNK_SIZE: usize = 0usize;
    fn into_chunks(
        self,
        size: rewrite::traits::structural::blob::ChunkSize,
    ) -> Box<dyn Iterator<Item = Self::Chunk>> {
        Box::new(self.into_chunks_iter(size).filter_map(|r| r.ok()))
    }
    fn into_chunks_iter(
        self,
        size: rewrite::traits::structural::blob::ChunkSize,
    ) -> Self::BlobIter {
        let serialized_data: rewrite::results::NetabaseResult<Vec<u8>> = (|| -> rewrite::results::NetabaseResult<
            Vec<u8>,
        > {
            Ok(
                rkyv::to_bytes::<rkyv::rancor::Error>(&self)
                    .map_err(|e| rewrite::results::NetabaseError::Serialization(
                        ::alloc::__export::must_use({
                            ::alloc::fmt::format(
                                format_args!("rkyv serialization failed: {0:?}", e),
                            )
                        }),
                    ))?
                    .to_vec(),
            )
        })();
        let chunk_size = match size {
            rewrite::traits::structural::blob::ChunkSize::Default => {
                if Self::DEFAULT_CHUNK_SIZE > 0 {
                    Self::DEFAULT_CHUNK_SIZE
                } else {
                    1024
                }
            }
            rewrite::traits::structural::blob::ChunkSize::Size(n) => n,
        };
        match serialized_data {
            Ok(data) => {
                data.chunks(chunk_size)
                    .enumerate()
                    .map(|(index, chunk_data)| {
                        Ok(Self::Chunk {
                            index,
                            data: chunk_data.to_vec(),
                        })
                    })
                    .collect::<Vec<_>>()
                    .into_iter()
            }
            Err(e) => {
                ::alloc::boxed::box_assume_init_into_vec_unsafe(
                        ::alloc::intrinsics::write_box_via_move(
                            ::alloc::boxed::Box::new_uninit(),
                            [Err(e)],
                        ),
                    )
                    .into_iter()
            }
        }
    }
    fn try_from_chunks(
        chunks: impl Iterator<Item = Self::Chunk>,
        size: rewrite::traits::structural::blob::ChunkSize,
    ) -> rewrite::results::NetabaseResult<Self> {
        let mut sorted_chunks: Vec<_> = chunks.collect();
        sorted_chunks.sort_by_key(|c| c.index);
        if sorted_chunks.is_empty() {
            return Err(
                rewrite::results::NetabaseError::BlobReconstruction(
                    rewrite::results::BlobReconstructionError::MissingChunks,
                ),
            );
        }
        let chunk_size = match size {
            rewrite::traits::structural::blob::ChunkSize::Default => {
                if Self::DEFAULT_CHUNK_SIZE > 0 {
                    Self::DEFAULT_CHUNK_SIZE
                } else {
                    1024
                }
            }
            rewrite::traits::structural::blob::ChunkSize::Size(n) => n,
        };
        let mut missing_details = Vec::new();
        let mut next_expected = 0;
        let max_idx = sorted_chunks.last().map(|c| c.index).unwrap_or(0);
        for chunk in &sorted_chunks {
            while chunk.index > next_expected {
                missing_details
                    .push(
                        ::alloc::__export::must_use({
                            ::alloc::fmt::format(
                                format_args!(
                                    "{0:?}({{ Index: {1}, Size: {2} }})",
                                    SimpleStreamingBlobChunkFill::Full(chunk_size),
                                    next_expected,
                                    chunk_size,
                                ),
                            )
                        }),
                    );
                next_expected += 1;
            }
            let fill = SimpleStreamingBlobChunkFill::from_size(
                chunk.data.len(),
                chunk_size,
            );
            match fill {
                SimpleStreamingBlobChunkFill::Corrupted(size) => {
                    return Err(
                        rewrite::results::NetabaseError::BlobReconstruction(
                            rewrite::results::BlobReconstructionError::InvalidChunkData(
                                ::alloc::__export::must_use({
                                    ::alloc::fmt::format(
                                        format_args!(
                                            "Corrupted chunk detected: {0:?}({{ Index: {1}, Size: {2} }}). Max allowed size is {3}.",
                                            fill,
                                            chunk.index,
                                            size,
                                            chunk_size,
                                        ),
                                    )
                                }),
                            ),
                        ),
                    );
                }
                SimpleStreamingBlobChunkFill::Partial(size) if chunk.index < max_idx => {
                    return Err(
                        rewrite::results::NetabaseError::BlobReconstruction(
                            rewrite::results::BlobReconstructionError::InvalidChunkData(
                                ::alloc::__export::must_use({
                                    ::alloc::fmt::format(
                                        format_args!(
                                            "Unexpected partial chunk in middle of stream: {0:?}({{ Index: {1}, Size: {2} }}). Expected {3} bytes.",
                                            fill,
                                            chunk.index,
                                            size,
                                            chunk_size,
                                        ),
                                    )
                                }),
                            ),
                        ),
                    );
                }
                _ => {}
            }
            if chunk.index == next_expected {
                next_expected += 1;
            }
        }
        if !missing_details.is_empty() {
            if let Some(last) = sorted_chunks.last() {
                let fill = SimpleStreamingBlobChunkFill::from_size(
                    last.data.len(),
                    chunk_size,
                );
                if #[allow(non_exhaustive_omitted_patterns)]
                match fill {
                    SimpleStreamingBlobChunkFill::Full(_) => true,
                    _ => false,
                } {
                    missing_details
                        .push(
                            ::alloc::__export::must_use({
                                ::alloc::fmt::format(
                                    format_args!(
                                        "... (Stream truncated: last chunk was Full, expected more data after Index {0})",
                                        last.index,
                                    ),
                                )
                            }),
                        );
                }
            }
        }
        if !missing_details.is_empty() {
            return Err(
                rewrite::results::NetabaseError::BlobReconstruction(
                    rewrite::results::BlobReconstructionError::InvalidChunkData(
                        ::alloc::__export::must_use({
                            ::alloc::fmt::format(
                                format_args!(
                                    "Missing chunks: [{0}]. Total chunks present: {1}",
                                    missing_details.join(", "),
                                    sorted_chunks.len(),
                                ),
                            )
                        }),
                    ),
                ),
            );
        }
        let serialized_data: Vec<u8> = sorted_chunks
            .into_iter()
            .flat_map(|c| c.data)
            .collect();
        Ok(
            rkyv::from_bytes::<Self, rkyv::rancor::Error>(&serialized_data)
                .map_err(|e| rewrite::results::NetabaseError::Serialization(
                    ::alloc::__export::must_use({
                        ::alloc::fmt::format(
                            format_args!("rkyv deserialization failed: {0:?}", e),
                        )
                    }),
                ))?,
        )
    }
    fn get_blob(&self) -> &Self::Chunk {
        {
            ::core::panicking::panic_fmt(
                format_args!(
                    "not implemented: {0}",
                    format_args!("get_blob() requires storing a chunk reference"),
                ),
            );
        }
    }
}
impl IntoIterator for SimpleStreamingBlob {
    type Item = rewrite::results::NetabaseResult<SimpleStreamingBlobChunk>;
    type IntoIter = std::vec::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        rewrite::traits::structural::blob::NetabaseBlobItem::into_chunks_iter(
            self,
            rewrite::traits::structural::blob::ChunkSize::Default,
        )
    }
}
struct GenericBlob<T>
where
    T: rkyv::Archive + Clone + Debug + PartialEq + 'static,
{
    data: T,
}
#[automatically_derived]
///An archived [`GenericBlob`]
#[bytecheck(crate = ::rkyv::bytecheck)]
#[repr(C)]
struct ArchivedGenericBlob<T>
where
    T: rkyv::Archive + Clone + Debug + PartialEq + 'static,
    T: ::rkyv::Archive,
{
    ///The archived counterpart of [`GenericBlob::data`]
    data: <T as ::rkyv::Archive>::Archived,
}
#[automatically_derived]
unsafe impl<
    T,
    __C: ::rkyv::bytecheck::rancor::Fallible + ?::core::marker::Sized,
> ::rkyv::bytecheck::CheckBytes<__C> for ArchivedGenericBlob<T>
where
    T: rkyv::Archive + Clone + Debug + PartialEq + 'static,
    T: ::rkyv::Archive,
    <__C as ::rkyv::bytecheck::rancor::Fallible>::Error: ::rkyv::bytecheck::rancor::Trace,
    <T as ::rkyv::Archive>::Archived: ::rkyv::bytecheck::CheckBytes<__C>,
{
    unsafe fn check_bytes(
        value: *const Self,
        context: &mut __C,
    ) -> ::core::result::Result<
        (),
        <__C as ::rkyv::bytecheck::rancor::Fallible>::Error,
    > {
        <<T as ::rkyv::Archive>::Archived as ::rkyv::bytecheck::CheckBytes<
            __C,
        >>::check_bytes(&raw const (*value).data, context)
            .map_err(|e| {
                <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Trace>::trace(
                    e,
                    ::rkyv::bytecheck::StructCheckContext {
                        struct_name: "ArchivedGenericBlob",
                        field_name: "data",
                    },
                )
            })?;
        ::core::result::Result::Ok(())
    }
}
#[automatically_derived]
///The resolver for an archived [`GenericBlob`]
struct GenericBlobResolver<T>
where
    T: rkyv::Archive + Clone + Debug + PartialEq + 'static,
    T: ::rkyv::Archive,
{
    data: <T as ::rkyv::Archive>::Resolver,
}
impl<T> ::rkyv::Archive for GenericBlob<T>
where
    T: rkyv::Archive + Clone + Debug + PartialEq + 'static,
    T: ::rkyv::Archive,
{
    type Archived = ArchivedGenericBlob<T>;
    type Resolver = GenericBlobResolver<T>;
    #[allow(clippy::unit_arg)]
    fn resolve(&self, resolver: Self::Resolver, out: ::rkyv::Place<Self::Archived>) {
        let field_ptr = unsafe { &raw mut (*out.ptr()).data };
        let field_out = unsafe { ::rkyv::Place::from_field_unchecked(out, field_ptr) };
        <T as ::rkyv::Archive>::resolve(&self.data, resolver.data, field_out);
    }
}
unsafe impl<T> ::rkyv::traits::Portable for ArchivedGenericBlob<T>
where
    T: rkyv::Archive + Clone + Debug + PartialEq + 'static,
    T: ::rkyv::Archive,
    <T as ::rkyv::Archive>::Archived: ::rkyv::traits::Portable,
{}
#[automatically_derived]
impl<__S: ::rkyv::rancor::Fallible + ?Sized, T> ::rkyv::Serialize<__S> for GenericBlob<T>
where
    T: rkyv::Archive + Clone + Debug + PartialEq + 'static,
    T: ::rkyv::Serialize<__S>,
{
    fn serialize(
        &self,
        serializer: &mut __S,
    ) -> ::core::result::Result<
        <Self as ::rkyv::Archive>::Resolver,
        <__S as ::rkyv::rancor::Fallible>::Error,
    > {
        let __this = self;
        ::core::result::Result::Ok(GenericBlobResolver {
            data: <T as ::rkyv::Serialize<__S>>::serialize(&__this.data, serializer)?,
        })
    }
}
#[automatically_derived]
impl<__D: ::rkyv::rancor::Fallible + ?Sized, T> ::rkyv::Deserialize<GenericBlob<T>, __D>
for ::rkyv::Archived<GenericBlob<T>>
where
    T: rkyv::Archive + Clone + Debug + PartialEq + 'static,
    T: ::rkyv::Archive,
    <T as ::rkyv::Archive>::Archived: ::rkyv::Deserialize<T, __D>,
{
    fn deserialize(
        &self,
        deserializer: &mut __D,
    ) -> ::core::result::Result<
        GenericBlob<T>,
        <__D as ::rkyv::rancor::Fallible>::Error,
    > {
        let __this = self;
        ::core::result::Result::Ok(GenericBlob {
            data: <<T as ::rkyv::Archive>::Archived as ::rkyv::Deserialize<
                T,
                __D,
            >>::deserialize(&__this.data, deserializer)?,
        })
    }
}
#[automatically_derived]
impl<T: ::core::fmt::Debug> ::core::fmt::Debug for GenericBlob<T>
where
    T: rkyv::Archive + Clone + Debug + PartialEq + 'static,
{
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::debug_struct_field1_finish(
            f,
            "GenericBlob",
            "data",
            &&self.data,
        )
    }
}
#[automatically_derived]
impl<T: ::core::clone::Clone> ::core::clone::Clone for GenericBlob<T>
where
    T: rkyv::Archive + Clone + Debug + PartialEq + 'static,
{
    #[inline]
    fn clone(&self) -> GenericBlob<T> {
        GenericBlob {
            data: ::core::clone::Clone::clone(&self.data),
        }
    }
}
#[automatically_derived]
impl<T> ::core::marker::StructuralPartialEq for GenericBlob<T>
where
    T: rkyv::Archive + Clone + Debug + PartialEq + 'static,
{}
#[automatically_derived]
impl<T: ::core::cmp::PartialEq> ::core::cmp::PartialEq for GenericBlob<T>
where
    T: rkyv::Archive + Clone + Debug + PartialEq + 'static,
{
    #[inline]
    fn eq(&self, other: &GenericBlob<T>) -> bool {
        self.data == other.data
    }
}
#[automatically_derived]
impl<T: ::core::cmp::Eq> ::core::cmp::Eq for GenericBlob<T>
where
    T: rkyv::Archive + Clone + Debug + PartialEq + 'static,
{
    #[inline]
    #[doc(hidden)]
    #[coverage(off)]
    fn assert_fields_are_eq(&self) {
        let _: ::core::cmp::AssertParamIsEq<T>;
    }
}
pub struct GenericBlobChunk {
    pub index: usize,
    pub data: Vec<u8>,
}
#[automatically_derived]
impl ::core::fmt::Debug for GenericBlobChunk {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::debug_struct_field2_finish(
            f,
            "GenericBlobChunk",
            "index",
            &self.index,
            "data",
            &&self.data,
        )
    }
}
#[automatically_derived]
impl ::core::clone::Clone for GenericBlobChunk {
    #[inline]
    fn clone(&self) -> GenericBlobChunk {
        GenericBlobChunk {
            index: ::core::clone::Clone::clone(&self.index),
            data: ::core::clone::Clone::clone(&self.data),
        }
    }
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for GenericBlobChunk {}
#[automatically_derived]
impl ::core::cmp::PartialEq for GenericBlobChunk {
    #[inline]
    fn eq(&self, other: &GenericBlobChunk) -> bool {
        self.index == other.index && self.data == other.data
    }
}
#[automatically_derived]
impl ::core::cmp::Eq for GenericBlobChunk {
    #[inline]
    #[doc(hidden)]
    #[coverage(off)]
    fn assert_fields_are_eq(&self) {
        let _: ::core::cmp::AssertParamIsEq<usize>;
        let _: ::core::cmp::AssertParamIsEq<Vec<u8>>;
    }
}
#[automatically_derived]
impl ::core::cmp::PartialOrd for GenericBlobChunk {
    #[inline]
    fn partial_cmp(
        &self,
        other: &GenericBlobChunk,
    ) -> ::core::option::Option<::core::cmp::Ordering> {
        match ::core::cmp::PartialOrd::partial_cmp(&self.index, &other.index) {
            ::core::option::Option::Some(::core::cmp::Ordering::Equal) => {
                ::core::cmp::PartialOrd::partial_cmp(&self.data, &other.data)
            }
            cmp => cmp,
        }
    }
}
#[automatically_derived]
impl ::core::cmp::Ord for GenericBlobChunk {
    #[inline]
    fn cmp(&self, other: &GenericBlobChunk) -> ::core::cmp::Ordering {
        match ::core::cmp::Ord::cmp(&self.index, &other.index) {
            ::core::cmp::Ordering::Equal => {
                ::core::cmp::Ord::cmp(&self.data, &other.data)
            }
            cmp => cmp,
        }
    }
}
#[automatically_derived]
///An archived [`GenericBlobChunk`]
#[bytecheck(crate = ::rkyv::bytecheck)]
#[repr(C)]
pub struct ArchivedGenericBlobChunk
where
    usize: ::rkyv::Archive,
    Vec<u8>: ::rkyv::Archive,
{
    ///The archived counterpart of [`GenericBlobChunk::index`]
    pub index: <usize as ::rkyv::Archive>::Archived,
    ///The archived counterpart of [`GenericBlobChunk::data`]
    pub data: <Vec<u8> as ::rkyv::Archive>::Archived,
}
#[automatically_derived]
unsafe impl<
    __C: ::rkyv::bytecheck::rancor::Fallible + ?::core::marker::Sized,
> ::rkyv::bytecheck::CheckBytes<__C> for ArchivedGenericBlobChunk
where
    usize: ::rkyv::Archive,
    Vec<u8>: ::rkyv::Archive,
    <__C as ::rkyv::bytecheck::rancor::Fallible>::Error: ::rkyv::bytecheck::rancor::Trace,
    <usize as ::rkyv::Archive>::Archived: ::rkyv::bytecheck::CheckBytes<__C>,
    <Vec<u8> as ::rkyv::Archive>::Archived: ::rkyv::bytecheck::CheckBytes<__C>,
{
    unsafe fn check_bytes(
        value: *const Self,
        context: &mut __C,
    ) -> ::core::result::Result<
        (),
        <__C as ::rkyv::bytecheck::rancor::Fallible>::Error,
    > {
        <<usize as ::rkyv::Archive>::Archived as ::rkyv::bytecheck::CheckBytes<
            __C,
        >>::check_bytes(&raw const (*value).index, context)
            .map_err(|e| {
                <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Trace>::trace(
                    e,
                    ::rkyv::bytecheck::StructCheckContext {
                        struct_name: "ArchivedGenericBlobChunk",
                        field_name: "index",
                    },
                )
            })?;
        <<Vec<
            u8,
        > as ::rkyv::Archive>::Archived as ::rkyv::bytecheck::CheckBytes<
            __C,
        >>::check_bytes(&raw const (*value).data, context)
            .map_err(|e| {
                <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Trace>::trace(
                    e,
                    ::rkyv::bytecheck::StructCheckContext {
                        struct_name: "ArchivedGenericBlobChunk",
                        field_name: "data",
                    },
                )
            })?;
        ::core::result::Result::Ok(())
    }
}
#[automatically_derived]
///The resolver for an archived [`GenericBlobChunk`]
pub struct GenericBlobChunkResolver
where
    usize: ::rkyv::Archive,
    Vec<u8>: ::rkyv::Archive,
{
    index: <usize as ::rkyv::Archive>::Resolver,
    data: <Vec<u8> as ::rkyv::Archive>::Resolver,
}
impl ::rkyv::Archive for GenericBlobChunk
where
    usize: ::rkyv::Archive,
    Vec<u8>: ::rkyv::Archive,
{
    type Archived = ArchivedGenericBlobChunk;
    type Resolver = GenericBlobChunkResolver;
    const COPY_OPTIMIZATION: ::rkyv::traits::CopyOptimization<Self> = unsafe {
        ::rkyv::traits::CopyOptimization::enable_if(
            0 + ::core::mem::size_of::<usize>() + ::core::mem::size_of::<Vec<u8>>()
                == ::core::mem::size_of::<GenericBlobChunk>()
                && <usize as ::rkyv::Archive>::COPY_OPTIMIZATION.is_enabled()
                && const { builtin # offset_of(GenericBlobChunk, index) }
                    == const { builtin # offset_of(ArchivedGenericBlobChunk, index) }
                && <Vec<u8> as ::rkyv::Archive>::COPY_OPTIMIZATION.is_enabled()
                && const { builtin # offset_of(GenericBlobChunk, data) }
                    == const { builtin # offset_of(ArchivedGenericBlobChunk, data) },
        )
    };
    #[allow(clippy::unit_arg)]
    fn resolve(&self, resolver: Self::Resolver, out: ::rkyv::Place<Self::Archived>) {
        let field_ptr = unsafe { &raw mut (*out.ptr()).index };
        let field_out = unsafe { ::rkyv::Place::from_field_unchecked(out, field_ptr) };
        <usize as ::rkyv::Archive>::resolve(&self.index, resolver.index, field_out);
        let field_ptr = unsafe { &raw mut (*out.ptr()).data };
        let field_out = unsafe { ::rkyv::Place::from_field_unchecked(out, field_ptr) };
        <Vec<u8> as ::rkyv::Archive>::resolve(&self.data, resolver.data, field_out);
    }
}
unsafe impl ::rkyv::traits::Portable for ArchivedGenericBlobChunk
where
    usize: ::rkyv::Archive,
    Vec<u8>: ::rkyv::Archive,
    <usize as ::rkyv::Archive>::Archived: ::rkyv::traits::Portable,
    <Vec<u8> as ::rkyv::Archive>::Archived: ::rkyv::traits::Portable,
{}
#[automatically_derived]
impl<__S: ::rkyv::rancor::Fallible + ?Sized> ::rkyv::Serialize<__S> for GenericBlobChunk
where
    usize: ::rkyv::Serialize<__S>,
    Vec<u8>: ::rkyv::Serialize<__S>,
{
    fn serialize(
        &self,
        serializer: &mut __S,
    ) -> ::core::result::Result<
        <Self as ::rkyv::Archive>::Resolver,
        <__S as ::rkyv::rancor::Fallible>::Error,
    > {
        let __this = self;
        ::core::result::Result::Ok(GenericBlobChunkResolver {
            index: <usize as ::rkyv::Serialize<
                __S,
            >>::serialize(&__this.index, serializer)?,
            data: <Vec<
                u8,
            > as ::rkyv::Serialize<__S>>::serialize(&__this.data, serializer)?,
        })
    }
}
#[automatically_derived]
impl<__D: ::rkyv::rancor::Fallible + ?Sized> ::rkyv::Deserialize<GenericBlobChunk, __D>
for ::rkyv::Archived<GenericBlobChunk>
where
    usize: ::rkyv::Archive,
    <usize as ::rkyv::Archive>::Archived: ::rkyv::Deserialize<usize, __D>,
    Vec<u8>: ::rkyv::Archive,
    <Vec<u8> as ::rkyv::Archive>::Archived: ::rkyv::Deserialize<Vec<u8>, __D>,
{
    fn deserialize(
        &self,
        deserializer: &mut __D,
    ) -> ::core::result::Result<
        GenericBlobChunk,
        <__D as ::rkyv::rancor::Fallible>::Error,
    > {
        let __this = self;
        ::core::result::Result::Ok(GenericBlobChunk {
            index: <<usize as ::rkyv::Archive>::Archived as ::rkyv::Deserialize<
                usize,
                __D,
            >>::deserialize(&__this.index, deserializer)?,
            data: <<Vec<
                u8,
            > as ::rkyv::Archive>::Archived as ::rkyv::Deserialize<
                Vec<u8>,
                __D,
            >>::deserialize(&__this.data, deserializer)?,
        })
    }
}
impl ::rewrite::traits::structural::blob::BlobItemChunk for GenericBlobChunk {
    type Index = usize;
    fn get_index(&self) -> &Self::Index {
        &self.index
    }
}
pub enum GenericBlobChunkFill {
    Full(usize),
    Partial(usize),
    Corrupted(usize),
}
#[automatically_derived]
impl ::core::fmt::Debug for GenericBlobChunkFill {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match self {
            GenericBlobChunkFill::Full(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "Full", &__self_0)
            }
            GenericBlobChunkFill::Partial(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(
                    f,
                    "Partial",
                    &__self_0,
                )
            }
            GenericBlobChunkFill::Corrupted(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(
                    f,
                    "Corrupted",
                    &__self_0,
                )
            }
        }
    }
}
#[automatically_derived]
#[doc(hidden)]
unsafe impl ::core::clone::TrivialClone for GenericBlobChunkFill {}
#[automatically_derived]
impl ::core::clone::Clone for GenericBlobChunkFill {
    #[inline]
    fn clone(&self) -> GenericBlobChunkFill {
        let _: ::core::clone::AssertParamIsClone<usize>;
        *self
    }
}
#[automatically_derived]
impl ::core::marker::Copy for GenericBlobChunkFill {}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for GenericBlobChunkFill {}
#[automatically_derived]
impl ::core::cmp::PartialEq for GenericBlobChunkFill {
    #[inline]
    fn eq(&self, other: &GenericBlobChunkFill) -> bool {
        let __self_discr = ::core::intrinsics::discriminant_value(self);
        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
        __self_discr == __arg1_discr
            && match (self, other) {
                (
                    GenericBlobChunkFill::Full(__self_0),
                    GenericBlobChunkFill::Full(__arg1_0),
                ) => __self_0 == __arg1_0,
                (
                    GenericBlobChunkFill::Partial(__self_0),
                    GenericBlobChunkFill::Partial(__arg1_0),
                ) => __self_0 == __arg1_0,
                (
                    GenericBlobChunkFill::Corrupted(__self_0),
                    GenericBlobChunkFill::Corrupted(__arg1_0),
                ) => __self_0 == __arg1_0,
                _ => unsafe { ::core::intrinsics::unreachable() }
            }
    }
}
#[automatically_derived]
impl ::core::cmp::Eq for GenericBlobChunkFill {
    #[inline]
    #[doc(hidden)]
    #[coverage(off)]
    fn assert_fields_are_eq(&self) {
        let _: ::core::cmp::AssertParamIsEq<usize>;
    }
}
#[automatically_derived]
impl ::core::cmp::PartialOrd for GenericBlobChunkFill {
    #[inline]
    fn partial_cmp(
        &self,
        other: &GenericBlobChunkFill,
    ) -> ::core::option::Option<::core::cmp::Ordering> {
        let __self_discr = ::core::intrinsics::discriminant_value(self);
        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
        match (self, other) {
            (
                GenericBlobChunkFill::Full(__self_0),
                GenericBlobChunkFill::Full(__arg1_0),
            ) => ::core::cmp::PartialOrd::partial_cmp(__self_0, __arg1_0),
            (
                GenericBlobChunkFill::Partial(__self_0),
                GenericBlobChunkFill::Partial(__arg1_0),
            ) => ::core::cmp::PartialOrd::partial_cmp(__self_0, __arg1_0),
            (
                GenericBlobChunkFill::Corrupted(__self_0),
                GenericBlobChunkFill::Corrupted(__arg1_0),
            ) => ::core::cmp::PartialOrd::partial_cmp(__self_0, __arg1_0),
            _ => ::core::cmp::PartialOrd::partial_cmp(&__self_discr, &__arg1_discr),
        }
    }
}
#[automatically_derived]
impl ::core::cmp::Ord for GenericBlobChunkFill {
    #[inline]
    fn cmp(&self, other: &GenericBlobChunkFill) -> ::core::cmp::Ordering {
        let __self_discr = ::core::intrinsics::discriminant_value(self);
        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
        match ::core::cmp::Ord::cmp(&__self_discr, &__arg1_discr) {
            ::core::cmp::Ordering::Equal => {
                match (self, other) {
                    (
                        GenericBlobChunkFill::Full(__self_0),
                        GenericBlobChunkFill::Full(__arg1_0),
                    ) => ::core::cmp::Ord::cmp(__self_0, __arg1_0),
                    (
                        GenericBlobChunkFill::Partial(__self_0),
                        GenericBlobChunkFill::Partial(__arg1_0),
                    ) => ::core::cmp::Ord::cmp(__self_0, __arg1_0),
                    (
                        GenericBlobChunkFill::Corrupted(__self_0),
                        GenericBlobChunkFill::Corrupted(__arg1_0),
                    ) => ::core::cmp::Ord::cmp(__self_0, __arg1_0),
                    _ => unsafe { ::core::intrinsics::unreachable() }
                }
            }
            cmp => cmp,
        }
    }
}
#[automatically_derived]
///An archived [`GenericBlobChunkFill`]
#[bytecheck(crate = ::rkyv::bytecheck)]
#[repr(u8)]
pub enum ArchivedGenericBlobChunkFill
where
    usize: ::rkyv::Archive,
    usize: ::rkyv::Archive,
    usize: ::rkyv::Archive,
{
    ///The archived counterpart of [`GenericBlobChunkFill::Full`]
    #[allow(dead_code)]
    Full(
        ///The archived counterpart of [`GenericBlobChunkFill::Full::0`]
        <usize as ::rkyv::Archive>::Archived,
    ),
    ///The archived counterpart of [`GenericBlobChunkFill::Partial`]
    #[allow(dead_code)]
    Partial(
        ///The archived counterpart of [`GenericBlobChunkFill::Partial::0`]
        <usize as ::rkyv::Archive>::Archived,
    ),
    ///The archived counterpart of [`GenericBlobChunkFill::Corrupted`]
    #[allow(dead_code)]
    Corrupted(
        ///The archived counterpart of [`GenericBlobChunkFill::Corrupted::0`]
        <usize as ::rkyv::Archive>::Archived,
    ),
}
const _: () = {
    #[repr(u8)]
    enum Tag {
        Full,
        Partial,
        Corrupted,
    }
    struct Discriminant;
    #[automatically_derived]
    impl Discriminant {
        #[allow(non_upper_case_globals)]
        const Full: u8 = Tag::Full as u8;
        #[allow(non_upper_case_globals)]
        const Partial: u8 = Tag::Partial as u8;
        #[allow(non_upper_case_globals)]
        const Corrupted: u8 = Tag::Corrupted as u8;
    }
    #[repr(C)]
    struct VariantFull(
        Tag,
        <usize as ::rkyv::Archive>::Archived,
        ::core::marker::PhantomData<ArchivedGenericBlobChunkFill>,
    )
    where
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive;
    #[repr(C)]
    struct VariantPartial(
        Tag,
        <usize as ::rkyv::Archive>::Archived,
        ::core::marker::PhantomData<ArchivedGenericBlobChunkFill>,
    )
    where
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive;
    #[repr(C)]
    struct VariantCorrupted(
        Tag,
        <usize as ::rkyv::Archive>::Archived,
        ::core::marker::PhantomData<ArchivedGenericBlobChunkFill>,
    )
    where
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive;
    #[automatically_derived]
    unsafe impl<
        __C: ::rkyv::bytecheck::rancor::Fallible + ?::core::marker::Sized,
    > ::rkyv::bytecheck::CheckBytes<__C> for ArchivedGenericBlobChunkFill
    where
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive,
        <__C as ::rkyv::bytecheck::rancor::Fallible>::Error: ::rkyv::bytecheck::rancor::Source,
        <usize as ::rkyv::Archive>::Archived: ::rkyv::bytecheck::CheckBytes<__C>,
        <usize as ::rkyv::Archive>::Archived: ::rkyv::bytecheck::CheckBytes<__C>,
        <usize as ::rkyv::Archive>::Archived: ::rkyv::bytecheck::CheckBytes<__C>,
    {
        unsafe fn check_bytes(
            value: *const Self,
            context: &mut __C,
        ) -> ::core::result::Result<
            (),
            <__C as ::rkyv::bytecheck::rancor::Fallible>::Error,
        > {
            let tag = *value.cast::<u8>();
            match tag {
                Discriminant::Full => {
                    let value = value.cast::<VariantFull>();
                    <<usize as ::rkyv::Archive>::Archived as ::rkyv::bytecheck::CheckBytes<
                        __C,
                    >>::check_bytes(&raw const (*value).1, context)
                        .map_err(|e| {
                            <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Trace>::trace(
                                e,
                                ::rkyv::bytecheck::UnnamedEnumVariantCheckContext {
                                    enum_name: "ArchivedGenericBlobChunkFill",
                                    variant_name: "Full",
                                    field_index: 1,
                                },
                            )
                        })?;
                }
                Discriminant::Partial => {
                    let value = value.cast::<VariantPartial>();
                    <<usize as ::rkyv::Archive>::Archived as ::rkyv::bytecheck::CheckBytes<
                        __C,
                    >>::check_bytes(&raw const (*value).1, context)
                        .map_err(|e| {
                            <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Trace>::trace(
                                e,
                                ::rkyv::bytecheck::UnnamedEnumVariantCheckContext {
                                    enum_name: "ArchivedGenericBlobChunkFill",
                                    variant_name: "Partial",
                                    field_index: 1,
                                },
                            )
                        })?;
                }
                Discriminant::Corrupted => {
                    let value = value.cast::<VariantCorrupted>();
                    <<usize as ::rkyv::Archive>::Archived as ::rkyv::bytecheck::CheckBytes<
                        __C,
                    >>::check_bytes(&raw const (*value).1, context)
                        .map_err(|e| {
                            <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Trace>::trace(
                                e,
                                ::rkyv::bytecheck::UnnamedEnumVariantCheckContext {
                                    enum_name: "ArchivedGenericBlobChunkFill",
                                    variant_name: "Corrupted",
                                    field_index: 1,
                                },
                            )
                        })?;
                }
                _ => {
                    return ::core::result::Result::Err(
                        <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Source>::new(::rkyv::bytecheck::InvalidEnumDiscriminantError {
                            enum_name: "ArchivedGenericBlobChunkFill",
                            invalid_discriminant: tag,
                        }),
                    );
                }
            }
            ::core::result::Result::Ok(())
        }
    }
};
#[automatically_derived]
///The resolver for an archived [`GenericBlobChunkFill`]
pub enum GenericBlobChunkFillResolver
where
    usize: ::rkyv::Archive,
    usize: ::rkyv::Archive,
    usize: ::rkyv::Archive,
{
    ///The resolver for [`GenericBlobChunkFill::Full`]
    #[allow(dead_code)]
    Full(<usize as ::rkyv::Archive>::Resolver),
    ///The resolver for [`GenericBlobChunkFill::Partial`]
    #[allow(dead_code)]
    Partial(<usize as ::rkyv::Archive>::Resolver),
    ///The resolver for [`GenericBlobChunkFill::Corrupted`]
    #[allow(dead_code)]
    Corrupted(<usize as ::rkyv::Archive>::Resolver),
}
const _: () = {
    #[repr(u8)]
    enum ArchivedTag {
        Full,
        Partial,
        Corrupted,
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for ArchivedTag {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for ArchivedTag {
        #[inline]
        fn eq(&self, other: &ArchivedTag) -> bool {
            let __self_discr = ::core::intrinsics::discriminant_value(self);
            let __arg1_discr = ::core::intrinsics::discriminant_value(other);
            __self_discr == __arg1_discr
        }
    }
    #[automatically_derived]
    impl ::core::cmp::PartialOrd for ArchivedTag {
        #[inline]
        fn partial_cmp(
            &self,
            other: &ArchivedTag,
        ) -> ::core::option::Option<::core::cmp::Ordering> {
            let __self_discr = ::core::intrinsics::discriminant_value(self);
            let __arg1_discr = ::core::intrinsics::discriminant_value(other);
            ::core::cmp::PartialOrd::partial_cmp(&__self_discr, &__arg1_discr)
        }
    }
    #[repr(C)]
    struct ArchivedVariantFull(
        ArchivedTag,
        <usize as ::rkyv::Archive>::Archived,
        ::core::marker::PhantomData<GenericBlobChunkFill>,
    )
    where
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive;
    #[repr(C)]
    struct ArchivedVariantPartial(
        ArchivedTag,
        <usize as ::rkyv::Archive>::Archived,
        ::core::marker::PhantomData<GenericBlobChunkFill>,
    )
    where
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive;
    #[repr(C)]
    struct ArchivedVariantCorrupted(
        ArchivedTag,
        <usize as ::rkyv::Archive>::Archived,
        ::core::marker::PhantomData<GenericBlobChunkFill>,
    )
    where
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive;
    impl ::rkyv::Archive for GenericBlobChunkFill
    where
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive,
    {
        type Archived = ArchivedGenericBlobChunkFill;
        type Resolver = GenericBlobChunkFillResolver;
        #[allow(clippy::unit_arg)]
        fn resolve(
            &self,
            resolver: <Self as ::rkyv::Archive>::Resolver,
            out: ::rkyv::Place<<Self as ::rkyv::Archive>::Archived>,
        ) {
            let __this = self;
            match resolver {
                GenericBlobChunkFillResolver::Full(resolver_0) => {
                    match __this {
                        GenericBlobChunkFill::Full(self_0, ..) => {
                            let out = unsafe {
                                out.cast_unchecked::<ArchivedVariantFull>()
                            };
                            let tag_ptr = unsafe { &raw mut (*out.ptr()).0 };
                            unsafe {
                                tag_ptr.write(ArchivedTag::Full);
                            }
                            let field_ptr = unsafe { &raw mut (*out.ptr()).1 };
                            let out_field = unsafe {
                                ::rkyv::Place::from_field_unchecked(out, field_ptr)
                            };
                            <usize as ::rkyv::Archive>::resolve(
                                self_0,
                                resolver_0,
                                out_field,
                            );
                        }
                        #[allow(unreachable_patterns)]
                        _ => unsafe { ::core::hint::unreachable_unchecked() }
                    }
                }
                GenericBlobChunkFillResolver::Partial(resolver_0) => {
                    match __this {
                        GenericBlobChunkFill::Partial(self_0, ..) => {
                            let out = unsafe {
                                out.cast_unchecked::<ArchivedVariantPartial>()
                            };
                            let tag_ptr = unsafe { &raw mut (*out.ptr()).0 };
                            unsafe {
                                tag_ptr.write(ArchivedTag::Partial);
                            }
                            let field_ptr = unsafe { &raw mut (*out.ptr()).1 };
                            let out_field = unsafe {
                                ::rkyv::Place::from_field_unchecked(out, field_ptr)
                            };
                            <usize as ::rkyv::Archive>::resolve(
                                self_0,
                                resolver_0,
                                out_field,
                            );
                        }
                        #[allow(unreachable_patterns)]
                        _ => unsafe { ::core::hint::unreachable_unchecked() }
                    }
                }
                GenericBlobChunkFillResolver::Corrupted(resolver_0) => {
                    match __this {
                        GenericBlobChunkFill::Corrupted(self_0, ..) => {
                            let out = unsafe {
                                out.cast_unchecked::<ArchivedVariantCorrupted>()
                            };
                            let tag_ptr = unsafe { &raw mut (*out.ptr()).0 };
                            unsafe {
                                tag_ptr.write(ArchivedTag::Corrupted);
                            }
                            let field_ptr = unsafe { &raw mut (*out.ptr()).1 };
                            let out_field = unsafe {
                                ::rkyv::Place::from_field_unchecked(out, field_ptr)
                            };
                            <usize as ::rkyv::Archive>::resolve(
                                self_0,
                                resolver_0,
                                out_field,
                            );
                        }
                        #[allow(unreachable_patterns)]
                        _ => unsafe { ::core::hint::unreachable_unchecked() }
                    }
                }
            }
        }
    }
};
unsafe impl ::rkyv::traits::Portable for ArchivedGenericBlobChunkFill
where
    usize: ::rkyv::Archive,
    usize: ::rkyv::Archive,
    usize: ::rkyv::Archive,
    <usize as ::rkyv::Archive>::Archived: ::rkyv::traits::Portable,
    <usize as ::rkyv::Archive>::Archived: ::rkyv::traits::Portable,
    <usize as ::rkyv::Archive>::Archived: ::rkyv::traits::Portable,
{}
#[automatically_derived]
impl<__S: ::rkyv::rancor::Fallible + ?Sized> ::rkyv::Serialize<__S>
for GenericBlobChunkFill
where
    usize: ::rkyv::Serialize<__S>,
    usize: ::rkyv::Serialize<__S>,
    usize: ::rkyv::Serialize<__S>,
{
    fn serialize(
        &self,
        serializer: &mut __S,
    ) -> ::core::result::Result<
        <Self as ::rkyv::Archive>::Resolver,
        <__S as ::rkyv::rancor::Fallible>::Error,
    > {
        let __this = self;
        ::core::result::Result::Ok(
            match __this {
                GenericBlobChunkFill::Full(_0, ..) => {
                    GenericBlobChunkFillResolver::Full(
                        <usize as ::rkyv::Serialize<__S>>::serialize(_0, serializer)?,
                    )
                }
                GenericBlobChunkFill::Partial(_0, ..) => {
                    GenericBlobChunkFillResolver::Partial(
                        <usize as ::rkyv::Serialize<__S>>::serialize(_0, serializer)?,
                    )
                }
                GenericBlobChunkFill::Corrupted(_0, ..) => {
                    GenericBlobChunkFillResolver::Corrupted(
                        <usize as ::rkyv::Serialize<__S>>::serialize(_0, serializer)?,
                    )
                }
            },
        )
    }
}
#[automatically_derived]
impl<
    __D: ::rkyv::rancor::Fallible + ?Sized,
> ::rkyv::Deserialize<GenericBlobChunkFill, __D>
for ::rkyv::Archived<GenericBlobChunkFill>
where
    usize: ::rkyv::Archive,
    <usize as ::rkyv::Archive>::Archived: ::rkyv::Deserialize<usize, __D>,
    usize: ::rkyv::Archive,
    <usize as ::rkyv::Archive>::Archived: ::rkyv::Deserialize<usize, __D>,
    usize: ::rkyv::Archive,
    <usize as ::rkyv::Archive>::Archived: ::rkyv::Deserialize<usize, __D>,
{
    fn deserialize(
        &self,
        deserializer: &mut __D,
    ) -> ::core::result::Result<
        GenericBlobChunkFill,
        <__D as ::rkyv::rancor::Fallible>::Error,
    > {
        let __this = self;
        ::core::result::Result::Ok(
            match __this {
                Self::Full(_0, ..) => {
                    GenericBlobChunkFill::Full(
                        <<usize as ::rkyv::Archive>::Archived as ::rkyv::Deserialize<
                            usize,
                            __D,
                        >>::deserialize(_0, deserializer)?,
                    )
                }
                Self::Partial(_0, ..) => {
                    GenericBlobChunkFill::Partial(
                        <<usize as ::rkyv::Archive>::Archived as ::rkyv::Deserialize<
                            usize,
                            __D,
                        >>::deserialize(_0, deserializer)?,
                    )
                }
                Self::Corrupted(_0, ..) => {
                    GenericBlobChunkFill::Corrupted(
                        <<usize as ::rkyv::Archive>::Archived as ::rkyv::Deserialize<
                            usize,
                            __D,
                        >>::deserialize(_0, deserializer)?,
                    )
                }
            },
        )
    }
}
impl GenericBlobChunkFill {
    pub fn from_size(actual: usize, expected: usize) -> Self {
        if actual == expected {
            Self::Full(actual)
        } else if actual < expected {
            Self::Partial(actual)
        } else {
            Self::Corrupted(actual)
        }
    }
}
impl<T> rewrite::traits::structural::blob::NetabaseBlobItem for GenericBlob<T>
where
    T: rkyv::Archive + Clone + Debug + PartialEq + 'static,
    T: rkyv::Archive,
    T: for<'__a> rkyv::Serialize<
        rkyv::rancor::Strategy<
            rkyv::ser::Serializer<
                rkyv::util::AlignedVec,
                rkyv::ser::allocator::ArenaHandle<'__a>,
                rkyv::ser::sharing::Share,
            >,
            rkyv::rancor::Error,
        >,
    >,
    <T as rkyv::Archive>::Archived: for<'__a> rkyv::bytecheck::CheckBytes<
        rkyv::rancor::Strategy<
            rkyv::validation::Validator<
                rkyv::validation::archive::ArchiveValidator<'__a>,
                rkyv::validation::shared::SharedValidator,
            >,
            rkyv::rancor::Error,
        >,
    >,
    <T as rkyv::Archive>::Archived: rkyv::Deserialize<
        T,
        rkyv::rancor::Strategy<rkyv::de::Pool, rkyv::rancor::Error>,
    >,
{
    type Chunk = GenericBlobChunk;
    type BlobIter = std::vec::IntoIter<rewrite::results::NetabaseResult<Self::Chunk>>;
    const DEFAULT_CHUNK_SIZE: usize = 0usize;
    fn into_chunks(
        self,
        size: rewrite::traits::structural::blob::ChunkSize,
    ) -> Box<dyn Iterator<Item = Self::Chunk>> {
        Box::new(self.into_chunks_iter(size).filter_map(|r| r.ok()))
    }
    fn into_chunks_iter(
        self,
        size: rewrite::traits::structural::blob::ChunkSize,
    ) -> Self::BlobIter {
        let serialized_data: rewrite::results::NetabaseResult<Vec<u8>> = (|| -> rewrite::results::NetabaseResult<
            Vec<u8>,
        > {
            Ok(
                rkyv::to_bytes::<rkyv::rancor::Error>(&self)
                    .map_err(|e| rewrite::results::NetabaseError::Serialization(
                        ::alloc::__export::must_use({
                            ::alloc::fmt::format(
                                format_args!("rkyv serialization failed: {0:?}", e),
                            )
                        }),
                    ))?
                    .to_vec(),
            )
        })();
        let chunk_size = match size {
            rewrite::traits::structural::blob::ChunkSize::Default => {
                if Self::DEFAULT_CHUNK_SIZE > 0 {
                    Self::DEFAULT_CHUNK_SIZE
                } else {
                    1024
                }
            }
            rewrite::traits::structural::blob::ChunkSize::Size(n) => n,
        };
        match serialized_data {
            Ok(data) => {
                data.chunks(chunk_size)
                    .enumerate()
                    .map(|(index, chunk_data)| {
                        Ok(Self::Chunk {
                            index,
                            data: chunk_data.to_vec(),
                        })
                    })
                    .collect::<Vec<_>>()
                    .into_iter()
            }
            Err(e) => {
                ::alloc::boxed::box_assume_init_into_vec_unsafe(
                        ::alloc::intrinsics::write_box_via_move(
                            ::alloc::boxed::Box::new_uninit(),
                            [Err(e)],
                        ),
                    )
                    .into_iter()
            }
        }
    }
    fn try_from_chunks(
        chunks: impl Iterator<Item = Self::Chunk>,
        size: rewrite::traits::structural::blob::ChunkSize,
    ) -> rewrite::results::NetabaseResult<Self> {
        let mut sorted_chunks: Vec<_> = chunks.collect();
        sorted_chunks.sort_by_key(|c| c.index);
        if sorted_chunks.is_empty() {
            return Err(
                rewrite::results::NetabaseError::BlobReconstruction(
                    rewrite::results::BlobReconstructionError::MissingChunks,
                ),
            );
        }
        let chunk_size = match size {
            rewrite::traits::structural::blob::ChunkSize::Default => {
                if Self::DEFAULT_CHUNK_SIZE > 0 {
                    Self::DEFAULT_CHUNK_SIZE
                } else {
                    1024
                }
            }
            rewrite::traits::structural::blob::ChunkSize::Size(n) => n,
        };
        let mut missing_details = Vec::new();
        let mut next_expected = 0;
        let max_idx = sorted_chunks.last().map(|c| c.index).unwrap_or(0);
        for chunk in &sorted_chunks {
            while chunk.index > next_expected {
                missing_details
                    .push(
                        ::alloc::__export::must_use({
                            ::alloc::fmt::format(
                                format_args!(
                                    "{0:?}({{ Index: {1}, Size: {2} }})",
                                    GenericBlobChunkFill::Full(chunk_size),
                                    next_expected,
                                    chunk_size,
                                ),
                            )
                        }),
                    );
                next_expected += 1;
            }
            let fill = GenericBlobChunkFill::from_size(chunk.data.len(), chunk_size);
            match fill {
                GenericBlobChunkFill::Corrupted(size) => {
                    return Err(
                        rewrite::results::NetabaseError::BlobReconstruction(
                            rewrite::results::BlobReconstructionError::InvalidChunkData(
                                ::alloc::__export::must_use({
                                    ::alloc::fmt::format(
                                        format_args!(
                                            "Corrupted chunk detected: {0:?}({{ Index: {1}, Size: {2} }}). Max allowed size is {3}.",
                                            fill,
                                            chunk.index,
                                            size,
                                            chunk_size,
                                        ),
                                    )
                                }),
                            ),
                        ),
                    );
                }
                GenericBlobChunkFill::Partial(size) if chunk.index < max_idx => {
                    return Err(
                        rewrite::results::NetabaseError::BlobReconstruction(
                            rewrite::results::BlobReconstructionError::InvalidChunkData(
                                ::alloc::__export::must_use({
                                    ::alloc::fmt::format(
                                        format_args!(
                                            "Unexpected partial chunk in middle of stream: {0:?}({{ Index: {1}, Size: {2} }}). Expected {3} bytes.",
                                            fill,
                                            chunk.index,
                                            size,
                                            chunk_size,
                                        ),
                                    )
                                }),
                            ),
                        ),
                    );
                }
                _ => {}
            }
            if chunk.index == next_expected {
                next_expected += 1;
            }
        }
        if !missing_details.is_empty() {
            if let Some(last) = sorted_chunks.last() {
                let fill = GenericBlobChunkFill::from_size(last.data.len(), chunk_size);
                if #[allow(non_exhaustive_omitted_patterns)]
                match fill {
                    GenericBlobChunkFill::Full(_) => true,
                    _ => false,
                } {
                    missing_details
                        .push(
                            ::alloc::__export::must_use({
                                ::alloc::fmt::format(
                                    format_args!(
                                        "... (Stream truncated: last chunk was Full, expected more data after Index {0})",
                                        last.index,
                                    ),
                                )
                            }),
                        );
                }
            }
        }
        if !missing_details.is_empty() {
            return Err(
                rewrite::results::NetabaseError::BlobReconstruction(
                    rewrite::results::BlobReconstructionError::InvalidChunkData(
                        ::alloc::__export::must_use({
                            ::alloc::fmt::format(
                                format_args!(
                                    "Missing chunks: [{0}]. Total chunks present: {1}",
                                    missing_details.join(", "),
                                    sorted_chunks.len(),
                                ),
                            )
                        }),
                    ),
                ),
            );
        }
        let serialized_data: Vec<u8> = sorted_chunks
            .into_iter()
            .flat_map(|c| c.data)
            .collect();
        Ok(
            rkyv::from_bytes::<Self, rkyv::rancor::Error>(&serialized_data)
                .map_err(|e| rewrite::results::NetabaseError::Serialization(
                    ::alloc::__export::must_use({
                        ::alloc::fmt::format(
                            format_args!("rkyv deserialization failed: {0:?}", e),
                        )
                    }),
                ))?,
        )
    }
    fn get_blob(&self) -> &Self::Chunk {
        {
            ::core::panicking::panic_fmt(
                format_args!(
                    "not implemented: {0}",
                    format_args!("get_blob() requires storing a chunk reference"),
                ),
            );
        }
    }
}
impl<T> IntoIterator for GenericBlob<T>
where
    T: rkyv::Archive + Clone + Debug + PartialEq + 'static,
    T: rkyv::Archive,
    T: for<'__a> rkyv::Serialize<
        rkyv::rancor::Strategy<
            rkyv::ser::Serializer<
                rkyv::util::AlignedVec,
                rkyv::ser::allocator::ArenaHandle<'__a>,
                rkyv::ser::sharing::Share,
            >,
            rkyv::rancor::Error,
        >,
    >,
    <T as rkyv::Archive>::Archived: for<'__a> rkyv::bytecheck::CheckBytes<
        rkyv::rancor::Strategy<
            rkyv::validation::Validator<
                rkyv::validation::archive::ArchiveValidator<'__a>,
                rkyv::validation::shared::SharedValidator,
            >,
            rkyv::rancor::Error,
        >,
    >,
    <T as rkyv::Archive>::Archived: rkyv::Deserialize<
        T,
        rkyv::rancor::Strategy<rkyv::de::Pool, rkyv::rancor::Error>,
    >,
{
    type Item = rewrite::results::NetabaseResult<GenericBlobChunk>;
    type IntoIter = std::vec::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        rewrite::traits::structural::blob::NetabaseBlobItem::into_chunks_iter(
            self,
            rewrite::traits::structural::blob::ChunkSize::Default,
        )
    }
}
enum PartialComplexEnum {
    Full(String),
    #[blob_field(chunk_size(64))]
    Partial { #[chunk_size(32)] meta: String, payload: Vec<u8> },
}
#[automatically_derived]
///An archived [`PartialComplexEnum`]
#[bytecheck(crate = ::rkyv::bytecheck)]
#[repr(u8)]
enum ArchivedPartialComplexEnum
where
    String: ::rkyv::Archive,
    String: ::rkyv::Archive,
    Vec<u8>: ::rkyv::Archive,
{
    ///The archived counterpart of [`PartialComplexEnum::Full`]
    #[allow(dead_code)]
    Full(
        ///The archived counterpart of [`PartialComplexEnum::Full::0`]
        <String as ::rkyv::Archive>::Archived,
    ),
    ///The archived counterpart of [`PartialComplexEnum::Partial`]
    #[allow(dead_code)]
    Partial {
        ///The archived counterpart of [`PartialComplexEnum::Partial::meta`]
        meta: <String as ::rkyv::Archive>::Archived,
        ///The archived counterpart of [`PartialComplexEnum::Partial::payload`]
        payload: <Vec<u8> as ::rkyv::Archive>::Archived,
    },
}
const _: () = {
    #[repr(u8)]
    enum Tag {
        Full,
        Partial,
    }
    struct Discriminant;
    #[automatically_derived]
    impl Discriminant {
        #[allow(non_upper_case_globals)]
        const Full: u8 = Tag::Full as u8;
        #[allow(non_upper_case_globals)]
        const Partial: u8 = Tag::Partial as u8;
    }
    #[repr(C)]
    struct VariantFull(
        Tag,
        <String as ::rkyv::Archive>::Archived,
        ::core::marker::PhantomData<ArchivedPartialComplexEnum>,
    )
    where
        String: ::rkyv::Archive,
        String: ::rkyv::Archive,
        Vec<u8>: ::rkyv::Archive;
    #[repr(C)]
    struct VariantPartial
    where
        String: ::rkyv::Archive,
        String: ::rkyv::Archive,
        Vec<u8>: ::rkyv::Archive,
    {
        __tag: Tag,
        meta: <String as ::rkyv::Archive>::Archived,
        payload: <Vec<u8> as ::rkyv::Archive>::Archived,
        __phantom: ::core::marker::PhantomData<ArchivedPartialComplexEnum>,
    }
    #[automatically_derived]
    unsafe impl<
        __C: ::rkyv::bytecheck::rancor::Fallible + ?::core::marker::Sized,
    > ::rkyv::bytecheck::CheckBytes<__C> for ArchivedPartialComplexEnum
    where
        String: ::rkyv::Archive,
        String: ::rkyv::Archive,
        Vec<u8>: ::rkyv::Archive,
        <__C as ::rkyv::bytecheck::rancor::Fallible>::Error: ::rkyv::bytecheck::rancor::Source,
        <String as ::rkyv::Archive>::Archived: ::rkyv::bytecheck::CheckBytes<__C>,
        <String as ::rkyv::Archive>::Archived: ::rkyv::bytecheck::CheckBytes<__C>,
        <Vec<u8> as ::rkyv::Archive>::Archived: ::rkyv::bytecheck::CheckBytes<__C>,
    {
        unsafe fn check_bytes(
            value: *const Self,
            context: &mut __C,
        ) -> ::core::result::Result<
            (),
            <__C as ::rkyv::bytecheck::rancor::Fallible>::Error,
        > {
            let tag = *value.cast::<u8>();
            match tag {
                Discriminant::Full => {
                    let value = value.cast::<VariantFull>();
                    <<String as ::rkyv::Archive>::Archived as ::rkyv::bytecheck::CheckBytes<
                        __C,
                    >>::check_bytes(&raw const (*value).1, context)
                        .map_err(|e| {
                            <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Trace>::trace(
                                e,
                                ::rkyv::bytecheck::UnnamedEnumVariantCheckContext {
                                    enum_name: "ArchivedPartialComplexEnum",
                                    variant_name: "Full",
                                    field_index: 1,
                                },
                            )
                        })?;
                }
                Discriminant::Partial => {
                    let value = value.cast::<VariantPartial>();
                    <<String as ::rkyv::Archive>::Archived as ::rkyv::bytecheck::CheckBytes<
                        __C,
                    >>::check_bytes(&raw const (*value).meta, context)
                        .map_err(|e| {
                            <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Trace>::trace(
                                e,
                                ::rkyv::bytecheck::NamedEnumVariantCheckContext {
                                    enum_name: "ArchivedPartialComplexEnum",
                                    variant_name: "Partial",
                                    field_name: "meta",
                                },
                            )
                        })?;
                    <<Vec<
                        u8,
                    > as ::rkyv::Archive>::Archived as ::rkyv::bytecheck::CheckBytes<
                        __C,
                    >>::check_bytes(&raw const (*value).payload, context)
                        .map_err(|e| {
                            <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Trace>::trace(
                                e,
                                ::rkyv::bytecheck::NamedEnumVariantCheckContext {
                                    enum_name: "ArchivedPartialComplexEnum",
                                    variant_name: "Partial",
                                    field_name: "payload",
                                },
                            )
                        })?;
                }
                _ => {
                    return ::core::result::Result::Err(
                        <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Source>::new(::rkyv::bytecheck::InvalidEnumDiscriminantError {
                            enum_name: "ArchivedPartialComplexEnum",
                            invalid_discriminant: tag,
                        }),
                    );
                }
            }
            ::core::result::Result::Ok(())
        }
    }
};
#[automatically_derived]
///The resolver for an archived [`PartialComplexEnum`]
enum PartialComplexEnumResolver
where
    String: ::rkyv::Archive,
    String: ::rkyv::Archive,
    Vec<u8>: ::rkyv::Archive,
{
    ///The resolver for [`PartialComplexEnum::Full`]
    #[allow(dead_code)]
    Full(<String as ::rkyv::Archive>::Resolver),
    ///The resolver for [`PartialComplexEnum::Partial`]
    #[allow(dead_code)]
    Partial {
        meta: <String as ::rkyv::Archive>::Resolver,
        payload: <Vec<u8> as ::rkyv::Archive>::Resolver,
    },
}
const _: () = {
    #[repr(u8)]
    enum ArchivedTag {
        Full,
        Partial,
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for ArchivedTag {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for ArchivedTag {
        #[inline]
        fn eq(&self, other: &ArchivedTag) -> bool {
            let __self_discr = ::core::intrinsics::discriminant_value(self);
            let __arg1_discr = ::core::intrinsics::discriminant_value(other);
            __self_discr == __arg1_discr
        }
    }
    #[automatically_derived]
    impl ::core::cmp::PartialOrd for ArchivedTag {
        #[inline]
        fn partial_cmp(
            &self,
            other: &ArchivedTag,
        ) -> ::core::option::Option<::core::cmp::Ordering> {
            let __self_discr = ::core::intrinsics::discriminant_value(self);
            let __arg1_discr = ::core::intrinsics::discriminant_value(other);
            ::core::cmp::PartialOrd::partial_cmp(&__self_discr, &__arg1_discr)
        }
    }
    #[repr(C)]
    struct ArchivedVariantFull(
        ArchivedTag,
        <String as ::rkyv::Archive>::Archived,
        ::core::marker::PhantomData<PartialComplexEnum>,
    )
    where
        String: ::rkyv::Archive,
        String: ::rkyv::Archive,
        Vec<u8>: ::rkyv::Archive;
    #[repr(C)]
    struct ArchivedVariantPartial
    where
        String: ::rkyv::Archive,
        String: ::rkyv::Archive,
        Vec<u8>: ::rkyv::Archive,
    {
        __tag: ArchivedTag,
        meta: <String as ::rkyv::Archive>::Archived,
        payload: <Vec<u8> as ::rkyv::Archive>::Archived,
        __phantom: ::core::marker::PhantomData<PartialComplexEnum>,
    }
    impl ::rkyv::Archive for PartialComplexEnum
    where
        String: ::rkyv::Archive,
        String: ::rkyv::Archive,
        Vec<u8>: ::rkyv::Archive,
    {
        type Archived = ArchivedPartialComplexEnum;
        type Resolver = PartialComplexEnumResolver;
        #[allow(clippy::unit_arg)]
        fn resolve(
            &self,
            resolver: <Self as ::rkyv::Archive>::Resolver,
            out: ::rkyv::Place<<Self as ::rkyv::Archive>::Archived>,
        ) {
            let __this = self;
            match resolver {
                PartialComplexEnumResolver::Full(resolver_0) => {
                    match __this {
                        PartialComplexEnum::Full(self_0, ..) => {
                            let out = unsafe {
                                out.cast_unchecked::<ArchivedVariantFull>()
                            };
                            let tag_ptr = unsafe { &raw mut (*out.ptr()).0 };
                            unsafe {
                                tag_ptr.write(ArchivedTag::Full);
                            }
                            let field_ptr = unsafe { &raw mut (*out.ptr()).1 };
                            let out_field = unsafe {
                                ::rkyv::Place::from_field_unchecked(out, field_ptr)
                            };
                            <String as ::rkyv::Archive>::resolve(
                                self_0,
                                resolver_0,
                                out_field,
                            );
                        }
                        #[allow(unreachable_patterns)]
                        _ => unsafe { ::core::hint::unreachable_unchecked() }
                    }
                }
                PartialComplexEnumResolver::Partial {
                    meta: resolver_0,
                    payload: resolver_1,
                } => {
                    match __this {
                        PartialComplexEnum::Partial {
                            meta: self_0,
                            payload: self_1,
                            ..
                        } => {
                            let out = unsafe {
                                out.cast_unchecked::<ArchivedVariantPartial>()
                            };
                            let tag_ptr = unsafe { &raw mut (*out.ptr()).__tag };
                            unsafe {
                                tag_ptr.write(ArchivedTag::Partial);
                            }
                            let field_ptr = unsafe { &raw mut (*out.ptr()).meta };
                            let out_field = unsafe {
                                ::rkyv::Place::from_field_unchecked(out, field_ptr)
                            };
                            <String as ::rkyv::Archive>::resolve(
                                self_0,
                                resolver_0,
                                out_field,
                            );
                            let field_ptr = unsafe { &raw mut (*out.ptr()).payload };
                            let out_field = unsafe {
                                ::rkyv::Place::from_field_unchecked(out, field_ptr)
                            };
                            <Vec<
                                u8,
                            > as ::rkyv::Archive>::resolve(
                                self_1,
                                resolver_1,
                                out_field,
                            );
                        }
                        #[allow(unreachable_patterns)]
                        _ => unsafe { ::core::hint::unreachable_unchecked() }
                    }
                }
            }
        }
    }
};
unsafe impl ::rkyv::traits::Portable for ArchivedPartialComplexEnum
where
    String: ::rkyv::Archive,
    String: ::rkyv::Archive,
    Vec<u8>: ::rkyv::Archive,
    <String as ::rkyv::Archive>::Archived: ::rkyv::traits::Portable,
    <String as ::rkyv::Archive>::Archived: ::rkyv::traits::Portable,
    <Vec<u8> as ::rkyv::Archive>::Archived: ::rkyv::traits::Portable,
{}
#[automatically_derived]
impl<__S: ::rkyv::rancor::Fallible + ?Sized> ::rkyv::Serialize<__S>
for PartialComplexEnum
where
    String: ::rkyv::Serialize<__S>,
    String: ::rkyv::Serialize<__S>,
    Vec<u8>: ::rkyv::Serialize<__S>,
{
    fn serialize(
        &self,
        serializer: &mut __S,
    ) -> ::core::result::Result<
        <Self as ::rkyv::Archive>::Resolver,
        <__S as ::rkyv::rancor::Fallible>::Error,
    > {
        let __this = self;
        ::core::result::Result::Ok(
            match __this {
                PartialComplexEnum::Full(_0, ..) => {
                    PartialComplexEnumResolver::Full(
                        <String as ::rkyv::Serialize<__S>>::serialize(_0, serializer)?,
                    )
                }
                PartialComplexEnum::Partial { meta, payload, .. } => {
                    PartialComplexEnumResolver::Partial {
                        meta: <String as ::rkyv::Serialize<
                            __S,
                        >>::serialize(meta, serializer)?,
                        payload: <Vec<
                            u8,
                        > as ::rkyv::Serialize<__S>>::serialize(payload, serializer)?,
                    }
                }
            },
        )
    }
}
#[automatically_derived]
impl<__D: ::rkyv::rancor::Fallible + ?Sized> ::rkyv::Deserialize<PartialComplexEnum, __D>
for ::rkyv::Archived<PartialComplexEnum>
where
    String: ::rkyv::Archive,
    <String as ::rkyv::Archive>::Archived: ::rkyv::Deserialize<String, __D>,
    String: ::rkyv::Archive,
    <String as ::rkyv::Archive>::Archived: ::rkyv::Deserialize<String, __D>,
    Vec<u8>: ::rkyv::Archive,
    <Vec<u8> as ::rkyv::Archive>::Archived: ::rkyv::Deserialize<Vec<u8>, __D>,
{
    fn deserialize(
        &self,
        deserializer: &mut __D,
    ) -> ::core::result::Result<
        PartialComplexEnum,
        <__D as ::rkyv::rancor::Fallible>::Error,
    > {
        let __this = self;
        ::core::result::Result::Ok(
            match __this {
                Self::Full(_0, ..) => {
                    PartialComplexEnum::Full(
                        <<String as ::rkyv::Archive>::Archived as ::rkyv::Deserialize<
                            String,
                            __D,
                        >>::deserialize(_0, deserializer)?,
                    )
                }
                Self::Partial { meta, payload, .. } => {
                    PartialComplexEnum::Partial {
                        meta: <<String as ::rkyv::Archive>::Archived as ::rkyv::Deserialize<
                            String,
                            __D,
                        >>::deserialize(meta, deserializer)?,
                        payload: <<Vec<
                            u8,
                        > as ::rkyv::Archive>::Archived as ::rkyv::Deserialize<
                            Vec<u8>,
                            __D,
                        >>::deserialize(payload, deserializer)?,
                    }
                }
            },
        )
    }
}
#[automatically_derived]
impl ::core::fmt::Debug for PartialComplexEnum {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match self {
            PartialComplexEnum::Full(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "Full", &__self_0)
            }
            PartialComplexEnum::Partial { meta: __self_0, payload: __self_1 } => {
                ::core::fmt::Formatter::debug_struct_field2_finish(
                    f,
                    "Partial",
                    "meta",
                    __self_0,
                    "payload",
                    &__self_1,
                )
            }
        }
    }
}
#[automatically_derived]
impl ::core::clone::Clone for PartialComplexEnum {
    #[inline]
    fn clone(&self) -> PartialComplexEnum {
        match self {
            PartialComplexEnum::Full(__self_0) => {
                PartialComplexEnum::Full(::core::clone::Clone::clone(__self_0))
            }
            PartialComplexEnum::Partial { meta: __self_0, payload: __self_1 } => {
                PartialComplexEnum::Partial {
                    meta: ::core::clone::Clone::clone(__self_0),
                    payload: ::core::clone::Clone::clone(__self_1),
                }
            }
        }
    }
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for PartialComplexEnum {}
#[automatically_derived]
impl ::core::cmp::PartialEq for PartialComplexEnum {
    #[inline]
    fn eq(&self, other: &PartialComplexEnum) -> bool {
        let __self_discr = ::core::intrinsics::discriminant_value(self);
        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
        __self_discr == __arg1_discr
            && match (self, other) {
                (
                    PartialComplexEnum::Full(__self_0),
                    PartialComplexEnum::Full(__arg1_0),
                ) => __self_0 == __arg1_0,
                (
                    PartialComplexEnum::Partial { meta: __self_0, payload: __self_1 },
                    PartialComplexEnum::Partial { meta: __arg1_0, payload: __arg1_1 },
                ) => __self_0 == __arg1_0 && __self_1 == __arg1_1,
                _ => unsafe { ::core::intrinsics::unreachable() }
            }
    }
}
#[automatically_derived]
impl ::core::cmp::Eq for PartialComplexEnum {
    #[inline]
    #[doc(hidden)]
    #[coverage(off)]
    fn assert_fields_are_eq(&self) {
        let _: ::core::cmp::AssertParamIsEq<String>;
        let _: ::core::cmp::AssertParamIsEq<Vec<u8>>;
    }
}
pub struct PartialComplexEnumFullChunk {
    pub index: usize,
    pub data: Vec<u8>,
}
#[automatically_derived]
impl ::core::fmt::Debug for PartialComplexEnumFullChunk {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::debug_struct_field2_finish(
            f,
            "PartialComplexEnumFullChunk",
            "index",
            &self.index,
            "data",
            &&self.data,
        )
    }
}
#[automatically_derived]
impl ::core::clone::Clone for PartialComplexEnumFullChunk {
    #[inline]
    fn clone(&self) -> PartialComplexEnumFullChunk {
        PartialComplexEnumFullChunk {
            index: ::core::clone::Clone::clone(&self.index),
            data: ::core::clone::Clone::clone(&self.data),
        }
    }
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for PartialComplexEnumFullChunk {}
#[automatically_derived]
impl ::core::cmp::PartialEq for PartialComplexEnumFullChunk {
    #[inline]
    fn eq(&self, other: &PartialComplexEnumFullChunk) -> bool {
        self.index == other.index && self.data == other.data
    }
}
#[automatically_derived]
impl ::core::cmp::Eq for PartialComplexEnumFullChunk {
    #[inline]
    #[doc(hidden)]
    #[coverage(off)]
    fn assert_fields_are_eq(&self) {
        let _: ::core::cmp::AssertParamIsEq<usize>;
        let _: ::core::cmp::AssertParamIsEq<Vec<u8>>;
    }
}
#[automatically_derived]
impl ::core::cmp::PartialOrd for PartialComplexEnumFullChunk {
    #[inline]
    fn partial_cmp(
        &self,
        other: &PartialComplexEnumFullChunk,
    ) -> ::core::option::Option<::core::cmp::Ordering> {
        match ::core::cmp::PartialOrd::partial_cmp(&self.index, &other.index) {
            ::core::option::Option::Some(::core::cmp::Ordering::Equal) => {
                ::core::cmp::PartialOrd::partial_cmp(&self.data, &other.data)
            }
            cmp => cmp,
        }
    }
}
#[automatically_derived]
impl ::core::cmp::Ord for PartialComplexEnumFullChunk {
    #[inline]
    fn cmp(&self, other: &PartialComplexEnumFullChunk) -> ::core::cmp::Ordering {
        match ::core::cmp::Ord::cmp(&self.index, &other.index) {
            ::core::cmp::Ordering::Equal => {
                ::core::cmp::Ord::cmp(&self.data, &other.data)
            }
            cmp => cmp,
        }
    }
}
#[automatically_derived]
///An archived [`PartialComplexEnumFullChunk`]
#[bytecheck(crate = ::rkyv::bytecheck)]
#[repr(C)]
pub struct ArchivedPartialComplexEnumFullChunk
where
    usize: ::rkyv::Archive,
    Vec<u8>: ::rkyv::Archive,
{
    ///The archived counterpart of [`PartialComplexEnumFullChunk::index`]
    pub index: <usize as ::rkyv::Archive>::Archived,
    ///The archived counterpart of [`PartialComplexEnumFullChunk::data`]
    pub data: <Vec<u8> as ::rkyv::Archive>::Archived,
}
#[automatically_derived]
unsafe impl<
    __C: ::rkyv::bytecheck::rancor::Fallible + ?::core::marker::Sized,
> ::rkyv::bytecheck::CheckBytes<__C> for ArchivedPartialComplexEnumFullChunk
where
    usize: ::rkyv::Archive,
    Vec<u8>: ::rkyv::Archive,
    <__C as ::rkyv::bytecheck::rancor::Fallible>::Error: ::rkyv::bytecheck::rancor::Trace,
    <usize as ::rkyv::Archive>::Archived: ::rkyv::bytecheck::CheckBytes<__C>,
    <Vec<u8> as ::rkyv::Archive>::Archived: ::rkyv::bytecheck::CheckBytes<__C>,
{
    unsafe fn check_bytes(
        value: *const Self,
        context: &mut __C,
    ) -> ::core::result::Result<
        (),
        <__C as ::rkyv::bytecheck::rancor::Fallible>::Error,
    > {
        <<usize as ::rkyv::Archive>::Archived as ::rkyv::bytecheck::CheckBytes<
            __C,
        >>::check_bytes(&raw const (*value).index, context)
            .map_err(|e| {
                <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Trace>::trace(
                    e,
                    ::rkyv::bytecheck::StructCheckContext {
                        struct_name: "ArchivedPartialComplexEnumFullChunk",
                        field_name: "index",
                    },
                )
            })?;
        <<Vec<
            u8,
        > as ::rkyv::Archive>::Archived as ::rkyv::bytecheck::CheckBytes<
            __C,
        >>::check_bytes(&raw const (*value).data, context)
            .map_err(|e| {
                <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Trace>::trace(
                    e,
                    ::rkyv::bytecheck::StructCheckContext {
                        struct_name: "ArchivedPartialComplexEnumFullChunk",
                        field_name: "data",
                    },
                )
            })?;
        ::core::result::Result::Ok(())
    }
}
#[automatically_derived]
///The resolver for an archived [`PartialComplexEnumFullChunk`]
pub struct PartialComplexEnumFullChunkResolver
where
    usize: ::rkyv::Archive,
    Vec<u8>: ::rkyv::Archive,
{
    index: <usize as ::rkyv::Archive>::Resolver,
    data: <Vec<u8> as ::rkyv::Archive>::Resolver,
}
impl ::rkyv::Archive for PartialComplexEnumFullChunk
where
    usize: ::rkyv::Archive,
    Vec<u8>: ::rkyv::Archive,
{
    type Archived = ArchivedPartialComplexEnumFullChunk;
    type Resolver = PartialComplexEnumFullChunkResolver;
    const COPY_OPTIMIZATION: ::rkyv::traits::CopyOptimization<Self> = unsafe {
        ::rkyv::traits::CopyOptimization::enable_if(
            0 + ::core::mem::size_of::<usize>() + ::core::mem::size_of::<Vec<u8>>()
                == ::core::mem::size_of::<PartialComplexEnumFullChunk>()
                && <usize as ::rkyv::Archive>::COPY_OPTIMIZATION.is_enabled()
                && const { builtin # offset_of(PartialComplexEnumFullChunk, index) }
                    == const {
                        builtin # offset_of(ArchivedPartialComplexEnumFullChunk, index)
                    } && <Vec<u8> as ::rkyv::Archive>::COPY_OPTIMIZATION.is_enabled()
                && const { builtin # offset_of(PartialComplexEnumFullChunk, data) }
                    == const {
                        builtin # offset_of(ArchivedPartialComplexEnumFullChunk, data)
                    },
        )
    };
    #[allow(clippy::unit_arg)]
    fn resolve(&self, resolver: Self::Resolver, out: ::rkyv::Place<Self::Archived>) {
        let field_ptr = unsafe { &raw mut (*out.ptr()).index };
        let field_out = unsafe { ::rkyv::Place::from_field_unchecked(out, field_ptr) };
        <usize as ::rkyv::Archive>::resolve(&self.index, resolver.index, field_out);
        let field_ptr = unsafe { &raw mut (*out.ptr()).data };
        let field_out = unsafe { ::rkyv::Place::from_field_unchecked(out, field_ptr) };
        <Vec<u8> as ::rkyv::Archive>::resolve(&self.data, resolver.data, field_out);
    }
}
unsafe impl ::rkyv::traits::Portable for ArchivedPartialComplexEnumFullChunk
where
    usize: ::rkyv::Archive,
    Vec<u8>: ::rkyv::Archive,
    <usize as ::rkyv::Archive>::Archived: ::rkyv::traits::Portable,
    <Vec<u8> as ::rkyv::Archive>::Archived: ::rkyv::traits::Portable,
{}
#[automatically_derived]
impl<__S: ::rkyv::rancor::Fallible + ?Sized> ::rkyv::Serialize<__S>
for PartialComplexEnumFullChunk
where
    usize: ::rkyv::Serialize<__S>,
    Vec<u8>: ::rkyv::Serialize<__S>,
{
    fn serialize(
        &self,
        serializer: &mut __S,
    ) -> ::core::result::Result<
        <Self as ::rkyv::Archive>::Resolver,
        <__S as ::rkyv::rancor::Fallible>::Error,
    > {
        let __this = self;
        ::core::result::Result::Ok(PartialComplexEnumFullChunkResolver {
            index: <usize as ::rkyv::Serialize<
                __S,
            >>::serialize(&__this.index, serializer)?,
            data: <Vec<
                u8,
            > as ::rkyv::Serialize<__S>>::serialize(&__this.data, serializer)?,
        })
    }
}
#[automatically_derived]
impl<
    __D: ::rkyv::rancor::Fallible + ?Sized,
> ::rkyv::Deserialize<PartialComplexEnumFullChunk, __D>
for ::rkyv::Archived<PartialComplexEnumFullChunk>
where
    usize: ::rkyv::Archive,
    <usize as ::rkyv::Archive>::Archived: ::rkyv::Deserialize<usize, __D>,
    Vec<u8>: ::rkyv::Archive,
    <Vec<u8> as ::rkyv::Archive>::Archived: ::rkyv::Deserialize<Vec<u8>, __D>,
{
    fn deserialize(
        &self,
        deserializer: &mut __D,
    ) -> ::core::result::Result<
        PartialComplexEnumFullChunk,
        <__D as ::rkyv::rancor::Fallible>::Error,
    > {
        let __this = self;
        ::core::result::Result::Ok(PartialComplexEnumFullChunk {
            index: <<usize as ::rkyv::Archive>::Archived as ::rkyv::Deserialize<
                usize,
                __D,
            >>::deserialize(&__this.index, deserializer)?,
            data: <<Vec<
                u8,
            > as ::rkyv::Archive>::Archived as ::rkyv::Deserialize<
                Vec<u8>,
                __D,
            >>::deserialize(&__this.data, deserializer)?,
        })
    }
}
impl ::rewrite::traits::structural::blob::BlobItemChunk for PartialComplexEnumFullChunk {
    type Index = usize;
    fn get_index(&self) -> &Self::Index {
        &self.index
    }
}
pub struct PartialComplexEnumPartialMetaChunk {
    pub index: usize,
    pub data: Vec<u8>,
}
#[automatically_derived]
impl ::core::fmt::Debug for PartialComplexEnumPartialMetaChunk {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::debug_struct_field2_finish(
            f,
            "PartialComplexEnumPartialMetaChunk",
            "index",
            &self.index,
            "data",
            &&self.data,
        )
    }
}
#[automatically_derived]
impl ::core::clone::Clone for PartialComplexEnumPartialMetaChunk {
    #[inline]
    fn clone(&self) -> PartialComplexEnumPartialMetaChunk {
        PartialComplexEnumPartialMetaChunk {
            index: ::core::clone::Clone::clone(&self.index),
            data: ::core::clone::Clone::clone(&self.data),
        }
    }
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for PartialComplexEnumPartialMetaChunk {}
#[automatically_derived]
impl ::core::cmp::PartialEq for PartialComplexEnumPartialMetaChunk {
    #[inline]
    fn eq(&self, other: &PartialComplexEnumPartialMetaChunk) -> bool {
        self.index == other.index && self.data == other.data
    }
}
#[automatically_derived]
impl ::core::cmp::Eq for PartialComplexEnumPartialMetaChunk {
    #[inline]
    #[doc(hidden)]
    #[coverage(off)]
    fn assert_fields_are_eq(&self) {
        let _: ::core::cmp::AssertParamIsEq<usize>;
        let _: ::core::cmp::AssertParamIsEq<Vec<u8>>;
    }
}
#[automatically_derived]
impl ::core::cmp::PartialOrd for PartialComplexEnumPartialMetaChunk {
    #[inline]
    fn partial_cmp(
        &self,
        other: &PartialComplexEnumPartialMetaChunk,
    ) -> ::core::option::Option<::core::cmp::Ordering> {
        match ::core::cmp::PartialOrd::partial_cmp(&self.index, &other.index) {
            ::core::option::Option::Some(::core::cmp::Ordering::Equal) => {
                ::core::cmp::PartialOrd::partial_cmp(&self.data, &other.data)
            }
            cmp => cmp,
        }
    }
}
#[automatically_derived]
impl ::core::cmp::Ord for PartialComplexEnumPartialMetaChunk {
    #[inline]
    fn cmp(&self, other: &PartialComplexEnumPartialMetaChunk) -> ::core::cmp::Ordering {
        match ::core::cmp::Ord::cmp(&self.index, &other.index) {
            ::core::cmp::Ordering::Equal => {
                ::core::cmp::Ord::cmp(&self.data, &other.data)
            }
            cmp => cmp,
        }
    }
}
#[automatically_derived]
///An archived [`PartialComplexEnumPartialMetaChunk`]
#[bytecheck(crate = ::rkyv::bytecheck)]
#[repr(C)]
pub struct ArchivedPartialComplexEnumPartialMetaChunk
where
    usize: ::rkyv::Archive,
    Vec<u8>: ::rkyv::Archive,
{
    ///The archived counterpart of [`PartialComplexEnumPartialMetaChunk::index`]
    pub index: <usize as ::rkyv::Archive>::Archived,
    ///The archived counterpart of [`PartialComplexEnumPartialMetaChunk::data`]
    pub data: <Vec<u8> as ::rkyv::Archive>::Archived,
}
#[automatically_derived]
unsafe impl<
    __C: ::rkyv::bytecheck::rancor::Fallible + ?::core::marker::Sized,
> ::rkyv::bytecheck::CheckBytes<__C> for ArchivedPartialComplexEnumPartialMetaChunk
where
    usize: ::rkyv::Archive,
    Vec<u8>: ::rkyv::Archive,
    <__C as ::rkyv::bytecheck::rancor::Fallible>::Error: ::rkyv::bytecheck::rancor::Trace,
    <usize as ::rkyv::Archive>::Archived: ::rkyv::bytecheck::CheckBytes<__C>,
    <Vec<u8> as ::rkyv::Archive>::Archived: ::rkyv::bytecheck::CheckBytes<__C>,
{
    unsafe fn check_bytes(
        value: *const Self,
        context: &mut __C,
    ) -> ::core::result::Result<
        (),
        <__C as ::rkyv::bytecheck::rancor::Fallible>::Error,
    > {
        <<usize as ::rkyv::Archive>::Archived as ::rkyv::bytecheck::CheckBytes<
            __C,
        >>::check_bytes(&raw const (*value).index, context)
            .map_err(|e| {
                <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Trace>::trace(
                    e,
                    ::rkyv::bytecheck::StructCheckContext {
                        struct_name: "ArchivedPartialComplexEnumPartialMetaChunk",
                        field_name: "index",
                    },
                )
            })?;
        <<Vec<
            u8,
        > as ::rkyv::Archive>::Archived as ::rkyv::bytecheck::CheckBytes<
            __C,
        >>::check_bytes(&raw const (*value).data, context)
            .map_err(|e| {
                <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Trace>::trace(
                    e,
                    ::rkyv::bytecheck::StructCheckContext {
                        struct_name: "ArchivedPartialComplexEnumPartialMetaChunk",
                        field_name: "data",
                    },
                )
            })?;
        ::core::result::Result::Ok(())
    }
}
#[automatically_derived]
///The resolver for an archived [`PartialComplexEnumPartialMetaChunk`]
pub struct PartialComplexEnumPartialMetaChunkResolver
where
    usize: ::rkyv::Archive,
    Vec<u8>: ::rkyv::Archive,
{
    index: <usize as ::rkyv::Archive>::Resolver,
    data: <Vec<u8> as ::rkyv::Archive>::Resolver,
}
impl ::rkyv::Archive for PartialComplexEnumPartialMetaChunk
where
    usize: ::rkyv::Archive,
    Vec<u8>: ::rkyv::Archive,
{
    type Archived = ArchivedPartialComplexEnumPartialMetaChunk;
    type Resolver = PartialComplexEnumPartialMetaChunkResolver;
    const COPY_OPTIMIZATION: ::rkyv::traits::CopyOptimization<Self> = unsafe {
        ::rkyv::traits::CopyOptimization::enable_if(
            0 + ::core::mem::size_of::<usize>() + ::core::mem::size_of::<Vec<u8>>()
                == ::core::mem::size_of::<PartialComplexEnumPartialMetaChunk>()
                && <usize as ::rkyv::Archive>::COPY_OPTIMIZATION.is_enabled()
                && const {
                    builtin # offset_of(PartialComplexEnumPartialMetaChunk, index)
                }
                    == const {
                        builtin # offset_of(
                            ArchivedPartialComplexEnumPartialMetaChunk, index
                        )
                    } && <Vec<u8> as ::rkyv::Archive>::COPY_OPTIMIZATION.is_enabled()
                && const {
                    builtin # offset_of(PartialComplexEnumPartialMetaChunk, data)
                }
                    == const {
                        builtin # offset_of(
                            ArchivedPartialComplexEnumPartialMetaChunk, data
                        )
                    },
        )
    };
    #[allow(clippy::unit_arg)]
    fn resolve(&self, resolver: Self::Resolver, out: ::rkyv::Place<Self::Archived>) {
        let field_ptr = unsafe { &raw mut (*out.ptr()).index };
        let field_out = unsafe { ::rkyv::Place::from_field_unchecked(out, field_ptr) };
        <usize as ::rkyv::Archive>::resolve(&self.index, resolver.index, field_out);
        let field_ptr = unsafe { &raw mut (*out.ptr()).data };
        let field_out = unsafe { ::rkyv::Place::from_field_unchecked(out, field_ptr) };
        <Vec<u8> as ::rkyv::Archive>::resolve(&self.data, resolver.data, field_out);
    }
}
unsafe impl ::rkyv::traits::Portable for ArchivedPartialComplexEnumPartialMetaChunk
where
    usize: ::rkyv::Archive,
    Vec<u8>: ::rkyv::Archive,
    <usize as ::rkyv::Archive>::Archived: ::rkyv::traits::Portable,
    <Vec<u8> as ::rkyv::Archive>::Archived: ::rkyv::traits::Portable,
{}
#[automatically_derived]
impl<__S: ::rkyv::rancor::Fallible + ?Sized> ::rkyv::Serialize<__S>
for PartialComplexEnumPartialMetaChunk
where
    usize: ::rkyv::Serialize<__S>,
    Vec<u8>: ::rkyv::Serialize<__S>,
{
    fn serialize(
        &self,
        serializer: &mut __S,
    ) -> ::core::result::Result<
        <Self as ::rkyv::Archive>::Resolver,
        <__S as ::rkyv::rancor::Fallible>::Error,
    > {
        let __this = self;
        ::core::result::Result::Ok(PartialComplexEnumPartialMetaChunkResolver {
            index: <usize as ::rkyv::Serialize<
                __S,
            >>::serialize(&__this.index, serializer)?,
            data: <Vec<
                u8,
            > as ::rkyv::Serialize<__S>>::serialize(&__this.data, serializer)?,
        })
    }
}
#[automatically_derived]
impl<
    __D: ::rkyv::rancor::Fallible + ?Sized,
> ::rkyv::Deserialize<PartialComplexEnumPartialMetaChunk, __D>
for ::rkyv::Archived<PartialComplexEnumPartialMetaChunk>
where
    usize: ::rkyv::Archive,
    <usize as ::rkyv::Archive>::Archived: ::rkyv::Deserialize<usize, __D>,
    Vec<u8>: ::rkyv::Archive,
    <Vec<u8> as ::rkyv::Archive>::Archived: ::rkyv::Deserialize<Vec<u8>, __D>,
{
    fn deserialize(
        &self,
        deserializer: &mut __D,
    ) -> ::core::result::Result<
        PartialComplexEnumPartialMetaChunk,
        <__D as ::rkyv::rancor::Fallible>::Error,
    > {
        let __this = self;
        ::core::result::Result::Ok(PartialComplexEnumPartialMetaChunk {
            index: <<usize as ::rkyv::Archive>::Archived as ::rkyv::Deserialize<
                usize,
                __D,
            >>::deserialize(&__this.index, deserializer)?,
            data: <<Vec<
                u8,
            > as ::rkyv::Archive>::Archived as ::rkyv::Deserialize<
                Vec<u8>,
                __D,
            >>::deserialize(&__this.data, deserializer)?,
        })
    }
}
impl ::rewrite::traits::structural::blob::BlobItemChunk
for PartialComplexEnumPartialMetaChunk {
    type Index = usize;
    fn get_index(&self) -> &Self::Index {
        &self.index
    }
}
pub struct PartialComplexEnumPartialPayloadChunk {
    pub index: usize,
    pub data: Vec<u8>,
}
#[automatically_derived]
impl ::core::fmt::Debug for PartialComplexEnumPartialPayloadChunk {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::debug_struct_field2_finish(
            f,
            "PartialComplexEnumPartialPayloadChunk",
            "index",
            &self.index,
            "data",
            &&self.data,
        )
    }
}
#[automatically_derived]
impl ::core::clone::Clone for PartialComplexEnumPartialPayloadChunk {
    #[inline]
    fn clone(&self) -> PartialComplexEnumPartialPayloadChunk {
        PartialComplexEnumPartialPayloadChunk {
            index: ::core::clone::Clone::clone(&self.index),
            data: ::core::clone::Clone::clone(&self.data),
        }
    }
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for PartialComplexEnumPartialPayloadChunk {}
#[automatically_derived]
impl ::core::cmp::PartialEq for PartialComplexEnumPartialPayloadChunk {
    #[inline]
    fn eq(&self, other: &PartialComplexEnumPartialPayloadChunk) -> bool {
        self.index == other.index && self.data == other.data
    }
}
#[automatically_derived]
impl ::core::cmp::Eq for PartialComplexEnumPartialPayloadChunk {
    #[inline]
    #[doc(hidden)]
    #[coverage(off)]
    fn assert_fields_are_eq(&self) {
        let _: ::core::cmp::AssertParamIsEq<usize>;
        let _: ::core::cmp::AssertParamIsEq<Vec<u8>>;
    }
}
#[automatically_derived]
impl ::core::cmp::PartialOrd for PartialComplexEnumPartialPayloadChunk {
    #[inline]
    fn partial_cmp(
        &self,
        other: &PartialComplexEnumPartialPayloadChunk,
    ) -> ::core::option::Option<::core::cmp::Ordering> {
        match ::core::cmp::PartialOrd::partial_cmp(&self.index, &other.index) {
            ::core::option::Option::Some(::core::cmp::Ordering::Equal) => {
                ::core::cmp::PartialOrd::partial_cmp(&self.data, &other.data)
            }
            cmp => cmp,
        }
    }
}
#[automatically_derived]
impl ::core::cmp::Ord for PartialComplexEnumPartialPayloadChunk {
    #[inline]
    fn cmp(
        &self,
        other: &PartialComplexEnumPartialPayloadChunk,
    ) -> ::core::cmp::Ordering {
        match ::core::cmp::Ord::cmp(&self.index, &other.index) {
            ::core::cmp::Ordering::Equal => {
                ::core::cmp::Ord::cmp(&self.data, &other.data)
            }
            cmp => cmp,
        }
    }
}
#[automatically_derived]
///An archived [`PartialComplexEnumPartialPayloadChunk`]
#[bytecheck(crate = ::rkyv::bytecheck)]
#[repr(C)]
pub struct ArchivedPartialComplexEnumPartialPayloadChunk
where
    usize: ::rkyv::Archive,
    Vec<u8>: ::rkyv::Archive,
{
    ///The archived counterpart of [`PartialComplexEnumPartialPayloadChunk::index`]
    pub index: <usize as ::rkyv::Archive>::Archived,
    ///The archived counterpart of [`PartialComplexEnumPartialPayloadChunk::data`]
    pub data: <Vec<u8> as ::rkyv::Archive>::Archived,
}
#[automatically_derived]
unsafe impl<
    __C: ::rkyv::bytecheck::rancor::Fallible + ?::core::marker::Sized,
> ::rkyv::bytecheck::CheckBytes<__C> for ArchivedPartialComplexEnumPartialPayloadChunk
where
    usize: ::rkyv::Archive,
    Vec<u8>: ::rkyv::Archive,
    <__C as ::rkyv::bytecheck::rancor::Fallible>::Error: ::rkyv::bytecheck::rancor::Trace,
    <usize as ::rkyv::Archive>::Archived: ::rkyv::bytecheck::CheckBytes<__C>,
    <Vec<u8> as ::rkyv::Archive>::Archived: ::rkyv::bytecheck::CheckBytes<__C>,
{
    unsafe fn check_bytes(
        value: *const Self,
        context: &mut __C,
    ) -> ::core::result::Result<
        (),
        <__C as ::rkyv::bytecheck::rancor::Fallible>::Error,
    > {
        <<usize as ::rkyv::Archive>::Archived as ::rkyv::bytecheck::CheckBytes<
            __C,
        >>::check_bytes(&raw const (*value).index, context)
            .map_err(|e| {
                <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Trace>::trace(
                    e,
                    ::rkyv::bytecheck::StructCheckContext {
                        struct_name: "ArchivedPartialComplexEnumPartialPayloadChunk",
                        field_name: "index",
                    },
                )
            })?;
        <<Vec<
            u8,
        > as ::rkyv::Archive>::Archived as ::rkyv::bytecheck::CheckBytes<
            __C,
        >>::check_bytes(&raw const (*value).data, context)
            .map_err(|e| {
                <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Trace>::trace(
                    e,
                    ::rkyv::bytecheck::StructCheckContext {
                        struct_name: "ArchivedPartialComplexEnumPartialPayloadChunk",
                        field_name: "data",
                    },
                )
            })?;
        ::core::result::Result::Ok(())
    }
}
#[automatically_derived]
///The resolver for an archived [`PartialComplexEnumPartialPayloadChunk`]
pub struct PartialComplexEnumPartialPayloadChunkResolver
where
    usize: ::rkyv::Archive,
    Vec<u8>: ::rkyv::Archive,
{
    index: <usize as ::rkyv::Archive>::Resolver,
    data: <Vec<u8> as ::rkyv::Archive>::Resolver,
}
impl ::rkyv::Archive for PartialComplexEnumPartialPayloadChunk
where
    usize: ::rkyv::Archive,
    Vec<u8>: ::rkyv::Archive,
{
    type Archived = ArchivedPartialComplexEnumPartialPayloadChunk;
    type Resolver = PartialComplexEnumPartialPayloadChunkResolver;
    const COPY_OPTIMIZATION: ::rkyv::traits::CopyOptimization<Self> = unsafe {
        ::rkyv::traits::CopyOptimization::enable_if(
            0 + ::core::mem::size_of::<usize>() + ::core::mem::size_of::<Vec<u8>>()
                == ::core::mem::size_of::<PartialComplexEnumPartialPayloadChunk>()
                && <usize as ::rkyv::Archive>::COPY_OPTIMIZATION.is_enabled()
                && const {
                    builtin # offset_of(PartialComplexEnumPartialPayloadChunk, index)
                }
                    == const {
                        builtin # offset_of(
                            ArchivedPartialComplexEnumPartialPayloadChunk, index
                        )
                    } && <Vec<u8> as ::rkyv::Archive>::COPY_OPTIMIZATION.is_enabled()
                && const {
                    builtin # offset_of(PartialComplexEnumPartialPayloadChunk, data)
                }
                    == const {
                        builtin # offset_of(
                            ArchivedPartialComplexEnumPartialPayloadChunk, data
                        )
                    },
        )
    };
    #[allow(clippy::unit_arg)]
    fn resolve(&self, resolver: Self::Resolver, out: ::rkyv::Place<Self::Archived>) {
        let field_ptr = unsafe { &raw mut (*out.ptr()).index };
        let field_out = unsafe { ::rkyv::Place::from_field_unchecked(out, field_ptr) };
        <usize as ::rkyv::Archive>::resolve(&self.index, resolver.index, field_out);
        let field_ptr = unsafe { &raw mut (*out.ptr()).data };
        let field_out = unsafe { ::rkyv::Place::from_field_unchecked(out, field_ptr) };
        <Vec<u8> as ::rkyv::Archive>::resolve(&self.data, resolver.data, field_out);
    }
}
unsafe impl ::rkyv::traits::Portable for ArchivedPartialComplexEnumPartialPayloadChunk
where
    usize: ::rkyv::Archive,
    Vec<u8>: ::rkyv::Archive,
    <usize as ::rkyv::Archive>::Archived: ::rkyv::traits::Portable,
    <Vec<u8> as ::rkyv::Archive>::Archived: ::rkyv::traits::Portable,
{}
#[automatically_derived]
impl<__S: ::rkyv::rancor::Fallible + ?Sized> ::rkyv::Serialize<__S>
for PartialComplexEnumPartialPayloadChunk
where
    usize: ::rkyv::Serialize<__S>,
    Vec<u8>: ::rkyv::Serialize<__S>,
{
    fn serialize(
        &self,
        serializer: &mut __S,
    ) -> ::core::result::Result<
        <Self as ::rkyv::Archive>::Resolver,
        <__S as ::rkyv::rancor::Fallible>::Error,
    > {
        let __this = self;
        ::core::result::Result::Ok(PartialComplexEnumPartialPayloadChunkResolver {
            index: <usize as ::rkyv::Serialize<
                __S,
            >>::serialize(&__this.index, serializer)?,
            data: <Vec<
                u8,
            > as ::rkyv::Serialize<__S>>::serialize(&__this.data, serializer)?,
        })
    }
}
#[automatically_derived]
impl<
    __D: ::rkyv::rancor::Fallible + ?Sized,
> ::rkyv::Deserialize<PartialComplexEnumPartialPayloadChunk, __D>
for ::rkyv::Archived<PartialComplexEnumPartialPayloadChunk>
where
    usize: ::rkyv::Archive,
    <usize as ::rkyv::Archive>::Archived: ::rkyv::Deserialize<usize, __D>,
    Vec<u8>: ::rkyv::Archive,
    <Vec<u8> as ::rkyv::Archive>::Archived: ::rkyv::Deserialize<Vec<u8>, __D>,
{
    fn deserialize(
        &self,
        deserializer: &mut __D,
    ) -> ::core::result::Result<
        PartialComplexEnumPartialPayloadChunk,
        <__D as ::rkyv::rancor::Fallible>::Error,
    > {
        let __this = self;
        ::core::result::Result::Ok(PartialComplexEnumPartialPayloadChunk {
            index: <<usize as ::rkyv::Archive>::Archived as ::rkyv::Deserialize<
                usize,
                __D,
            >>::deserialize(&__this.index, deserializer)?,
            data: <<Vec<
                u8,
            > as ::rkyv::Archive>::Archived as ::rkyv::Deserialize<
                Vec<u8>,
                __D,
            >>::deserialize(&__this.data, deserializer)?,
        })
    }
}
impl ::rewrite::traits::structural::blob::BlobItemChunk
for PartialComplexEnumPartialPayloadChunk {
    type Index = usize;
    fn get_index(&self) -> &Self::Index {
        &self.index
    }
}
pub enum PartialComplexEnumPartialChunk {
    Meta(PartialComplexEnumPartialMetaChunk),
    Payload(PartialComplexEnumPartialPayloadChunk),
    Missing,
}
#[automatically_derived]
impl ::core::fmt::Debug for PartialComplexEnumPartialChunk {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match self {
            PartialComplexEnumPartialChunk::Meta(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "Meta", &__self_0)
            }
            PartialComplexEnumPartialChunk::Payload(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(
                    f,
                    "Payload",
                    &__self_0,
                )
            }
            PartialComplexEnumPartialChunk::Missing => {
                ::core::fmt::Formatter::write_str(f, "Missing")
            }
        }
    }
}
#[automatically_derived]
impl ::core::clone::Clone for PartialComplexEnumPartialChunk {
    #[inline]
    fn clone(&self) -> PartialComplexEnumPartialChunk {
        match self {
            PartialComplexEnumPartialChunk::Meta(__self_0) => {
                PartialComplexEnumPartialChunk::Meta(
                    ::core::clone::Clone::clone(__self_0),
                )
            }
            PartialComplexEnumPartialChunk::Payload(__self_0) => {
                PartialComplexEnumPartialChunk::Payload(
                    ::core::clone::Clone::clone(__self_0),
                )
            }
            PartialComplexEnumPartialChunk::Missing => {
                PartialComplexEnumPartialChunk::Missing
            }
        }
    }
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for PartialComplexEnumPartialChunk {}
#[automatically_derived]
impl ::core::cmp::PartialEq for PartialComplexEnumPartialChunk {
    #[inline]
    fn eq(&self, other: &PartialComplexEnumPartialChunk) -> bool {
        let __self_discr = ::core::intrinsics::discriminant_value(self);
        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
        __self_discr == __arg1_discr
            && match (self, other) {
                (
                    PartialComplexEnumPartialChunk::Meta(__self_0),
                    PartialComplexEnumPartialChunk::Meta(__arg1_0),
                ) => __self_0 == __arg1_0,
                (
                    PartialComplexEnumPartialChunk::Payload(__self_0),
                    PartialComplexEnumPartialChunk::Payload(__arg1_0),
                ) => __self_0 == __arg1_0,
                _ => true,
            }
    }
}
#[automatically_derived]
impl ::core::cmp::Eq for PartialComplexEnumPartialChunk {
    #[inline]
    #[doc(hidden)]
    #[coverage(off)]
    fn assert_fields_are_eq(&self) {
        let _: ::core::cmp::AssertParamIsEq<PartialComplexEnumPartialMetaChunk>;
        let _: ::core::cmp::AssertParamIsEq<PartialComplexEnumPartialPayloadChunk>;
    }
}
#[automatically_derived]
impl ::core::cmp::PartialOrd for PartialComplexEnumPartialChunk {
    #[inline]
    fn partial_cmp(
        &self,
        other: &PartialComplexEnumPartialChunk,
    ) -> ::core::option::Option<::core::cmp::Ordering> {
        let __self_discr = ::core::intrinsics::discriminant_value(self);
        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
        match (self, other) {
            (
                PartialComplexEnumPartialChunk::Meta(__self_0),
                PartialComplexEnumPartialChunk::Meta(__arg1_0),
            ) => ::core::cmp::PartialOrd::partial_cmp(__self_0, __arg1_0),
            (
                PartialComplexEnumPartialChunk::Payload(__self_0),
                PartialComplexEnumPartialChunk::Payload(__arg1_0),
            ) => ::core::cmp::PartialOrd::partial_cmp(__self_0, __arg1_0),
            _ => ::core::cmp::PartialOrd::partial_cmp(&__self_discr, &__arg1_discr),
        }
    }
}
#[automatically_derived]
impl ::core::cmp::Ord for PartialComplexEnumPartialChunk {
    #[inline]
    fn cmp(&self, other: &PartialComplexEnumPartialChunk) -> ::core::cmp::Ordering {
        let __self_discr = ::core::intrinsics::discriminant_value(self);
        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
        match ::core::cmp::Ord::cmp(&__self_discr, &__arg1_discr) {
            ::core::cmp::Ordering::Equal => {
                match (self, other) {
                    (
                        PartialComplexEnumPartialChunk::Meta(__self_0),
                        PartialComplexEnumPartialChunk::Meta(__arg1_0),
                    ) => ::core::cmp::Ord::cmp(__self_0, __arg1_0),
                    (
                        PartialComplexEnumPartialChunk::Payload(__self_0),
                        PartialComplexEnumPartialChunk::Payload(__arg1_0),
                    ) => ::core::cmp::Ord::cmp(__self_0, __arg1_0),
                    _ => ::core::cmp::Ordering::Equal,
                }
            }
            cmp => cmp,
        }
    }
}
#[automatically_derived]
///An archived [`PartialComplexEnumPartialChunk`]
#[bytecheck(crate = ::rkyv::bytecheck)]
#[repr(u8)]
pub enum ArchivedPartialComplexEnumPartialChunk
where
    PartialComplexEnumPartialMetaChunk: ::rkyv::Archive,
    PartialComplexEnumPartialPayloadChunk: ::rkyv::Archive,
{
    ///The archived counterpart of [`PartialComplexEnumPartialChunk::Meta`]
    #[allow(dead_code)]
    Meta(
        ///The archived counterpart of [`PartialComplexEnumPartialChunk::Meta::0`]
        <PartialComplexEnumPartialMetaChunk as ::rkyv::Archive>::Archived,
    ),
    ///The archived counterpart of [`PartialComplexEnumPartialChunk::Payload`]
    #[allow(dead_code)]
    Payload(
        ///The archived counterpart of [`PartialComplexEnumPartialChunk::Payload::0`]
        <PartialComplexEnumPartialPayloadChunk as ::rkyv::Archive>::Archived,
    ),
    ///The archived counterpart of [`PartialComplexEnumPartialChunk::Missing`]
    #[allow(dead_code)]
    Missing,
}
const _: () = {
    #[repr(u8)]
    enum Tag {
        Meta,
        Payload,
        Missing,
    }
    struct Discriminant;
    #[automatically_derived]
    impl Discriminant {
        #[allow(non_upper_case_globals)]
        const Meta: u8 = Tag::Meta as u8;
        #[allow(non_upper_case_globals)]
        const Payload: u8 = Tag::Payload as u8;
        #[allow(non_upper_case_globals)]
        const Missing: u8 = Tag::Missing as u8;
    }
    #[repr(C)]
    struct VariantMeta(
        Tag,
        <PartialComplexEnumPartialMetaChunk as ::rkyv::Archive>::Archived,
        ::core::marker::PhantomData<ArchivedPartialComplexEnumPartialChunk>,
    )
    where
        PartialComplexEnumPartialMetaChunk: ::rkyv::Archive,
        PartialComplexEnumPartialPayloadChunk: ::rkyv::Archive;
    #[repr(C)]
    struct VariantPayload(
        Tag,
        <PartialComplexEnumPartialPayloadChunk as ::rkyv::Archive>::Archived,
        ::core::marker::PhantomData<ArchivedPartialComplexEnumPartialChunk>,
    )
    where
        PartialComplexEnumPartialMetaChunk: ::rkyv::Archive,
        PartialComplexEnumPartialPayloadChunk: ::rkyv::Archive;
    #[automatically_derived]
    unsafe impl<
        __C: ::rkyv::bytecheck::rancor::Fallible + ?::core::marker::Sized,
    > ::rkyv::bytecheck::CheckBytes<__C> for ArchivedPartialComplexEnumPartialChunk
    where
        PartialComplexEnumPartialMetaChunk: ::rkyv::Archive,
        PartialComplexEnumPartialPayloadChunk: ::rkyv::Archive,
        <__C as ::rkyv::bytecheck::rancor::Fallible>::Error: ::rkyv::bytecheck::rancor::Source,
        <PartialComplexEnumPartialMetaChunk as ::rkyv::Archive>::Archived: ::rkyv::bytecheck::CheckBytes<
            __C,
        >,
        <PartialComplexEnumPartialPayloadChunk as ::rkyv::Archive>::Archived: ::rkyv::bytecheck::CheckBytes<
            __C,
        >,
    {
        unsafe fn check_bytes(
            value: *const Self,
            context: &mut __C,
        ) -> ::core::result::Result<
            (),
            <__C as ::rkyv::bytecheck::rancor::Fallible>::Error,
        > {
            let tag = *value.cast::<u8>();
            match tag {
                Discriminant::Meta => {
                    let value = value.cast::<VariantMeta>();
                    <<PartialComplexEnumPartialMetaChunk as ::rkyv::Archive>::Archived as ::rkyv::bytecheck::CheckBytes<
                        __C,
                    >>::check_bytes(&raw const (*value).1, context)
                        .map_err(|e| {
                            <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Trace>::trace(
                                e,
                                ::rkyv::bytecheck::UnnamedEnumVariantCheckContext {
                                    enum_name: "ArchivedPartialComplexEnumPartialChunk",
                                    variant_name: "Meta",
                                    field_index: 1,
                                },
                            )
                        })?;
                }
                Discriminant::Payload => {
                    let value = value.cast::<VariantPayload>();
                    <<PartialComplexEnumPartialPayloadChunk as ::rkyv::Archive>::Archived as ::rkyv::bytecheck::CheckBytes<
                        __C,
                    >>::check_bytes(&raw const (*value).1, context)
                        .map_err(|e| {
                            <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Trace>::trace(
                                e,
                                ::rkyv::bytecheck::UnnamedEnumVariantCheckContext {
                                    enum_name: "ArchivedPartialComplexEnumPartialChunk",
                                    variant_name: "Payload",
                                    field_index: 1,
                                },
                            )
                        })?;
                }
                Discriminant::Missing => {}
                _ => {
                    return ::core::result::Result::Err(
                        <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Source>::new(::rkyv::bytecheck::InvalidEnumDiscriminantError {
                            enum_name: "ArchivedPartialComplexEnumPartialChunk",
                            invalid_discriminant: tag,
                        }),
                    );
                }
            }
            ::core::result::Result::Ok(())
        }
    }
};
#[automatically_derived]
///The resolver for an archived [`PartialComplexEnumPartialChunk`]
pub enum PartialComplexEnumPartialChunkResolver
where
    PartialComplexEnumPartialMetaChunk: ::rkyv::Archive,
    PartialComplexEnumPartialPayloadChunk: ::rkyv::Archive,
{
    ///The resolver for [`PartialComplexEnumPartialChunk::Meta`]
    #[allow(dead_code)]
    Meta(<PartialComplexEnumPartialMetaChunk as ::rkyv::Archive>::Resolver),
    ///The resolver for [`PartialComplexEnumPartialChunk::Payload`]
    #[allow(dead_code)]
    Payload(<PartialComplexEnumPartialPayloadChunk as ::rkyv::Archive>::Resolver),
    ///The resolver for [`PartialComplexEnumPartialChunk::Missing`]
    #[allow(dead_code)]
    Missing,
}
const _: () = {
    #[repr(u8)]
    enum ArchivedTag {
        Meta,
        Payload,
        Missing,
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for ArchivedTag {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for ArchivedTag {
        #[inline]
        fn eq(&self, other: &ArchivedTag) -> bool {
            let __self_discr = ::core::intrinsics::discriminant_value(self);
            let __arg1_discr = ::core::intrinsics::discriminant_value(other);
            __self_discr == __arg1_discr
        }
    }
    #[automatically_derived]
    impl ::core::cmp::PartialOrd for ArchivedTag {
        #[inline]
        fn partial_cmp(
            &self,
            other: &ArchivedTag,
        ) -> ::core::option::Option<::core::cmp::Ordering> {
            let __self_discr = ::core::intrinsics::discriminant_value(self);
            let __arg1_discr = ::core::intrinsics::discriminant_value(other);
            ::core::cmp::PartialOrd::partial_cmp(&__self_discr, &__arg1_discr)
        }
    }
    #[repr(C)]
    struct ArchivedVariantMeta(
        ArchivedTag,
        <PartialComplexEnumPartialMetaChunk as ::rkyv::Archive>::Archived,
        ::core::marker::PhantomData<PartialComplexEnumPartialChunk>,
    )
    where
        PartialComplexEnumPartialMetaChunk: ::rkyv::Archive,
        PartialComplexEnumPartialPayloadChunk: ::rkyv::Archive;
    #[repr(C)]
    struct ArchivedVariantPayload(
        ArchivedTag,
        <PartialComplexEnumPartialPayloadChunk as ::rkyv::Archive>::Archived,
        ::core::marker::PhantomData<PartialComplexEnumPartialChunk>,
    )
    where
        PartialComplexEnumPartialMetaChunk: ::rkyv::Archive,
        PartialComplexEnumPartialPayloadChunk: ::rkyv::Archive;
    impl ::rkyv::Archive for PartialComplexEnumPartialChunk
    where
        PartialComplexEnumPartialMetaChunk: ::rkyv::Archive,
        PartialComplexEnumPartialPayloadChunk: ::rkyv::Archive,
    {
        type Archived = ArchivedPartialComplexEnumPartialChunk;
        type Resolver = PartialComplexEnumPartialChunkResolver;
        #[allow(clippy::unit_arg)]
        fn resolve(
            &self,
            resolver: <Self as ::rkyv::Archive>::Resolver,
            out: ::rkyv::Place<<Self as ::rkyv::Archive>::Archived>,
        ) {
            let __this = self;
            match resolver {
                PartialComplexEnumPartialChunkResolver::Meta(resolver_0) => {
                    match __this {
                        PartialComplexEnumPartialChunk::Meta(self_0, ..) => {
                            let out = unsafe {
                                out.cast_unchecked::<ArchivedVariantMeta>()
                            };
                            let tag_ptr = unsafe { &raw mut (*out.ptr()).0 };
                            unsafe {
                                tag_ptr.write(ArchivedTag::Meta);
                            }
                            let field_ptr = unsafe { &raw mut (*out.ptr()).1 };
                            let out_field = unsafe {
                                ::rkyv::Place::from_field_unchecked(out, field_ptr)
                            };
                            <PartialComplexEnumPartialMetaChunk as ::rkyv::Archive>::resolve(
                                self_0,
                                resolver_0,
                                out_field,
                            );
                        }
                        #[allow(unreachable_patterns)]
                        _ => unsafe { ::core::hint::unreachable_unchecked() }
                    }
                }
                PartialComplexEnumPartialChunkResolver::Payload(resolver_0) => {
                    match __this {
                        PartialComplexEnumPartialChunk::Payload(self_0, ..) => {
                            let out = unsafe {
                                out.cast_unchecked::<ArchivedVariantPayload>()
                            };
                            let tag_ptr = unsafe { &raw mut (*out.ptr()).0 };
                            unsafe {
                                tag_ptr.write(ArchivedTag::Payload);
                            }
                            let field_ptr = unsafe { &raw mut (*out.ptr()).1 };
                            let out_field = unsafe {
                                ::rkyv::Place::from_field_unchecked(out, field_ptr)
                            };
                            <PartialComplexEnumPartialPayloadChunk as ::rkyv::Archive>::resolve(
                                self_0,
                                resolver_0,
                                out_field,
                            );
                        }
                        #[allow(unreachable_patterns)]
                        _ => unsafe { ::core::hint::unreachable_unchecked() }
                    }
                }
                PartialComplexEnumPartialChunkResolver::Missing => {
                    let out = unsafe { out.cast_unchecked::<ArchivedTag>() };
                    unsafe {
                        out.write_unchecked(ArchivedTag::Missing);
                    }
                }
            }
        }
    }
};
unsafe impl ::rkyv::traits::Portable for ArchivedPartialComplexEnumPartialChunk
where
    PartialComplexEnumPartialMetaChunk: ::rkyv::Archive,
    PartialComplexEnumPartialPayloadChunk: ::rkyv::Archive,
    <PartialComplexEnumPartialMetaChunk as ::rkyv::Archive>::Archived: ::rkyv::traits::Portable,
    <PartialComplexEnumPartialPayloadChunk as ::rkyv::Archive>::Archived: ::rkyv::traits::Portable,
{}
#[automatically_derived]
impl<__S: ::rkyv::rancor::Fallible + ?Sized> ::rkyv::Serialize<__S>
for PartialComplexEnumPartialChunk
where
    PartialComplexEnumPartialMetaChunk: ::rkyv::Serialize<__S>,
    PartialComplexEnumPartialPayloadChunk: ::rkyv::Serialize<__S>,
{
    fn serialize(
        &self,
        serializer: &mut __S,
    ) -> ::core::result::Result<
        <Self as ::rkyv::Archive>::Resolver,
        <__S as ::rkyv::rancor::Fallible>::Error,
    > {
        let __this = self;
        ::core::result::Result::Ok(
            match __this {
                PartialComplexEnumPartialChunk::Meta(_0, ..) => {
                    PartialComplexEnumPartialChunkResolver::Meta(
                        <PartialComplexEnumPartialMetaChunk as ::rkyv::Serialize<
                            __S,
                        >>::serialize(_0, serializer)?,
                    )
                }
                PartialComplexEnumPartialChunk::Payload(_0, ..) => {
                    PartialComplexEnumPartialChunkResolver::Payload(
                        <PartialComplexEnumPartialPayloadChunk as ::rkyv::Serialize<
                            __S,
                        >>::serialize(_0, serializer)?,
                    )
                }
                PartialComplexEnumPartialChunk::Missing => {
                    PartialComplexEnumPartialChunkResolver::Missing
                }
            },
        )
    }
}
#[automatically_derived]
impl<
    __D: ::rkyv::rancor::Fallible + ?Sized,
> ::rkyv::Deserialize<PartialComplexEnumPartialChunk, __D>
for ::rkyv::Archived<PartialComplexEnumPartialChunk>
where
    PartialComplexEnumPartialMetaChunk: ::rkyv::Archive,
    <PartialComplexEnumPartialMetaChunk as ::rkyv::Archive>::Archived: ::rkyv::Deserialize<
        PartialComplexEnumPartialMetaChunk,
        __D,
    >,
    PartialComplexEnumPartialPayloadChunk: ::rkyv::Archive,
    <PartialComplexEnumPartialPayloadChunk as ::rkyv::Archive>::Archived: ::rkyv::Deserialize<
        PartialComplexEnumPartialPayloadChunk,
        __D,
    >,
{
    fn deserialize(
        &self,
        deserializer: &mut __D,
    ) -> ::core::result::Result<
        PartialComplexEnumPartialChunk,
        <__D as ::rkyv::rancor::Fallible>::Error,
    > {
        let __this = self;
        ::core::result::Result::Ok(
            match __this {
                Self::Meta(_0, ..) => {
                    PartialComplexEnumPartialChunk::Meta(
                        <<PartialComplexEnumPartialMetaChunk as ::rkyv::Archive>::Archived as ::rkyv::Deserialize<
                            PartialComplexEnumPartialMetaChunk,
                            __D,
                        >>::deserialize(_0, deserializer)?,
                    )
                }
                Self::Payload(_0, ..) => {
                    PartialComplexEnumPartialChunk::Payload(
                        <<PartialComplexEnumPartialPayloadChunk as ::rkyv::Archive>::Archived as ::rkyv::Deserialize<
                            PartialComplexEnumPartialPayloadChunk,
                            __D,
                        >>::deserialize(_0, deserializer)?,
                    )
                }
                Self::Missing => PartialComplexEnumPartialChunk::Missing,
            },
        )
    }
}
impl rewrite::traits::structural::blob::BlobItemChunk
for PartialComplexEnumPartialChunk {
    type Index = usize;
    fn get_index(&self) -> &Self::Index {
        match self {
            Self::Meta(inner) => inner.get_index(),
            Self::Payload(inner) => inner.get_index(),
            Self::Missing => {
                ::core::panicking::panic_fmt(
                    format_args!("Called get_index() on Missing chunk"),
                );
            }
        }
    }
}
pub enum PartialComplexEnumChunk {
    Full(PartialComplexEnumFullChunk),
    Partial(PartialComplexEnumPartialChunk),
    Missing,
}
#[automatically_derived]
impl ::core::fmt::Debug for PartialComplexEnumChunk {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match self {
            PartialComplexEnumChunk::Full(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "Full", &__self_0)
            }
            PartialComplexEnumChunk::Partial(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(
                    f,
                    "Partial",
                    &__self_0,
                )
            }
            PartialComplexEnumChunk::Missing => {
                ::core::fmt::Formatter::write_str(f, "Missing")
            }
        }
    }
}
#[automatically_derived]
impl ::core::clone::Clone for PartialComplexEnumChunk {
    #[inline]
    fn clone(&self) -> PartialComplexEnumChunk {
        match self {
            PartialComplexEnumChunk::Full(__self_0) => {
                PartialComplexEnumChunk::Full(::core::clone::Clone::clone(__self_0))
            }
            PartialComplexEnumChunk::Partial(__self_0) => {
                PartialComplexEnumChunk::Partial(::core::clone::Clone::clone(__self_0))
            }
            PartialComplexEnumChunk::Missing => PartialComplexEnumChunk::Missing,
        }
    }
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for PartialComplexEnumChunk {}
#[automatically_derived]
impl ::core::cmp::PartialEq for PartialComplexEnumChunk {
    #[inline]
    fn eq(&self, other: &PartialComplexEnumChunk) -> bool {
        let __self_discr = ::core::intrinsics::discriminant_value(self);
        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
        __self_discr == __arg1_discr
            && match (self, other) {
                (
                    PartialComplexEnumChunk::Full(__self_0),
                    PartialComplexEnumChunk::Full(__arg1_0),
                ) => __self_0 == __arg1_0,
                (
                    PartialComplexEnumChunk::Partial(__self_0),
                    PartialComplexEnumChunk::Partial(__arg1_0),
                ) => __self_0 == __arg1_0,
                _ => true,
            }
    }
}
#[automatically_derived]
impl ::core::cmp::Eq for PartialComplexEnumChunk {
    #[inline]
    #[doc(hidden)]
    #[coverage(off)]
    fn assert_fields_are_eq(&self) {
        let _: ::core::cmp::AssertParamIsEq<PartialComplexEnumFullChunk>;
        let _: ::core::cmp::AssertParamIsEq<PartialComplexEnumPartialChunk>;
    }
}
#[automatically_derived]
impl ::core::cmp::PartialOrd for PartialComplexEnumChunk {
    #[inline]
    fn partial_cmp(
        &self,
        other: &PartialComplexEnumChunk,
    ) -> ::core::option::Option<::core::cmp::Ordering> {
        let __self_discr = ::core::intrinsics::discriminant_value(self);
        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
        match (self, other) {
            (
                PartialComplexEnumChunk::Full(__self_0),
                PartialComplexEnumChunk::Full(__arg1_0),
            ) => ::core::cmp::PartialOrd::partial_cmp(__self_0, __arg1_0),
            (
                PartialComplexEnumChunk::Partial(__self_0),
                PartialComplexEnumChunk::Partial(__arg1_0),
            ) => ::core::cmp::PartialOrd::partial_cmp(__self_0, __arg1_0),
            _ => ::core::cmp::PartialOrd::partial_cmp(&__self_discr, &__arg1_discr),
        }
    }
}
#[automatically_derived]
impl ::core::cmp::Ord for PartialComplexEnumChunk {
    #[inline]
    fn cmp(&self, other: &PartialComplexEnumChunk) -> ::core::cmp::Ordering {
        let __self_discr = ::core::intrinsics::discriminant_value(self);
        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
        match ::core::cmp::Ord::cmp(&__self_discr, &__arg1_discr) {
            ::core::cmp::Ordering::Equal => {
                match (self, other) {
                    (
                        PartialComplexEnumChunk::Full(__self_0),
                        PartialComplexEnumChunk::Full(__arg1_0),
                    ) => ::core::cmp::Ord::cmp(__self_0, __arg1_0),
                    (
                        PartialComplexEnumChunk::Partial(__self_0),
                        PartialComplexEnumChunk::Partial(__arg1_0),
                    ) => ::core::cmp::Ord::cmp(__self_0, __arg1_0),
                    _ => ::core::cmp::Ordering::Equal,
                }
            }
            cmp => cmp,
        }
    }
}
#[automatically_derived]
///An archived [`PartialComplexEnumChunk`]
#[bytecheck(crate = ::rkyv::bytecheck)]
#[repr(u8)]
pub enum ArchivedPartialComplexEnumChunk
where
    PartialComplexEnumFullChunk: ::rkyv::Archive,
    PartialComplexEnumPartialChunk: ::rkyv::Archive,
{
    ///The archived counterpart of [`PartialComplexEnumChunk::Full`]
    #[allow(dead_code)]
    Full(
        ///The archived counterpart of [`PartialComplexEnumChunk::Full::0`]
        <PartialComplexEnumFullChunk as ::rkyv::Archive>::Archived,
    ),
    ///The archived counterpart of [`PartialComplexEnumChunk::Partial`]
    #[allow(dead_code)]
    Partial(
        ///The archived counterpart of [`PartialComplexEnumChunk::Partial::0`]
        <PartialComplexEnumPartialChunk as ::rkyv::Archive>::Archived,
    ),
    ///The archived counterpart of [`PartialComplexEnumChunk::Missing`]
    #[allow(dead_code)]
    Missing,
}
const _: () = {
    #[repr(u8)]
    enum Tag {
        Full,
        Partial,
        Missing,
    }
    struct Discriminant;
    #[automatically_derived]
    impl Discriminant {
        #[allow(non_upper_case_globals)]
        const Full: u8 = Tag::Full as u8;
        #[allow(non_upper_case_globals)]
        const Partial: u8 = Tag::Partial as u8;
        #[allow(non_upper_case_globals)]
        const Missing: u8 = Tag::Missing as u8;
    }
    #[repr(C)]
    struct VariantFull(
        Tag,
        <PartialComplexEnumFullChunk as ::rkyv::Archive>::Archived,
        ::core::marker::PhantomData<ArchivedPartialComplexEnumChunk>,
    )
    where
        PartialComplexEnumFullChunk: ::rkyv::Archive,
        PartialComplexEnumPartialChunk: ::rkyv::Archive;
    #[repr(C)]
    struct VariantPartial(
        Tag,
        <PartialComplexEnumPartialChunk as ::rkyv::Archive>::Archived,
        ::core::marker::PhantomData<ArchivedPartialComplexEnumChunk>,
    )
    where
        PartialComplexEnumFullChunk: ::rkyv::Archive,
        PartialComplexEnumPartialChunk: ::rkyv::Archive;
    #[automatically_derived]
    unsafe impl<
        __C: ::rkyv::bytecheck::rancor::Fallible + ?::core::marker::Sized,
    > ::rkyv::bytecheck::CheckBytes<__C> for ArchivedPartialComplexEnumChunk
    where
        PartialComplexEnumFullChunk: ::rkyv::Archive,
        PartialComplexEnumPartialChunk: ::rkyv::Archive,
        <__C as ::rkyv::bytecheck::rancor::Fallible>::Error: ::rkyv::bytecheck::rancor::Source,
        <PartialComplexEnumFullChunk as ::rkyv::Archive>::Archived: ::rkyv::bytecheck::CheckBytes<
            __C,
        >,
        <PartialComplexEnumPartialChunk as ::rkyv::Archive>::Archived: ::rkyv::bytecheck::CheckBytes<
            __C,
        >,
    {
        unsafe fn check_bytes(
            value: *const Self,
            context: &mut __C,
        ) -> ::core::result::Result<
            (),
            <__C as ::rkyv::bytecheck::rancor::Fallible>::Error,
        > {
            let tag = *value.cast::<u8>();
            match tag {
                Discriminant::Full => {
                    let value = value.cast::<VariantFull>();
                    <<PartialComplexEnumFullChunk as ::rkyv::Archive>::Archived as ::rkyv::bytecheck::CheckBytes<
                        __C,
                    >>::check_bytes(&raw const (*value).1, context)
                        .map_err(|e| {
                            <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Trace>::trace(
                                e,
                                ::rkyv::bytecheck::UnnamedEnumVariantCheckContext {
                                    enum_name: "ArchivedPartialComplexEnumChunk",
                                    variant_name: "Full",
                                    field_index: 1,
                                },
                            )
                        })?;
                }
                Discriminant::Partial => {
                    let value = value.cast::<VariantPartial>();
                    <<PartialComplexEnumPartialChunk as ::rkyv::Archive>::Archived as ::rkyv::bytecheck::CheckBytes<
                        __C,
                    >>::check_bytes(&raw const (*value).1, context)
                        .map_err(|e| {
                            <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Trace>::trace(
                                e,
                                ::rkyv::bytecheck::UnnamedEnumVariantCheckContext {
                                    enum_name: "ArchivedPartialComplexEnumChunk",
                                    variant_name: "Partial",
                                    field_index: 1,
                                },
                            )
                        })?;
                }
                Discriminant::Missing => {}
                _ => {
                    return ::core::result::Result::Err(
                        <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Source>::new(::rkyv::bytecheck::InvalidEnumDiscriminantError {
                            enum_name: "ArchivedPartialComplexEnumChunk",
                            invalid_discriminant: tag,
                        }),
                    );
                }
            }
            ::core::result::Result::Ok(())
        }
    }
};
#[automatically_derived]
///The resolver for an archived [`PartialComplexEnumChunk`]
pub enum PartialComplexEnumChunkResolver
where
    PartialComplexEnumFullChunk: ::rkyv::Archive,
    PartialComplexEnumPartialChunk: ::rkyv::Archive,
{
    ///The resolver for [`PartialComplexEnumChunk::Full`]
    #[allow(dead_code)]
    Full(<PartialComplexEnumFullChunk as ::rkyv::Archive>::Resolver),
    ///The resolver for [`PartialComplexEnumChunk::Partial`]
    #[allow(dead_code)]
    Partial(<PartialComplexEnumPartialChunk as ::rkyv::Archive>::Resolver),
    ///The resolver for [`PartialComplexEnumChunk::Missing`]
    #[allow(dead_code)]
    Missing,
}
const _: () = {
    #[repr(u8)]
    enum ArchivedTag {
        Full,
        Partial,
        Missing,
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for ArchivedTag {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for ArchivedTag {
        #[inline]
        fn eq(&self, other: &ArchivedTag) -> bool {
            let __self_discr = ::core::intrinsics::discriminant_value(self);
            let __arg1_discr = ::core::intrinsics::discriminant_value(other);
            __self_discr == __arg1_discr
        }
    }
    #[automatically_derived]
    impl ::core::cmp::PartialOrd for ArchivedTag {
        #[inline]
        fn partial_cmp(
            &self,
            other: &ArchivedTag,
        ) -> ::core::option::Option<::core::cmp::Ordering> {
            let __self_discr = ::core::intrinsics::discriminant_value(self);
            let __arg1_discr = ::core::intrinsics::discriminant_value(other);
            ::core::cmp::PartialOrd::partial_cmp(&__self_discr, &__arg1_discr)
        }
    }
    #[repr(C)]
    struct ArchivedVariantFull(
        ArchivedTag,
        <PartialComplexEnumFullChunk as ::rkyv::Archive>::Archived,
        ::core::marker::PhantomData<PartialComplexEnumChunk>,
    )
    where
        PartialComplexEnumFullChunk: ::rkyv::Archive,
        PartialComplexEnumPartialChunk: ::rkyv::Archive;
    #[repr(C)]
    struct ArchivedVariantPartial(
        ArchivedTag,
        <PartialComplexEnumPartialChunk as ::rkyv::Archive>::Archived,
        ::core::marker::PhantomData<PartialComplexEnumChunk>,
    )
    where
        PartialComplexEnumFullChunk: ::rkyv::Archive,
        PartialComplexEnumPartialChunk: ::rkyv::Archive;
    impl ::rkyv::Archive for PartialComplexEnumChunk
    where
        PartialComplexEnumFullChunk: ::rkyv::Archive,
        PartialComplexEnumPartialChunk: ::rkyv::Archive,
    {
        type Archived = ArchivedPartialComplexEnumChunk;
        type Resolver = PartialComplexEnumChunkResolver;
        #[allow(clippy::unit_arg)]
        fn resolve(
            &self,
            resolver: <Self as ::rkyv::Archive>::Resolver,
            out: ::rkyv::Place<<Self as ::rkyv::Archive>::Archived>,
        ) {
            let __this = self;
            match resolver {
                PartialComplexEnumChunkResolver::Full(resolver_0) => {
                    match __this {
                        PartialComplexEnumChunk::Full(self_0, ..) => {
                            let out = unsafe {
                                out.cast_unchecked::<ArchivedVariantFull>()
                            };
                            let tag_ptr = unsafe { &raw mut (*out.ptr()).0 };
                            unsafe {
                                tag_ptr.write(ArchivedTag::Full);
                            }
                            let field_ptr = unsafe { &raw mut (*out.ptr()).1 };
                            let out_field = unsafe {
                                ::rkyv::Place::from_field_unchecked(out, field_ptr)
                            };
                            <PartialComplexEnumFullChunk as ::rkyv::Archive>::resolve(
                                self_0,
                                resolver_0,
                                out_field,
                            );
                        }
                        #[allow(unreachable_patterns)]
                        _ => unsafe { ::core::hint::unreachable_unchecked() }
                    }
                }
                PartialComplexEnumChunkResolver::Partial(resolver_0) => {
                    match __this {
                        PartialComplexEnumChunk::Partial(self_0, ..) => {
                            let out = unsafe {
                                out.cast_unchecked::<ArchivedVariantPartial>()
                            };
                            let tag_ptr = unsafe { &raw mut (*out.ptr()).0 };
                            unsafe {
                                tag_ptr.write(ArchivedTag::Partial);
                            }
                            let field_ptr = unsafe { &raw mut (*out.ptr()).1 };
                            let out_field = unsafe {
                                ::rkyv::Place::from_field_unchecked(out, field_ptr)
                            };
                            <PartialComplexEnumPartialChunk as ::rkyv::Archive>::resolve(
                                self_0,
                                resolver_0,
                                out_field,
                            );
                        }
                        #[allow(unreachable_patterns)]
                        _ => unsafe { ::core::hint::unreachable_unchecked() }
                    }
                }
                PartialComplexEnumChunkResolver::Missing => {
                    let out = unsafe { out.cast_unchecked::<ArchivedTag>() };
                    unsafe {
                        out.write_unchecked(ArchivedTag::Missing);
                    }
                }
            }
        }
    }
};
unsafe impl ::rkyv::traits::Portable for ArchivedPartialComplexEnumChunk
where
    PartialComplexEnumFullChunk: ::rkyv::Archive,
    PartialComplexEnumPartialChunk: ::rkyv::Archive,
    <PartialComplexEnumFullChunk as ::rkyv::Archive>::Archived: ::rkyv::traits::Portable,
    <PartialComplexEnumPartialChunk as ::rkyv::Archive>::Archived: ::rkyv::traits::Portable,
{}
#[automatically_derived]
impl<__S: ::rkyv::rancor::Fallible + ?Sized> ::rkyv::Serialize<__S>
for PartialComplexEnumChunk
where
    PartialComplexEnumFullChunk: ::rkyv::Serialize<__S>,
    PartialComplexEnumPartialChunk: ::rkyv::Serialize<__S>,
{
    fn serialize(
        &self,
        serializer: &mut __S,
    ) -> ::core::result::Result<
        <Self as ::rkyv::Archive>::Resolver,
        <__S as ::rkyv::rancor::Fallible>::Error,
    > {
        let __this = self;
        ::core::result::Result::Ok(
            match __this {
                PartialComplexEnumChunk::Full(_0, ..) => {
                    PartialComplexEnumChunkResolver::Full(
                        <PartialComplexEnumFullChunk as ::rkyv::Serialize<
                            __S,
                        >>::serialize(_0, serializer)?,
                    )
                }
                PartialComplexEnumChunk::Partial(_0, ..) => {
                    PartialComplexEnumChunkResolver::Partial(
                        <PartialComplexEnumPartialChunk as ::rkyv::Serialize<
                            __S,
                        >>::serialize(_0, serializer)?,
                    )
                }
                PartialComplexEnumChunk::Missing => {
                    PartialComplexEnumChunkResolver::Missing
                }
            },
        )
    }
}
#[automatically_derived]
impl<
    __D: ::rkyv::rancor::Fallible + ?Sized,
> ::rkyv::Deserialize<PartialComplexEnumChunk, __D>
for ::rkyv::Archived<PartialComplexEnumChunk>
where
    PartialComplexEnumFullChunk: ::rkyv::Archive,
    <PartialComplexEnumFullChunk as ::rkyv::Archive>::Archived: ::rkyv::Deserialize<
        PartialComplexEnumFullChunk,
        __D,
    >,
    PartialComplexEnumPartialChunk: ::rkyv::Archive,
    <PartialComplexEnumPartialChunk as ::rkyv::Archive>::Archived: ::rkyv::Deserialize<
        PartialComplexEnumPartialChunk,
        __D,
    >,
{
    fn deserialize(
        &self,
        deserializer: &mut __D,
    ) -> ::core::result::Result<
        PartialComplexEnumChunk,
        <__D as ::rkyv::rancor::Fallible>::Error,
    > {
        let __this = self;
        ::core::result::Result::Ok(
            match __this {
                Self::Full(_0, ..) => {
                    PartialComplexEnumChunk::Full(
                        <<PartialComplexEnumFullChunk as ::rkyv::Archive>::Archived as ::rkyv::Deserialize<
                            PartialComplexEnumFullChunk,
                            __D,
                        >>::deserialize(_0, deserializer)?,
                    )
                }
                Self::Partial(_0, ..) => {
                    PartialComplexEnumChunk::Partial(
                        <<PartialComplexEnumPartialChunk as ::rkyv::Archive>::Archived as ::rkyv::Deserialize<
                            PartialComplexEnumPartialChunk,
                            __D,
                        >>::deserialize(_0, deserializer)?,
                    )
                }
                Self::Missing => PartialComplexEnumChunk::Missing,
            },
        )
    }
}
pub enum PartialComplexEnumChunkFill {
    Full(usize),
    Partial(usize),
    Corrupted(usize),
}
#[automatically_derived]
impl ::core::fmt::Debug for PartialComplexEnumChunkFill {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match self {
            PartialComplexEnumChunkFill::Full(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "Full", &__self_0)
            }
            PartialComplexEnumChunkFill::Partial(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(
                    f,
                    "Partial",
                    &__self_0,
                )
            }
            PartialComplexEnumChunkFill::Corrupted(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(
                    f,
                    "Corrupted",
                    &__self_0,
                )
            }
        }
    }
}
#[automatically_derived]
#[doc(hidden)]
unsafe impl ::core::clone::TrivialClone for PartialComplexEnumChunkFill {}
#[automatically_derived]
impl ::core::clone::Clone for PartialComplexEnumChunkFill {
    #[inline]
    fn clone(&self) -> PartialComplexEnumChunkFill {
        let _: ::core::clone::AssertParamIsClone<usize>;
        *self
    }
}
#[automatically_derived]
impl ::core::marker::Copy for PartialComplexEnumChunkFill {}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for PartialComplexEnumChunkFill {}
#[automatically_derived]
impl ::core::cmp::PartialEq for PartialComplexEnumChunkFill {
    #[inline]
    fn eq(&self, other: &PartialComplexEnumChunkFill) -> bool {
        let __self_discr = ::core::intrinsics::discriminant_value(self);
        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
        __self_discr == __arg1_discr
            && match (self, other) {
                (
                    PartialComplexEnumChunkFill::Full(__self_0),
                    PartialComplexEnumChunkFill::Full(__arg1_0),
                ) => __self_0 == __arg1_0,
                (
                    PartialComplexEnumChunkFill::Partial(__self_0),
                    PartialComplexEnumChunkFill::Partial(__arg1_0),
                ) => __self_0 == __arg1_0,
                (
                    PartialComplexEnumChunkFill::Corrupted(__self_0),
                    PartialComplexEnumChunkFill::Corrupted(__arg1_0),
                ) => __self_0 == __arg1_0,
                _ => unsafe { ::core::intrinsics::unreachable() }
            }
    }
}
#[automatically_derived]
impl ::core::cmp::Eq for PartialComplexEnumChunkFill {
    #[inline]
    #[doc(hidden)]
    #[coverage(off)]
    fn assert_fields_are_eq(&self) {
        let _: ::core::cmp::AssertParamIsEq<usize>;
    }
}
#[automatically_derived]
impl ::core::cmp::PartialOrd for PartialComplexEnumChunkFill {
    #[inline]
    fn partial_cmp(
        &self,
        other: &PartialComplexEnumChunkFill,
    ) -> ::core::option::Option<::core::cmp::Ordering> {
        let __self_discr = ::core::intrinsics::discriminant_value(self);
        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
        match (self, other) {
            (
                PartialComplexEnumChunkFill::Full(__self_0),
                PartialComplexEnumChunkFill::Full(__arg1_0),
            ) => ::core::cmp::PartialOrd::partial_cmp(__self_0, __arg1_0),
            (
                PartialComplexEnumChunkFill::Partial(__self_0),
                PartialComplexEnumChunkFill::Partial(__arg1_0),
            ) => ::core::cmp::PartialOrd::partial_cmp(__self_0, __arg1_0),
            (
                PartialComplexEnumChunkFill::Corrupted(__self_0),
                PartialComplexEnumChunkFill::Corrupted(__arg1_0),
            ) => ::core::cmp::PartialOrd::partial_cmp(__self_0, __arg1_0),
            _ => ::core::cmp::PartialOrd::partial_cmp(&__self_discr, &__arg1_discr),
        }
    }
}
#[automatically_derived]
impl ::core::cmp::Ord for PartialComplexEnumChunkFill {
    #[inline]
    fn cmp(&self, other: &PartialComplexEnumChunkFill) -> ::core::cmp::Ordering {
        let __self_discr = ::core::intrinsics::discriminant_value(self);
        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
        match ::core::cmp::Ord::cmp(&__self_discr, &__arg1_discr) {
            ::core::cmp::Ordering::Equal => {
                match (self, other) {
                    (
                        PartialComplexEnumChunkFill::Full(__self_0),
                        PartialComplexEnumChunkFill::Full(__arg1_0),
                    ) => ::core::cmp::Ord::cmp(__self_0, __arg1_0),
                    (
                        PartialComplexEnumChunkFill::Partial(__self_0),
                        PartialComplexEnumChunkFill::Partial(__arg1_0),
                    ) => ::core::cmp::Ord::cmp(__self_0, __arg1_0),
                    (
                        PartialComplexEnumChunkFill::Corrupted(__self_0),
                        PartialComplexEnumChunkFill::Corrupted(__arg1_0),
                    ) => ::core::cmp::Ord::cmp(__self_0, __arg1_0),
                    _ => unsafe { ::core::intrinsics::unreachable() }
                }
            }
            cmp => cmp,
        }
    }
}
#[automatically_derived]
///An archived [`PartialComplexEnumChunkFill`]
#[bytecheck(crate = ::rkyv::bytecheck)]
#[repr(u8)]
pub enum ArchivedPartialComplexEnumChunkFill
where
    usize: ::rkyv::Archive,
    usize: ::rkyv::Archive,
    usize: ::rkyv::Archive,
{
    ///The archived counterpart of [`PartialComplexEnumChunkFill::Full`]
    #[allow(dead_code)]
    Full(
        ///The archived counterpart of [`PartialComplexEnumChunkFill::Full::0`]
        <usize as ::rkyv::Archive>::Archived,
    ),
    ///The archived counterpart of [`PartialComplexEnumChunkFill::Partial`]
    #[allow(dead_code)]
    Partial(
        ///The archived counterpart of [`PartialComplexEnumChunkFill::Partial::0`]
        <usize as ::rkyv::Archive>::Archived,
    ),
    ///The archived counterpart of [`PartialComplexEnumChunkFill::Corrupted`]
    #[allow(dead_code)]
    Corrupted(
        ///The archived counterpart of [`PartialComplexEnumChunkFill::Corrupted::0`]
        <usize as ::rkyv::Archive>::Archived,
    ),
}
const _: () = {
    #[repr(u8)]
    enum Tag {
        Full,
        Partial,
        Corrupted,
    }
    struct Discriminant;
    #[automatically_derived]
    impl Discriminant {
        #[allow(non_upper_case_globals)]
        const Full: u8 = Tag::Full as u8;
        #[allow(non_upper_case_globals)]
        const Partial: u8 = Tag::Partial as u8;
        #[allow(non_upper_case_globals)]
        const Corrupted: u8 = Tag::Corrupted as u8;
    }
    #[repr(C)]
    struct VariantFull(
        Tag,
        <usize as ::rkyv::Archive>::Archived,
        ::core::marker::PhantomData<ArchivedPartialComplexEnumChunkFill>,
    )
    where
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive;
    #[repr(C)]
    struct VariantPartial(
        Tag,
        <usize as ::rkyv::Archive>::Archived,
        ::core::marker::PhantomData<ArchivedPartialComplexEnumChunkFill>,
    )
    where
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive;
    #[repr(C)]
    struct VariantCorrupted(
        Tag,
        <usize as ::rkyv::Archive>::Archived,
        ::core::marker::PhantomData<ArchivedPartialComplexEnumChunkFill>,
    )
    where
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive;
    #[automatically_derived]
    unsafe impl<
        __C: ::rkyv::bytecheck::rancor::Fallible + ?::core::marker::Sized,
    > ::rkyv::bytecheck::CheckBytes<__C> for ArchivedPartialComplexEnumChunkFill
    where
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive,
        <__C as ::rkyv::bytecheck::rancor::Fallible>::Error: ::rkyv::bytecheck::rancor::Source,
        <usize as ::rkyv::Archive>::Archived: ::rkyv::bytecheck::CheckBytes<__C>,
        <usize as ::rkyv::Archive>::Archived: ::rkyv::bytecheck::CheckBytes<__C>,
        <usize as ::rkyv::Archive>::Archived: ::rkyv::bytecheck::CheckBytes<__C>,
    {
        unsafe fn check_bytes(
            value: *const Self,
            context: &mut __C,
        ) -> ::core::result::Result<
            (),
            <__C as ::rkyv::bytecheck::rancor::Fallible>::Error,
        > {
            let tag = *value.cast::<u8>();
            match tag {
                Discriminant::Full => {
                    let value = value.cast::<VariantFull>();
                    <<usize as ::rkyv::Archive>::Archived as ::rkyv::bytecheck::CheckBytes<
                        __C,
                    >>::check_bytes(&raw const (*value).1, context)
                        .map_err(|e| {
                            <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Trace>::trace(
                                e,
                                ::rkyv::bytecheck::UnnamedEnumVariantCheckContext {
                                    enum_name: "ArchivedPartialComplexEnumChunkFill",
                                    variant_name: "Full",
                                    field_index: 1,
                                },
                            )
                        })?;
                }
                Discriminant::Partial => {
                    let value = value.cast::<VariantPartial>();
                    <<usize as ::rkyv::Archive>::Archived as ::rkyv::bytecheck::CheckBytes<
                        __C,
                    >>::check_bytes(&raw const (*value).1, context)
                        .map_err(|e| {
                            <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Trace>::trace(
                                e,
                                ::rkyv::bytecheck::UnnamedEnumVariantCheckContext {
                                    enum_name: "ArchivedPartialComplexEnumChunkFill",
                                    variant_name: "Partial",
                                    field_index: 1,
                                },
                            )
                        })?;
                }
                Discriminant::Corrupted => {
                    let value = value.cast::<VariantCorrupted>();
                    <<usize as ::rkyv::Archive>::Archived as ::rkyv::bytecheck::CheckBytes<
                        __C,
                    >>::check_bytes(&raw const (*value).1, context)
                        .map_err(|e| {
                            <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Trace>::trace(
                                e,
                                ::rkyv::bytecheck::UnnamedEnumVariantCheckContext {
                                    enum_name: "ArchivedPartialComplexEnumChunkFill",
                                    variant_name: "Corrupted",
                                    field_index: 1,
                                },
                            )
                        })?;
                }
                _ => {
                    return ::core::result::Result::Err(
                        <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Source>::new(::rkyv::bytecheck::InvalidEnumDiscriminantError {
                            enum_name: "ArchivedPartialComplexEnumChunkFill",
                            invalid_discriminant: tag,
                        }),
                    );
                }
            }
            ::core::result::Result::Ok(())
        }
    }
};
#[automatically_derived]
///The resolver for an archived [`PartialComplexEnumChunkFill`]
pub enum PartialComplexEnumChunkFillResolver
where
    usize: ::rkyv::Archive,
    usize: ::rkyv::Archive,
    usize: ::rkyv::Archive,
{
    ///The resolver for [`PartialComplexEnumChunkFill::Full`]
    #[allow(dead_code)]
    Full(<usize as ::rkyv::Archive>::Resolver),
    ///The resolver for [`PartialComplexEnumChunkFill::Partial`]
    #[allow(dead_code)]
    Partial(<usize as ::rkyv::Archive>::Resolver),
    ///The resolver for [`PartialComplexEnumChunkFill::Corrupted`]
    #[allow(dead_code)]
    Corrupted(<usize as ::rkyv::Archive>::Resolver),
}
const _: () = {
    #[repr(u8)]
    enum ArchivedTag {
        Full,
        Partial,
        Corrupted,
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for ArchivedTag {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for ArchivedTag {
        #[inline]
        fn eq(&self, other: &ArchivedTag) -> bool {
            let __self_discr = ::core::intrinsics::discriminant_value(self);
            let __arg1_discr = ::core::intrinsics::discriminant_value(other);
            __self_discr == __arg1_discr
        }
    }
    #[automatically_derived]
    impl ::core::cmp::PartialOrd for ArchivedTag {
        #[inline]
        fn partial_cmp(
            &self,
            other: &ArchivedTag,
        ) -> ::core::option::Option<::core::cmp::Ordering> {
            let __self_discr = ::core::intrinsics::discriminant_value(self);
            let __arg1_discr = ::core::intrinsics::discriminant_value(other);
            ::core::cmp::PartialOrd::partial_cmp(&__self_discr, &__arg1_discr)
        }
    }
    #[repr(C)]
    struct ArchivedVariantFull(
        ArchivedTag,
        <usize as ::rkyv::Archive>::Archived,
        ::core::marker::PhantomData<PartialComplexEnumChunkFill>,
    )
    where
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive;
    #[repr(C)]
    struct ArchivedVariantPartial(
        ArchivedTag,
        <usize as ::rkyv::Archive>::Archived,
        ::core::marker::PhantomData<PartialComplexEnumChunkFill>,
    )
    where
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive;
    #[repr(C)]
    struct ArchivedVariantCorrupted(
        ArchivedTag,
        <usize as ::rkyv::Archive>::Archived,
        ::core::marker::PhantomData<PartialComplexEnumChunkFill>,
    )
    where
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive;
    impl ::rkyv::Archive for PartialComplexEnumChunkFill
    where
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive,
    {
        type Archived = ArchivedPartialComplexEnumChunkFill;
        type Resolver = PartialComplexEnumChunkFillResolver;
        #[allow(clippy::unit_arg)]
        fn resolve(
            &self,
            resolver: <Self as ::rkyv::Archive>::Resolver,
            out: ::rkyv::Place<<Self as ::rkyv::Archive>::Archived>,
        ) {
            let __this = self;
            match resolver {
                PartialComplexEnumChunkFillResolver::Full(resolver_0) => {
                    match __this {
                        PartialComplexEnumChunkFill::Full(self_0, ..) => {
                            let out = unsafe {
                                out.cast_unchecked::<ArchivedVariantFull>()
                            };
                            let tag_ptr = unsafe { &raw mut (*out.ptr()).0 };
                            unsafe {
                                tag_ptr.write(ArchivedTag::Full);
                            }
                            let field_ptr = unsafe { &raw mut (*out.ptr()).1 };
                            let out_field = unsafe {
                                ::rkyv::Place::from_field_unchecked(out, field_ptr)
                            };
                            <usize as ::rkyv::Archive>::resolve(
                                self_0,
                                resolver_0,
                                out_field,
                            );
                        }
                        #[allow(unreachable_patterns)]
                        _ => unsafe { ::core::hint::unreachable_unchecked() }
                    }
                }
                PartialComplexEnumChunkFillResolver::Partial(resolver_0) => {
                    match __this {
                        PartialComplexEnumChunkFill::Partial(self_0, ..) => {
                            let out = unsafe {
                                out.cast_unchecked::<ArchivedVariantPartial>()
                            };
                            let tag_ptr = unsafe { &raw mut (*out.ptr()).0 };
                            unsafe {
                                tag_ptr.write(ArchivedTag::Partial);
                            }
                            let field_ptr = unsafe { &raw mut (*out.ptr()).1 };
                            let out_field = unsafe {
                                ::rkyv::Place::from_field_unchecked(out, field_ptr)
                            };
                            <usize as ::rkyv::Archive>::resolve(
                                self_0,
                                resolver_0,
                                out_field,
                            );
                        }
                        #[allow(unreachable_patterns)]
                        _ => unsafe { ::core::hint::unreachable_unchecked() }
                    }
                }
                PartialComplexEnumChunkFillResolver::Corrupted(resolver_0) => {
                    match __this {
                        PartialComplexEnumChunkFill::Corrupted(self_0, ..) => {
                            let out = unsafe {
                                out.cast_unchecked::<ArchivedVariantCorrupted>()
                            };
                            let tag_ptr = unsafe { &raw mut (*out.ptr()).0 };
                            unsafe {
                                tag_ptr.write(ArchivedTag::Corrupted);
                            }
                            let field_ptr = unsafe { &raw mut (*out.ptr()).1 };
                            let out_field = unsafe {
                                ::rkyv::Place::from_field_unchecked(out, field_ptr)
                            };
                            <usize as ::rkyv::Archive>::resolve(
                                self_0,
                                resolver_0,
                                out_field,
                            );
                        }
                        #[allow(unreachable_patterns)]
                        _ => unsafe { ::core::hint::unreachable_unchecked() }
                    }
                }
            }
        }
    }
};
unsafe impl ::rkyv::traits::Portable for ArchivedPartialComplexEnumChunkFill
where
    usize: ::rkyv::Archive,
    usize: ::rkyv::Archive,
    usize: ::rkyv::Archive,
    <usize as ::rkyv::Archive>::Archived: ::rkyv::traits::Portable,
    <usize as ::rkyv::Archive>::Archived: ::rkyv::traits::Portable,
    <usize as ::rkyv::Archive>::Archived: ::rkyv::traits::Portable,
{}
#[automatically_derived]
impl<__S: ::rkyv::rancor::Fallible + ?Sized> ::rkyv::Serialize<__S>
for PartialComplexEnumChunkFill
where
    usize: ::rkyv::Serialize<__S>,
    usize: ::rkyv::Serialize<__S>,
    usize: ::rkyv::Serialize<__S>,
{
    fn serialize(
        &self,
        serializer: &mut __S,
    ) -> ::core::result::Result<
        <Self as ::rkyv::Archive>::Resolver,
        <__S as ::rkyv::rancor::Fallible>::Error,
    > {
        let __this = self;
        ::core::result::Result::Ok(
            match __this {
                PartialComplexEnumChunkFill::Full(_0, ..) => {
                    PartialComplexEnumChunkFillResolver::Full(
                        <usize as ::rkyv::Serialize<__S>>::serialize(_0, serializer)?,
                    )
                }
                PartialComplexEnumChunkFill::Partial(_0, ..) => {
                    PartialComplexEnumChunkFillResolver::Partial(
                        <usize as ::rkyv::Serialize<__S>>::serialize(_0, serializer)?,
                    )
                }
                PartialComplexEnumChunkFill::Corrupted(_0, ..) => {
                    PartialComplexEnumChunkFillResolver::Corrupted(
                        <usize as ::rkyv::Serialize<__S>>::serialize(_0, serializer)?,
                    )
                }
            },
        )
    }
}
#[automatically_derived]
impl<
    __D: ::rkyv::rancor::Fallible + ?Sized,
> ::rkyv::Deserialize<PartialComplexEnumChunkFill, __D>
for ::rkyv::Archived<PartialComplexEnumChunkFill>
where
    usize: ::rkyv::Archive,
    <usize as ::rkyv::Archive>::Archived: ::rkyv::Deserialize<usize, __D>,
    usize: ::rkyv::Archive,
    <usize as ::rkyv::Archive>::Archived: ::rkyv::Deserialize<usize, __D>,
    usize: ::rkyv::Archive,
    <usize as ::rkyv::Archive>::Archived: ::rkyv::Deserialize<usize, __D>,
{
    fn deserialize(
        &self,
        deserializer: &mut __D,
    ) -> ::core::result::Result<
        PartialComplexEnumChunkFill,
        <__D as ::rkyv::rancor::Fallible>::Error,
    > {
        let __this = self;
        ::core::result::Result::Ok(
            match __this {
                Self::Full(_0, ..) => {
                    PartialComplexEnumChunkFill::Full(
                        <<usize as ::rkyv::Archive>::Archived as ::rkyv::Deserialize<
                            usize,
                            __D,
                        >>::deserialize(_0, deserializer)?,
                    )
                }
                Self::Partial(_0, ..) => {
                    PartialComplexEnumChunkFill::Partial(
                        <<usize as ::rkyv::Archive>::Archived as ::rkyv::Deserialize<
                            usize,
                            __D,
                        >>::deserialize(_0, deserializer)?,
                    )
                }
                Self::Corrupted(_0, ..) => {
                    PartialComplexEnumChunkFill::Corrupted(
                        <<usize as ::rkyv::Archive>::Archived as ::rkyv::Deserialize<
                            usize,
                            __D,
                        >>::deserialize(_0, deserializer)?,
                    )
                }
            },
        )
    }
}
impl PartialComplexEnumChunkFill {
    pub fn from_size(actual: usize, expected: usize) -> Self {
        if actual == expected {
            Self::Full(actual)
        } else if actual < expected {
            Self::Partial(actual)
        } else {
            Self::Corrupted(actual)
        }
    }
}
impl rewrite::traits::structural::blob::BlobItemChunk for PartialComplexEnumChunk {
    type Index = usize;
    fn get_index(&self) -> &Self::Index {
        match self {
            Self::Full(c) => c.get_index(),
            Self::Partial(c) => c.get_index(),
            _ => {
                ::core::panicking::panic_fmt(
                    format_args!("get_index called on missing chunk"),
                );
            }
        }
    }
}
impl rewrite::traits::structural::blob::NetabaseBlobItem for PartialComplexEnum {
    type Chunk = PartialComplexEnumChunk;
    type BlobIter = std::vec::IntoIter<rewrite::results::NetabaseResult<Self::Chunk>>;
    const DEFAULT_CHUNK_SIZE: usize = 0usize;
    fn into_chunks(
        self,
        size: rewrite::traits::structural::blob::ChunkSize,
    ) -> Box<dyn Iterator<Item = Self::Chunk>> {
        Box::new(self.into_chunks_iter(size).filter_map(|r| r.ok()))
    }
    fn into_chunks_iter(
        self,
        size: rewrite::traits::structural::blob::ChunkSize,
    ) -> Self::BlobIter {
        let mut all_chunks = Vec::new();
        match self {
            Self::Full(ref f0) => {
                let serialized: rewrite::results::NetabaseResult<Vec<u8>> = rkyv::to_bytes::<
                    rkyv::rancor::Error,
                >(&self)
                    .map_err(|e| rewrite::results::NetabaseError::Serialization(
                        ::alloc::__export::must_use({
                            ::alloc::fmt::format(
                                format_args!(
                                    "rkyv serialization failed for enum variant {0}: {1:?}",
                                    "Full",
                                    e,
                                ),
                            )
                        }),
                    ))
                    .map(|d| d.to_vec());
                match serialized {
                    Ok(data) => {
                        let chunk_size = match size {
                            rewrite::traits::structural::blob::ChunkSize::Default => {}
                            rewrite::traits::structural::blob::ChunkSize::Size(n) => n,
                        };
                        all_chunks
                            .extend(
                                data
                                    .chunks(chunk_size)
                                    .enumerate()
                                    .map(|(index, chunk_data)| {
                                        Ok(
                                            Self::Chunk::Full(PartialComplexEnumFullChunk {
                                                index,
                                                data: chunk_data.to_vec(),
                                            }),
                                        )
                                    }),
                            );
                    }
                    Err(e) => all_chunks.push(Err(e)),
                }
            }
            Self::Partial { meta: ref meta, payload: ref payload } => {
                {
                    let serialized_field: rewrite::results::NetabaseResult<Vec<u8>> = rkyv::to_bytes::<
                        rkyv::rancor::Error,
                    >(meta)
                        .map_err(|e| rewrite::results::NetabaseError::Serialization(
                            ::alloc::__export::must_use({
                                ::alloc::fmt::format(
                                    format_args!(
                                        "rkyv serialization failed for variant {0} field {1}: {2:?}",
                                        "Partial",
                                        "meta",
                                        e,
                                    ),
                                )
                            }),
                        ))
                        .map(|d| d.to_vec());
                    match serialized_field {
                        Ok(data) => {
                            let chunk_size = match size {
                                rewrite::traits::structural::blob::ChunkSize::Default => {
                                    32usize
                                }
                                rewrite::traits::structural::blob::ChunkSize::Size(n) => n,
                            };
                            all_chunks
                                .extend(
                                    data
                                        .chunks(chunk_size)
                                        .enumerate()
                                        .map(|(index, chunk_data)| {
                                            Ok(
                                                Self::Chunk::Partial(
                                                    PartialComplexEnumPartialChunk::Meta(PartialComplexEnumPartialMetaChunk {
                                                        index,
                                                        data: chunk_data.to_vec(),
                                                    }),
                                                ),
                                            )
                                        }),
                                );
                        }
                        Err(e) => all_chunks.push(Err(e)),
                    }
                }
                {
                    let serialized_field: rewrite::results::NetabaseResult<Vec<u8>> = rkyv::to_bytes::<
                        rkyv::rancor::Error,
                    >(payload)
                        .map_err(|e| rewrite::results::NetabaseError::Serialization(
                            ::alloc::__export::must_use({
                                ::alloc::fmt::format(
                                    format_args!(
                                        "rkyv serialization failed for variant {0} field {1}: {2:?}",
                                        "Partial",
                                        "payload",
                                        e,
                                    ),
                                )
                            }),
                        ))
                        .map(|d| d.to_vec());
                    match serialized_field {
                        Ok(data) => {
                            let chunk_size = match size {
                                rewrite::traits::structural::blob::ChunkSize::Default => {
                                    64usize
                                }
                                rewrite::traits::structural::blob::ChunkSize::Size(n) => n,
                            };
                            all_chunks
                                .extend(
                                    data
                                        .chunks(chunk_size)
                                        .enumerate()
                                        .map(|(index, chunk_data)| {
                                            Ok(
                                                Self::Chunk::Partial(
                                                    PartialComplexEnumPartialChunk::Payload(PartialComplexEnumPartialPayloadChunk {
                                                        index,
                                                        data: chunk_data.to_vec(),
                                                    }),
                                                ),
                                            )
                                        }),
                                );
                        }
                        Err(e) => all_chunks.push(Err(e)),
                    }
                }
            }
        }
        all_chunks.into_iter()
    }
    fn try_from_chunks(
        chunks: impl Iterator<Item = Self::Chunk>,
        size: rewrite::traits::structural::blob::ChunkSize,
    ) -> rewrite::results::NetabaseResult<Self> {
        let mut all_variant_chunks: std::collections::HashMap<
            String,
            Vec<Self::Chunk>,
        > = std::collections::HashMap::new();
        for chunk in chunks {
            let key = match &chunk {
                Self::Chunk::Full(_) => "Full".to_string(),
                Self::Chunk::Partial(_) => "Partial".to_string(),
                _ => "Unknown".to_string(),
            };
            all_variant_chunks.entry(key).or_default().push(chunk);
        }
        for (variant_name, chunks) in all_variant_chunks {
            let res: rewrite::results::NetabaseResult<Self> = match variant_name.as_str()
            {
                "Full" => {
                    (|| {
                        let chunks = chunks;
                        let mut sorted = chunks;
                        sorted
                            .sort_by_key(|c| match c {
                                Self::Chunk::Full(inner) => inner.index,
                                _ => 0,
                            });
                        let data: Vec<u8> = sorted
                            .into_iter()
                            .flat_map(|c| match c {
                                Self::Chunk::Full(inner) => inner.data,
                                _ => Vec::new(),
                            })
                            .collect();
                        let val: Self = rkyv::from_bytes::<
                            Self,
                            rkyv::rancor::Error,
                        >(&data)
                            .map_err(|e| rewrite::results::NetabaseError::Serialization(
                                ::alloc::__export::must_use({
                                    ::alloc::fmt::format(
                                        format_args!(
                                            "rkyv deserialization failed for variant {0}: {1:?}",
                                            "Full",
                                            e,
                                        ),
                                    )
                                }),
                            ))?;
                        return Ok(val);
                    })()
                }
                "Partial" => {
                    (|| {
                        let chunks = chunks;
                        let mut field_0 = Vec::new();
                        let mut field_1 = Vec::new();
                        for c in chunks {
                            if let Self::Chunk::Partial(inner) = c {
                                match inner {
                                    PartialComplexEnumPartialChunk::Meta(field_chunk) => {
                                        field_0.push(field_chunk);
                                    }
                                    PartialComplexEnumPartialChunk::Payload(field_chunk) => {
                                        field_1.push(field_chunk);
                                    }
                                    _ => {}
                                }
                            }
                        }
                        let field_0 = {
                            if field_0.is_empty() {
                                return Err(
                                    rewrite::results::NetabaseError::BlobReconstruction(
                                        rewrite::results::BlobReconstructionError::MissingChunks,
                                    ),
                                );
                            }
                            let mut sorted = field_0;
                            sorted.sort_by_key(|c| c.index);
                            let chunk_size = match size {
                                rewrite::traits::structural::blob::ChunkSize::Default => {
                                    32usize
                                }
                                rewrite::traits::structural::blob::ChunkSize::Size(n) => n,
                            };
                            let mut next_expected = 0;
                            let max_idx = sorted.last().map(|c| c.index).unwrap_or(0);
                            for chunk in &sorted {
                                if chunk.index > next_expected {
                                    return Err(
                                        rewrite::results::NetabaseError::BlobReconstruction(
                                            rewrite::results::BlobReconstructionError::InvalidChunkData(
                                                ::alloc::__export::must_use({
                                                    ::alloc::fmt::format(
                                                        format_args!("Gap at index {0}", next_expected),
                                                    )
                                                }),
                                            ),
                                        ),
                                    );
                                }
                                if chunk.data.len() > chunk_size {
                                    return Err(
                                        rewrite::results::NetabaseError::BlobReconstruction(
                                            rewrite::results::BlobReconstructionError::InvalidChunkData(
                                                ::alloc::__export::must_use({
                                                    ::alloc::fmt::format(
                                                        format_args!("Chunk overflow at index {0}", chunk.index),
                                                    )
                                                }),
                                            ),
                                        ),
                                    );
                                }
                                if chunk.data.len() < chunk_size && chunk.index < max_idx {
                                    return Err(
                                        rewrite::results::NetabaseError::BlobReconstruction(
                                            rewrite::results::BlobReconstructionError::InvalidChunkData(
                                                ::alloc::__export::must_use({
                                                    ::alloc::fmt::format(
                                                        format_args!(
                                                            "Partial chunk in middle at index {0}",
                                                            chunk.index,
                                                        ),
                                                    )
                                                }),
                                            ),
                                        ),
                                    );
                                }
                                next_expected += 1;
                            }
                            let data: Vec<u8> = sorted
                                .into_iter()
                                .flat_map(|c| c.data)
                                .collect();
                            rkyv::from_bytes::<String, rkyv::rancor::Error>(&data)
                                .map_err(|e| rewrite::results::NetabaseError::Serialization(
                                    ::alloc::__export::must_use({
                                        ::alloc::fmt::format(
                                            format_args!(
                                                "rkyv deserialization failed for field {0}: {1:?}",
                                                "meta",
                                                e,
                                            ),
                                        )
                                    }),
                                ))?
                        };
                        let field_1 = {
                            if field_1.is_empty() {
                                return Err(
                                    rewrite::results::NetabaseError::BlobReconstruction(
                                        rewrite::results::BlobReconstructionError::MissingChunks,
                                    ),
                                );
                            }
                            let mut sorted = field_1;
                            sorted.sort_by_key(|c| c.index);
                            let chunk_size = match size {
                                rewrite::traits::structural::blob::ChunkSize::Default => {
                                    64usize
                                }
                                rewrite::traits::structural::blob::ChunkSize::Size(n) => n,
                            };
                            let mut next_expected = 0;
                            let max_idx = sorted.last().map(|c| c.index).unwrap_or(0);
                            for chunk in &sorted {
                                if chunk.index > next_expected {
                                    return Err(
                                        rewrite::results::NetabaseError::BlobReconstruction(
                                            rewrite::results::BlobReconstructionError::InvalidChunkData(
                                                ::alloc::__export::must_use({
                                                    ::alloc::fmt::format(
                                                        format_args!("Gap at index {0}", next_expected),
                                                    )
                                                }),
                                            ),
                                        ),
                                    );
                                }
                                if chunk.data.len() > chunk_size {
                                    return Err(
                                        rewrite::results::NetabaseError::BlobReconstruction(
                                            rewrite::results::BlobReconstructionError::InvalidChunkData(
                                                ::alloc::__export::must_use({
                                                    ::alloc::fmt::format(
                                                        format_args!("Chunk overflow at index {0}", chunk.index),
                                                    )
                                                }),
                                            ),
                                        ),
                                    );
                                }
                                if chunk.data.len() < chunk_size && chunk.index < max_idx {
                                    return Err(
                                        rewrite::results::NetabaseError::BlobReconstruction(
                                            rewrite::results::BlobReconstructionError::InvalidChunkData(
                                                ::alloc::__export::must_use({
                                                    ::alloc::fmt::format(
                                                        format_args!(
                                                            "Partial chunk in middle at index {0}",
                                                            chunk.index,
                                                        ),
                                                    )
                                                }),
                                            ),
                                        ),
                                    );
                                }
                                next_expected += 1;
                            }
                            let data: Vec<u8> = sorted
                                .into_iter()
                                .flat_map(|c| c.data)
                                .collect();
                            rkyv::from_bytes::<Vec<u8>, rkyv::rancor::Error>(&data)
                                .map_err(|e| rewrite::results::NetabaseError::Serialization(
                                    ::alloc::__export::must_use({
                                        ::alloc::fmt::format(
                                            format_args!(
                                                "rkyv deserialization failed for field {0}: {1:?}",
                                                "payload",
                                                e,
                                            ),
                                        )
                                    }),
                                ))?
                        };
                        return Ok(Self::Partial {
                            meta: field_0,
                            payload: field_1,
                        });
                    })()
                }
                _ => {
                    Err(
                        rewrite::results::NetabaseError::BlobReconstruction(
                            rewrite::results::BlobReconstructionError::InvalidChunkData(
                                ::alloc::__export::must_use({
                                    ::alloc::fmt::format(
                                        format_args!("Unknown variant {0}", variant_name),
                                    )
                                }),
                            ),
                        ),
                    )
                }
            };
            if res.is_ok() {
                return res;
            }
        }
        Err(
            rewrite::results::NetabaseError::BlobReconstruction(
                rewrite::results::BlobReconstructionError::InvalidChunkData(
                    "Could not reconstruct any variant".to_string(),
                ),
            ),
        )
    }
    fn get_blob(&self) -> &Self::Chunk {
        ::core::panicking::panic("not implemented")
    }
}
impl IntoIterator for PartialComplexEnum {
    type Item = rewrite::results::NetabaseResult<PartialComplexEnumChunk>;
    type IntoIter = std::vec::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        rewrite::traits::structural::blob::NetabaseBlobItem::into_chunks_iter(
            self,
            rewrite::traits::structural::blob::ChunkSize::Default,
        )
    }
}
#[blob(strategy = "full")]
struct ForcedFull {
    #[chunk_size(64)]
    field1: String,
}
#[automatically_derived]
///An archived [`ForcedFull`]
#[bytecheck(crate = ::rkyv::bytecheck)]
#[repr(C)]
struct ArchivedForcedFull
where
    String: ::rkyv::Archive,
{
    ///The archived counterpart of [`ForcedFull::field1`]
    field1: <String as ::rkyv::Archive>::Archived,
}
#[automatically_derived]
unsafe impl<
    __C: ::rkyv::bytecheck::rancor::Fallible + ?::core::marker::Sized,
> ::rkyv::bytecheck::CheckBytes<__C> for ArchivedForcedFull
where
    String: ::rkyv::Archive,
    <__C as ::rkyv::bytecheck::rancor::Fallible>::Error: ::rkyv::bytecheck::rancor::Trace,
    <String as ::rkyv::Archive>::Archived: ::rkyv::bytecheck::CheckBytes<__C>,
{
    unsafe fn check_bytes(
        value: *const Self,
        context: &mut __C,
    ) -> ::core::result::Result<
        (),
        <__C as ::rkyv::bytecheck::rancor::Fallible>::Error,
    > {
        <<String as ::rkyv::Archive>::Archived as ::rkyv::bytecheck::CheckBytes<
            __C,
        >>::check_bytes(&raw const (*value).field1, context)
            .map_err(|e| {
                <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Trace>::trace(
                    e,
                    ::rkyv::bytecheck::StructCheckContext {
                        struct_name: "ArchivedForcedFull",
                        field_name: "field1",
                    },
                )
            })?;
        ::core::result::Result::Ok(())
    }
}
#[automatically_derived]
///The resolver for an archived [`ForcedFull`]
struct ForcedFullResolver
where
    String: ::rkyv::Archive,
{
    field1: <String as ::rkyv::Archive>::Resolver,
}
impl ::rkyv::Archive for ForcedFull
where
    String: ::rkyv::Archive,
{
    type Archived = ArchivedForcedFull;
    type Resolver = ForcedFullResolver;
    const COPY_OPTIMIZATION: ::rkyv::traits::CopyOptimization<Self> = unsafe {
        ::rkyv::traits::CopyOptimization::enable_if(
            0 + ::core::mem::size_of::<String>() == ::core::mem::size_of::<ForcedFull>()
                && <String as ::rkyv::Archive>::COPY_OPTIMIZATION.is_enabled()
                && const { builtin # offset_of(ForcedFull, field1) }
                    == const { builtin # offset_of(ArchivedForcedFull, field1) },
        )
    };
    #[allow(clippy::unit_arg)]
    fn resolve(&self, resolver: Self::Resolver, out: ::rkyv::Place<Self::Archived>) {
        let field_ptr = unsafe { &raw mut (*out.ptr()).field1 };
        let field_out = unsafe { ::rkyv::Place::from_field_unchecked(out, field_ptr) };
        <String as ::rkyv::Archive>::resolve(&self.field1, resolver.field1, field_out);
    }
}
unsafe impl ::rkyv::traits::Portable for ArchivedForcedFull
where
    String: ::rkyv::Archive,
    <String as ::rkyv::Archive>::Archived: ::rkyv::traits::Portable,
{}
#[automatically_derived]
impl<__S: ::rkyv::rancor::Fallible + ?Sized> ::rkyv::Serialize<__S> for ForcedFull
where
    String: ::rkyv::Serialize<__S>,
{
    fn serialize(
        &self,
        serializer: &mut __S,
    ) -> ::core::result::Result<
        <Self as ::rkyv::Archive>::Resolver,
        <__S as ::rkyv::rancor::Fallible>::Error,
    > {
        let __this = self;
        ::core::result::Result::Ok(ForcedFullResolver {
            field1: <String as ::rkyv::Serialize<
                __S,
            >>::serialize(&__this.field1, serializer)?,
        })
    }
}
#[automatically_derived]
impl<__D: ::rkyv::rancor::Fallible + ?Sized> ::rkyv::Deserialize<ForcedFull, __D>
for ::rkyv::Archived<ForcedFull>
where
    String: ::rkyv::Archive,
    <String as ::rkyv::Archive>::Archived: ::rkyv::Deserialize<String, __D>,
{
    fn deserialize(
        &self,
        deserializer: &mut __D,
    ) -> ::core::result::Result<ForcedFull, <__D as ::rkyv::rancor::Fallible>::Error> {
        let __this = self;
        ::core::result::Result::Ok(ForcedFull {
            field1: <<String as ::rkyv::Archive>::Archived as ::rkyv::Deserialize<
                String,
                __D,
            >>::deserialize(&__this.field1, deserializer)?,
        })
    }
}
#[automatically_derived]
impl ::core::fmt::Debug for ForcedFull {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::debug_struct_field1_finish(
            f,
            "ForcedFull",
            "field1",
            &&self.field1,
        )
    }
}
#[automatically_derived]
impl ::core::clone::Clone for ForcedFull {
    #[inline]
    fn clone(&self) -> ForcedFull {
        ForcedFull {
            field1: ::core::clone::Clone::clone(&self.field1),
        }
    }
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for ForcedFull {}
#[automatically_derived]
impl ::core::cmp::PartialEq for ForcedFull {
    #[inline]
    fn eq(&self, other: &ForcedFull) -> bool {
        self.field1 == other.field1
    }
}
#[automatically_derived]
impl ::core::cmp::Eq for ForcedFull {
    #[inline]
    #[doc(hidden)]
    #[coverage(off)]
    fn assert_fields_are_eq(&self) {
        let _: ::core::cmp::AssertParamIsEq<String>;
    }
}
pub struct ForcedFullChunk {
    pub index: usize,
    pub data: Vec<u8>,
}
#[automatically_derived]
impl ::core::fmt::Debug for ForcedFullChunk {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::debug_struct_field2_finish(
            f,
            "ForcedFullChunk",
            "index",
            &self.index,
            "data",
            &&self.data,
        )
    }
}
#[automatically_derived]
impl ::core::clone::Clone for ForcedFullChunk {
    #[inline]
    fn clone(&self) -> ForcedFullChunk {
        ForcedFullChunk {
            index: ::core::clone::Clone::clone(&self.index),
            data: ::core::clone::Clone::clone(&self.data),
        }
    }
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for ForcedFullChunk {}
#[automatically_derived]
impl ::core::cmp::PartialEq for ForcedFullChunk {
    #[inline]
    fn eq(&self, other: &ForcedFullChunk) -> bool {
        self.index == other.index && self.data == other.data
    }
}
#[automatically_derived]
impl ::core::cmp::Eq for ForcedFullChunk {
    #[inline]
    #[doc(hidden)]
    #[coverage(off)]
    fn assert_fields_are_eq(&self) {
        let _: ::core::cmp::AssertParamIsEq<usize>;
        let _: ::core::cmp::AssertParamIsEq<Vec<u8>>;
    }
}
#[automatically_derived]
impl ::core::cmp::PartialOrd for ForcedFullChunk {
    #[inline]
    fn partial_cmp(
        &self,
        other: &ForcedFullChunk,
    ) -> ::core::option::Option<::core::cmp::Ordering> {
        match ::core::cmp::PartialOrd::partial_cmp(&self.index, &other.index) {
            ::core::option::Option::Some(::core::cmp::Ordering::Equal) => {
                ::core::cmp::PartialOrd::partial_cmp(&self.data, &other.data)
            }
            cmp => cmp,
        }
    }
}
#[automatically_derived]
impl ::core::cmp::Ord for ForcedFullChunk {
    #[inline]
    fn cmp(&self, other: &ForcedFullChunk) -> ::core::cmp::Ordering {
        match ::core::cmp::Ord::cmp(&self.index, &other.index) {
            ::core::cmp::Ordering::Equal => {
                ::core::cmp::Ord::cmp(&self.data, &other.data)
            }
            cmp => cmp,
        }
    }
}
#[automatically_derived]
///An archived [`ForcedFullChunk`]
#[bytecheck(crate = ::rkyv::bytecheck)]
#[repr(C)]
pub struct ArchivedForcedFullChunk
where
    usize: ::rkyv::Archive,
    Vec<u8>: ::rkyv::Archive,
{
    ///The archived counterpart of [`ForcedFullChunk::index`]
    pub index: <usize as ::rkyv::Archive>::Archived,
    ///The archived counterpart of [`ForcedFullChunk::data`]
    pub data: <Vec<u8> as ::rkyv::Archive>::Archived,
}
#[automatically_derived]
unsafe impl<
    __C: ::rkyv::bytecheck::rancor::Fallible + ?::core::marker::Sized,
> ::rkyv::bytecheck::CheckBytes<__C> for ArchivedForcedFullChunk
where
    usize: ::rkyv::Archive,
    Vec<u8>: ::rkyv::Archive,
    <__C as ::rkyv::bytecheck::rancor::Fallible>::Error: ::rkyv::bytecheck::rancor::Trace,
    <usize as ::rkyv::Archive>::Archived: ::rkyv::bytecheck::CheckBytes<__C>,
    <Vec<u8> as ::rkyv::Archive>::Archived: ::rkyv::bytecheck::CheckBytes<__C>,
{
    unsafe fn check_bytes(
        value: *const Self,
        context: &mut __C,
    ) -> ::core::result::Result<
        (),
        <__C as ::rkyv::bytecheck::rancor::Fallible>::Error,
    > {
        <<usize as ::rkyv::Archive>::Archived as ::rkyv::bytecheck::CheckBytes<
            __C,
        >>::check_bytes(&raw const (*value).index, context)
            .map_err(|e| {
                <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Trace>::trace(
                    e,
                    ::rkyv::bytecheck::StructCheckContext {
                        struct_name: "ArchivedForcedFullChunk",
                        field_name: "index",
                    },
                )
            })?;
        <<Vec<
            u8,
        > as ::rkyv::Archive>::Archived as ::rkyv::bytecheck::CheckBytes<
            __C,
        >>::check_bytes(&raw const (*value).data, context)
            .map_err(|e| {
                <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Trace>::trace(
                    e,
                    ::rkyv::bytecheck::StructCheckContext {
                        struct_name: "ArchivedForcedFullChunk",
                        field_name: "data",
                    },
                )
            })?;
        ::core::result::Result::Ok(())
    }
}
#[automatically_derived]
///The resolver for an archived [`ForcedFullChunk`]
pub struct ForcedFullChunkResolver
where
    usize: ::rkyv::Archive,
    Vec<u8>: ::rkyv::Archive,
{
    index: <usize as ::rkyv::Archive>::Resolver,
    data: <Vec<u8> as ::rkyv::Archive>::Resolver,
}
impl ::rkyv::Archive for ForcedFullChunk
where
    usize: ::rkyv::Archive,
    Vec<u8>: ::rkyv::Archive,
{
    type Archived = ArchivedForcedFullChunk;
    type Resolver = ForcedFullChunkResolver;
    const COPY_OPTIMIZATION: ::rkyv::traits::CopyOptimization<Self> = unsafe {
        ::rkyv::traits::CopyOptimization::enable_if(
            0 + ::core::mem::size_of::<usize>() + ::core::mem::size_of::<Vec<u8>>()
                == ::core::mem::size_of::<ForcedFullChunk>()
                && <usize as ::rkyv::Archive>::COPY_OPTIMIZATION.is_enabled()
                && const { builtin # offset_of(ForcedFullChunk, index) }
                    == const { builtin # offset_of(ArchivedForcedFullChunk, index) }
                && <Vec<u8> as ::rkyv::Archive>::COPY_OPTIMIZATION.is_enabled()
                && const { builtin # offset_of(ForcedFullChunk, data) }
                    == const { builtin # offset_of(ArchivedForcedFullChunk, data) },
        )
    };
    #[allow(clippy::unit_arg)]
    fn resolve(&self, resolver: Self::Resolver, out: ::rkyv::Place<Self::Archived>) {
        let field_ptr = unsafe { &raw mut (*out.ptr()).index };
        let field_out = unsafe { ::rkyv::Place::from_field_unchecked(out, field_ptr) };
        <usize as ::rkyv::Archive>::resolve(&self.index, resolver.index, field_out);
        let field_ptr = unsafe { &raw mut (*out.ptr()).data };
        let field_out = unsafe { ::rkyv::Place::from_field_unchecked(out, field_ptr) };
        <Vec<u8> as ::rkyv::Archive>::resolve(&self.data, resolver.data, field_out);
    }
}
unsafe impl ::rkyv::traits::Portable for ArchivedForcedFullChunk
where
    usize: ::rkyv::Archive,
    Vec<u8>: ::rkyv::Archive,
    <usize as ::rkyv::Archive>::Archived: ::rkyv::traits::Portable,
    <Vec<u8> as ::rkyv::Archive>::Archived: ::rkyv::traits::Portable,
{}
#[automatically_derived]
impl<__S: ::rkyv::rancor::Fallible + ?Sized> ::rkyv::Serialize<__S> for ForcedFullChunk
where
    usize: ::rkyv::Serialize<__S>,
    Vec<u8>: ::rkyv::Serialize<__S>,
{
    fn serialize(
        &self,
        serializer: &mut __S,
    ) -> ::core::result::Result<
        <Self as ::rkyv::Archive>::Resolver,
        <__S as ::rkyv::rancor::Fallible>::Error,
    > {
        let __this = self;
        ::core::result::Result::Ok(ForcedFullChunkResolver {
            index: <usize as ::rkyv::Serialize<
                __S,
            >>::serialize(&__this.index, serializer)?,
            data: <Vec<
                u8,
            > as ::rkyv::Serialize<__S>>::serialize(&__this.data, serializer)?,
        })
    }
}
#[automatically_derived]
impl<__D: ::rkyv::rancor::Fallible + ?Sized> ::rkyv::Deserialize<ForcedFullChunk, __D>
for ::rkyv::Archived<ForcedFullChunk>
where
    usize: ::rkyv::Archive,
    <usize as ::rkyv::Archive>::Archived: ::rkyv::Deserialize<usize, __D>,
    Vec<u8>: ::rkyv::Archive,
    <Vec<u8> as ::rkyv::Archive>::Archived: ::rkyv::Deserialize<Vec<u8>, __D>,
{
    fn deserialize(
        &self,
        deserializer: &mut __D,
    ) -> ::core::result::Result<
        ForcedFullChunk,
        <__D as ::rkyv::rancor::Fallible>::Error,
    > {
        let __this = self;
        ::core::result::Result::Ok(ForcedFullChunk {
            index: <<usize as ::rkyv::Archive>::Archived as ::rkyv::Deserialize<
                usize,
                __D,
            >>::deserialize(&__this.index, deserializer)?,
            data: <<Vec<
                u8,
            > as ::rkyv::Archive>::Archived as ::rkyv::Deserialize<
                Vec<u8>,
                __D,
            >>::deserialize(&__this.data, deserializer)?,
        })
    }
}
impl ::rewrite::traits::structural::blob::BlobItemChunk for ForcedFullChunk {
    type Index = usize;
    fn get_index(&self) -> &Self::Index {
        &self.index
    }
}
pub enum ForcedFullChunkFill {
    Full(usize),
    Partial(usize),
    Corrupted(usize),
}
#[automatically_derived]
impl ::core::fmt::Debug for ForcedFullChunkFill {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match self {
            ForcedFullChunkFill::Full(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "Full", &__self_0)
            }
            ForcedFullChunkFill::Partial(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(
                    f,
                    "Partial",
                    &__self_0,
                )
            }
            ForcedFullChunkFill::Corrupted(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(
                    f,
                    "Corrupted",
                    &__self_0,
                )
            }
        }
    }
}
#[automatically_derived]
#[doc(hidden)]
unsafe impl ::core::clone::TrivialClone for ForcedFullChunkFill {}
#[automatically_derived]
impl ::core::clone::Clone for ForcedFullChunkFill {
    #[inline]
    fn clone(&self) -> ForcedFullChunkFill {
        let _: ::core::clone::AssertParamIsClone<usize>;
        *self
    }
}
#[automatically_derived]
impl ::core::marker::Copy for ForcedFullChunkFill {}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for ForcedFullChunkFill {}
#[automatically_derived]
impl ::core::cmp::PartialEq for ForcedFullChunkFill {
    #[inline]
    fn eq(&self, other: &ForcedFullChunkFill) -> bool {
        let __self_discr = ::core::intrinsics::discriminant_value(self);
        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
        __self_discr == __arg1_discr
            && match (self, other) {
                (
                    ForcedFullChunkFill::Full(__self_0),
                    ForcedFullChunkFill::Full(__arg1_0),
                ) => __self_0 == __arg1_0,
                (
                    ForcedFullChunkFill::Partial(__self_0),
                    ForcedFullChunkFill::Partial(__arg1_0),
                ) => __self_0 == __arg1_0,
                (
                    ForcedFullChunkFill::Corrupted(__self_0),
                    ForcedFullChunkFill::Corrupted(__arg1_0),
                ) => __self_0 == __arg1_0,
                _ => unsafe { ::core::intrinsics::unreachable() }
            }
    }
}
#[automatically_derived]
impl ::core::cmp::Eq for ForcedFullChunkFill {
    #[inline]
    #[doc(hidden)]
    #[coverage(off)]
    fn assert_fields_are_eq(&self) {
        let _: ::core::cmp::AssertParamIsEq<usize>;
    }
}
#[automatically_derived]
impl ::core::cmp::PartialOrd for ForcedFullChunkFill {
    #[inline]
    fn partial_cmp(
        &self,
        other: &ForcedFullChunkFill,
    ) -> ::core::option::Option<::core::cmp::Ordering> {
        let __self_discr = ::core::intrinsics::discriminant_value(self);
        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
        match (self, other) {
            (
                ForcedFullChunkFill::Full(__self_0),
                ForcedFullChunkFill::Full(__arg1_0),
            ) => ::core::cmp::PartialOrd::partial_cmp(__self_0, __arg1_0),
            (
                ForcedFullChunkFill::Partial(__self_0),
                ForcedFullChunkFill::Partial(__arg1_0),
            ) => ::core::cmp::PartialOrd::partial_cmp(__self_0, __arg1_0),
            (
                ForcedFullChunkFill::Corrupted(__self_0),
                ForcedFullChunkFill::Corrupted(__arg1_0),
            ) => ::core::cmp::PartialOrd::partial_cmp(__self_0, __arg1_0),
            _ => ::core::cmp::PartialOrd::partial_cmp(&__self_discr, &__arg1_discr),
        }
    }
}
#[automatically_derived]
impl ::core::cmp::Ord for ForcedFullChunkFill {
    #[inline]
    fn cmp(&self, other: &ForcedFullChunkFill) -> ::core::cmp::Ordering {
        let __self_discr = ::core::intrinsics::discriminant_value(self);
        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
        match ::core::cmp::Ord::cmp(&__self_discr, &__arg1_discr) {
            ::core::cmp::Ordering::Equal => {
                match (self, other) {
                    (
                        ForcedFullChunkFill::Full(__self_0),
                        ForcedFullChunkFill::Full(__arg1_0),
                    ) => ::core::cmp::Ord::cmp(__self_0, __arg1_0),
                    (
                        ForcedFullChunkFill::Partial(__self_0),
                        ForcedFullChunkFill::Partial(__arg1_0),
                    ) => ::core::cmp::Ord::cmp(__self_0, __arg1_0),
                    (
                        ForcedFullChunkFill::Corrupted(__self_0),
                        ForcedFullChunkFill::Corrupted(__arg1_0),
                    ) => ::core::cmp::Ord::cmp(__self_0, __arg1_0),
                    _ => unsafe { ::core::intrinsics::unreachable() }
                }
            }
            cmp => cmp,
        }
    }
}
#[automatically_derived]
///An archived [`ForcedFullChunkFill`]
#[bytecheck(crate = ::rkyv::bytecheck)]
#[repr(u8)]
pub enum ArchivedForcedFullChunkFill
where
    usize: ::rkyv::Archive,
    usize: ::rkyv::Archive,
    usize: ::rkyv::Archive,
{
    ///The archived counterpart of [`ForcedFullChunkFill::Full`]
    #[allow(dead_code)]
    Full(
        ///The archived counterpart of [`ForcedFullChunkFill::Full::0`]
        <usize as ::rkyv::Archive>::Archived,
    ),
    ///The archived counterpart of [`ForcedFullChunkFill::Partial`]
    #[allow(dead_code)]
    Partial(
        ///The archived counterpart of [`ForcedFullChunkFill::Partial::0`]
        <usize as ::rkyv::Archive>::Archived,
    ),
    ///The archived counterpart of [`ForcedFullChunkFill::Corrupted`]
    #[allow(dead_code)]
    Corrupted(
        ///The archived counterpart of [`ForcedFullChunkFill::Corrupted::0`]
        <usize as ::rkyv::Archive>::Archived,
    ),
}
const _: () = {
    #[repr(u8)]
    enum Tag {
        Full,
        Partial,
        Corrupted,
    }
    struct Discriminant;
    #[automatically_derived]
    impl Discriminant {
        #[allow(non_upper_case_globals)]
        const Full: u8 = Tag::Full as u8;
        #[allow(non_upper_case_globals)]
        const Partial: u8 = Tag::Partial as u8;
        #[allow(non_upper_case_globals)]
        const Corrupted: u8 = Tag::Corrupted as u8;
    }
    #[repr(C)]
    struct VariantFull(
        Tag,
        <usize as ::rkyv::Archive>::Archived,
        ::core::marker::PhantomData<ArchivedForcedFullChunkFill>,
    )
    where
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive;
    #[repr(C)]
    struct VariantPartial(
        Tag,
        <usize as ::rkyv::Archive>::Archived,
        ::core::marker::PhantomData<ArchivedForcedFullChunkFill>,
    )
    where
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive;
    #[repr(C)]
    struct VariantCorrupted(
        Tag,
        <usize as ::rkyv::Archive>::Archived,
        ::core::marker::PhantomData<ArchivedForcedFullChunkFill>,
    )
    where
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive;
    #[automatically_derived]
    unsafe impl<
        __C: ::rkyv::bytecheck::rancor::Fallible + ?::core::marker::Sized,
    > ::rkyv::bytecheck::CheckBytes<__C> for ArchivedForcedFullChunkFill
    where
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive,
        <__C as ::rkyv::bytecheck::rancor::Fallible>::Error: ::rkyv::bytecheck::rancor::Source,
        <usize as ::rkyv::Archive>::Archived: ::rkyv::bytecheck::CheckBytes<__C>,
        <usize as ::rkyv::Archive>::Archived: ::rkyv::bytecheck::CheckBytes<__C>,
        <usize as ::rkyv::Archive>::Archived: ::rkyv::bytecheck::CheckBytes<__C>,
    {
        unsafe fn check_bytes(
            value: *const Self,
            context: &mut __C,
        ) -> ::core::result::Result<
            (),
            <__C as ::rkyv::bytecheck::rancor::Fallible>::Error,
        > {
            let tag = *value.cast::<u8>();
            match tag {
                Discriminant::Full => {
                    let value = value.cast::<VariantFull>();
                    <<usize as ::rkyv::Archive>::Archived as ::rkyv::bytecheck::CheckBytes<
                        __C,
                    >>::check_bytes(&raw const (*value).1, context)
                        .map_err(|e| {
                            <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Trace>::trace(
                                e,
                                ::rkyv::bytecheck::UnnamedEnumVariantCheckContext {
                                    enum_name: "ArchivedForcedFullChunkFill",
                                    variant_name: "Full",
                                    field_index: 1,
                                },
                            )
                        })?;
                }
                Discriminant::Partial => {
                    let value = value.cast::<VariantPartial>();
                    <<usize as ::rkyv::Archive>::Archived as ::rkyv::bytecheck::CheckBytes<
                        __C,
                    >>::check_bytes(&raw const (*value).1, context)
                        .map_err(|e| {
                            <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Trace>::trace(
                                e,
                                ::rkyv::bytecheck::UnnamedEnumVariantCheckContext {
                                    enum_name: "ArchivedForcedFullChunkFill",
                                    variant_name: "Partial",
                                    field_index: 1,
                                },
                            )
                        })?;
                }
                Discriminant::Corrupted => {
                    let value = value.cast::<VariantCorrupted>();
                    <<usize as ::rkyv::Archive>::Archived as ::rkyv::bytecheck::CheckBytes<
                        __C,
                    >>::check_bytes(&raw const (*value).1, context)
                        .map_err(|e| {
                            <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Trace>::trace(
                                e,
                                ::rkyv::bytecheck::UnnamedEnumVariantCheckContext {
                                    enum_name: "ArchivedForcedFullChunkFill",
                                    variant_name: "Corrupted",
                                    field_index: 1,
                                },
                            )
                        })?;
                }
                _ => {
                    return ::core::result::Result::Err(
                        <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Source>::new(::rkyv::bytecheck::InvalidEnumDiscriminantError {
                            enum_name: "ArchivedForcedFullChunkFill",
                            invalid_discriminant: tag,
                        }),
                    );
                }
            }
            ::core::result::Result::Ok(())
        }
    }
};
#[automatically_derived]
///The resolver for an archived [`ForcedFullChunkFill`]
pub enum ForcedFullChunkFillResolver
where
    usize: ::rkyv::Archive,
    usize: ::rkyv::Archive,
    usize: ::rkyv::Archive,
{
    ///The resolver for [`ForcedFullChunkFill::Full`]
    #[allow(dead_code)]
    Full(<usize as ::rkyv::Archive>::Resolver),
    ///The resolver for [`ForcedFullChunkFill::Partial`]
    #[allow(dead_code)]
    Partial(<usize as ::rkyv::Archive>::Resolver),
    ///The resolver for [`ForcedFullChunkFill::Corrupted`]
    #[allow(dead_code)]
    Corrupted(<usize as ::rkyv::Archive>::Resolver),
}
const _: () = {
    #[repr(u8)]
    enum ArchivedTag {
        Full,
        Partial,
        Corrupted,
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for ArchivedTag {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for ArchivedTag {
        #[inline]
        fn eq(&self, other: &ArchivedTag) -> bool {
            let __self_discr = ::core::intrinsics::discriminant_value(self);
            let __arg1_discr = ::core::intrinsics::discriminant_value(other);
            __self_discr == __arg1_discr
        }
    }
    #[automatically_derived]
    impl ::core::cmp::PartialOrd for ArchivedTag {
        #[inline]
        fn partial_cmp(
            &self,
            other: &ArchivedTag,
        ) -> ::core::option::Option<::core::cmp::Ordering> {
            let __self_discr = ::core::intrinsics::discriminant_value(self);
            let __arg1_discr = ::core::intrinsics::discriminant_value(other);
            ::core::cmp::PartialOrd::partial_cmp(&__self_discr, &__arg1_discr)
        }
    }
    #[repr(C)]
    struct ArchivedVariantFull(
        ArchivedTag,
        <usize as ::rkyv::Archive>::Archived,
        ::core::marker::PhantomData<ForcedFullChunkFill>,
    )
    where
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive;
    #[repr(C)]
    struct ArchivedVariantPartial(
        ArchivedTag,
        <usize as ::rkyv::Archive>::Archived,
        ::core::marker::PhantomData<ForcedFullChunkFill>,
    )
    where
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive;
    #[repr(C)]
    struct ArchivedVariantCorrupted(
        ArchivedTag,
        <usize as ::rkyv::Archive>::Archived,
        ::core::marker::PhantomData<ForcedFullChunkFill>,
    )
    where
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive;
    impl ::rkyv::Archive for ForcedFullChunkFill
    where
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive,
    {
        type Archived = ArchivedForcedFullChunkFill;
        type Resolver = ForcedFullChunkFillResolver;
        #[allow(clippy::unit_arg)]
        fn resolve(
            &self,
            resolver: <Self as ::rkyv::Archive>::Resolver,
            out: ::rkyv::Place<<Self as ::rkyv::Archive>::Archived>,
        ) {
            let __this = self;
            match resolver {
                ForcedFullChunkFillResolver::Full(resolver_0) => {
                    match __this {
                        ForcedFullChunkFill::Full(self_0, ..) => {
                            let out = unsafe {
                                out.cast_unchecked::<ArchivedVariantFull>()
                            };
                            let tag_ptr = unsafe { &raw mut (*out.ptr()).0 };
                            unsafe {
                                tag_ptr.write(ArchivedTag::Full);
                            }
                            let field_ptr = unsafe { &raw mut (*out.ptr()).1 };
                            let out_field = unsafe {
                                ::rkyv::Place::from_field_unchecked(out, field_ptr)
                            };
                            <usize as ::rkyv::Archive>::resolve(
                                self_0,
                                resolver_0,
                                out_field,
                            );
                        }
                        #[allow(unreachable_patterns)]
                        _ => unsafe { ::core::hint::unreachable_unchecked() }
                    }
                }
                ForcedFullChunkFillResolver::Partial(resolver_0) => {
                    match __this {
                        ForcedFullChunkFill::Partial(self_0, ..) => {
                            let out = unsafe {
                                out.cast_unchecked::<ArchivedVariantPartial>()
                            };
                            let tag_ptr = unsafe { &raw mut (*out.ptr()).0 };
                            unsafe {
                                tag_ptr.write(ArchivedTag::Partial);
                            }
                            let field_ptr = unsafe { &raw mut (*out.ptr()).1 };
                            let out_field = unsafe {
                                ::rkyv::Place::from_field_unchecked(out, field_ptr)
                            };
                            <usize as ::rkyv::Archive>::resolve(
                                self_0,
                                resolver_0,
                                out_field,
                            );
                        }
                        #[allow(unreachable_patterns)]
                        _ => unsafe { ::core::hint::unreachable_unchecked() }
                    }
                }
                ForcedFullChunkFillResolver::Corrupted(resolver_0) => {
                    match __this {
                        ForcedFullChunkFill::Corrupted(self_0, ..) => {
                            let out = unsafe {
                                out.cast_unchecked::<ArchivedVariantCorrupted>()
                            };
                            let tag_ptr = unsafe { &raw mut (*out.ptr()).0 };
                            unsafe {
                                tag_ptr.write(ArchivedTag::Corrupted);
                            }
                            let field_ptr = unsafe { &raw mut (*out.ptr()).1 };
                            let out_field = unsafe {
                                ::rkyv::Place::from_field_unchecked(out, field_ptr)
                            };
                            <usize as ::rkyv::Archive>::resolve(
                                self_0,
                                resolver_0,
                                out_field,
                            );
                        }
                        #[allow(unreachable_patterns)]
                        _ => unsafe { ::core::hint::unreachable_unchecked() }
                    }
                }
            }
        }
    }
};
unsafe impl ::rkyv::traits::Portable for ArchivedForcedFullChunkFill
where
    usize: ::rkyv::Archive,
    usize: ::rkyv::Archive,
    usize: ::rkyv::Archive,
    <usize as ::rkyv::Archive>::Archived: ::rkyv::traits::Portable,
    <usize as ::rkyv::Archive>::Archived: ::rkyv::traits::Portable,
    <usize as ::rkyv::Archive>::Archived: ::rkyv::traits::Portable,
{}
#[automatically_derived]
impl<__S: ::rkyv::rancor::Fallible + ?Sized> ::rkyv::Serialize<__S>
for ForcedFullChunkFill
where
    usize: ::rkyv::Serialize<__S>,
    usize: ::rkyv::Serialize<__S>,
    usize: ::rkyv::Serialize<__S>,
{
    fn serialize(
        &self,
        serializer: &mut __S,
    ) -> ::core::result::Result<
        <Self as ::rkyv::Archive>::Resolver,
        <__S as ::rkyv::rancor::Fallible>::Error,
    > {
        let __this = self;
        ::core::result::Result::Ok(
            match __this {
                ForcedFullChunkFill::Full(_0, ..) => {
                    ForcedFullChunkFillResolver::Full(
                        <usize as ::rkyv::Serialize<__S>>::serialize(_0, serializer)?,
                    )
                }
                ForcedFullChunkFill::Partial(_0, ..) => {
                    ForcedFullChunkFillResolver::Partial(
                        <usize as ::rkyv::Serialize<__S>>::serialize(_0, serializer)?,
                    )
                }
                ForcedFullChunkFill::Corrupted(_0, ..) => {
                    ForcedFullChunkFillResolver::Corrupted(
                        <usize as ::rkyv::Serialize<__S>>::serialize(_0, serializer)?,
                    )
                }
            },
        )
    }
}
#[automatically_derived]
impl<
    __D: ::rkyv::rancor::Fallible + ?Sized,
> ::rkyv::Deserialize<ForcedFullChunkFill, __D> for ::rkyv::Archived<ForcedFullChunkFill>
where
    usize: ::rkyv::Archive,
    <usize as ::rkyv::Archive>::Archived: ::rkyv::Deserialize<usize, __D>,
    usize: ::rkyv::Archive,
    <usize as ::rkyv::Archive>::Archived: ::rkyv::Deserialize<usize, __D>,
    usize: ::rkyv::Archive,
    <usize as ::rkyv::Archive>::Archived: ::rkyv::Deserialize<usize, __D>,
{
    fn deserialize(
        &self,
        deserializer: &mut __D,
    ) -> ::core::result::Result<
        ForcedFullChunkFill,
        <__D as ::rkyv::rancor::Fallible>::Error,
    > {
        let __this = self;
        ::core::result::Result::Ok(
            match __this {
                Self::Full(_0, ..) => {
                    ForcedFullChunkFill::Full(
                        <<usize as ::rkyv::Archive>::Archived as ::rkyv::Deserialize<
                            usize,
                            __D,
                        >>::deserialize(_0, deserializer)?,
                    )
                }
                Self::Partial(_0, ..) => {
                    ForcedFullChunkFill::Partial(
                        <<usize as ::rkyv::Archive>::Archived as ::rkyv::Deserialize<
                            usize,
                            __D,
                        >>::deserialize(_0, deserializer)?,
                    )
                }
                Self::Corrupted(_0, ..) => {
                    ForcedFullChunkFill::Corrupted(
                        <<usize as ::rkyv::Archive>::Archived as ::rkyv::Deserialize<
                            usize,
                            __D,
                        >>::deserialize(_0, deserializer)?,
                    )
                }
            },
        )
    }
}
impl ForcedFullChunkFill {
    pub fn from_size(actual: usize, expected: usize) -> Self {
        if actual == expected {
            Self::Full(actual)
        } else if actual < expected {
            Self::Partial(actual)
        } else {
            Self::Corrupted(actual)
        }
    }
}
impl rewrite::traits::structural::blob::NetabaseBlobItem for ForcedFull {
    type Chunk = ForcedFullChunk;
    type BlobIter = std::vec::IntoIter<rewrite::results::NetabaseResult<Self::Chunk>>;
    const DEFAULT_CHUNK_SIZE: usize = 0usize;
    fn into_chunks(
        self,
        size: rewrite::traits::structural::blob::ChunkSize,
    ) -> Box<dyn Iterator<Item = Self::Chunk>> {
        Box::new(self.into_chunks_iter(size).filter_map(|r| r.ok()))
    }
    fn into_chunks_iter(
        self,
        size: rewrite::traits::structural::blob::ChunkSize,
    ) -> Self::BlobIter {
        let serialized_data: rewrite::results::NetabaseResult<Vec<u8>> = (|| -> rewrite::results::NetabaseResult<
            Vec<u8>,
        > {
            Ok(
                rkyv::to_bytes::<rkyv::rancor::Error>(&self)
                    .map_err(|e| rewrite::results::NetabaseError::Serialization(
                        ::alloc::__export::must_use({
                            ::alloc::fmt::format(
                                format_args!("rkyv serialization failed: {0:?}", e),
                            )
                        }),
                    ))?
                    .to_vec(),
            )
        })();
        let chunk_size = match size {
            rewrite::traits::structural::blob::ChunkSize::Default => {
                if Self::DEFAULT_CHUNK_SIZE > 0 {
                    Self::DEFAULT_CHUNK_SIZE
                } else {
                    1024
                }
            }
            rewrite::traits::structural::blob::ChunkSize::Size(n) => n,
        };
        match serialized_data {
            Ok(data) => {
                data.chunks(chunk_size)
                    .enumerate()
                    .map(|(index, chunk_data)| {
                        Ok(Self::Chunk {
                            index,
                            data: chunk_data.to_vec(),
                        })
                    })
                    .collect::<Vec<_>>()
                    .into_iter()
            }
            Err(e) => {
                ::alloc::boxed::box_assume_init_into_vec_unsafe(
                        ::alloc::intrinsics::write_box_via_move(
                            ::alloc::boxed::Box::new_uninit(),
                            [Err(e)],
                        ),
                    )
                    .into_iter()
            }
        }
    }
    fn try_from_chunks(
        chunks: impl Iterator<Item = Self::Chunk>,
        size: rewrite::traits::structural::blob::ChunkSize,
    ) -> rewrite::results::NetabaseResult<Self> {
        let mut sorted_chunks: Vec<_> = chunks.collect();
        sorted_chunks.sort_by_key(|c| c.index);
        if sorted_chunks.is_empty() {
            return Err(
                rewrite::results::NetabaseError::BlobReconstruction(
                    rewrite::results::BlobReconstructionError::MissingChunks,
                ),
            );
        }
        let chunk_size = match size {
            rewrite::traits::structural::blob::ChunkSize::Default => {
                if Self::DEFAULT_CHUNK_SIZE > 0 {
                    Self::DEFAULT_CHUNK_SIZE
                } else {
                    1024
                }
            }
            rewrite::traits::structural::blob::ChunkSize::Size(n) => n,
        };
        let mut missing_details = Vec::new();
        let mut next_expected = 0;
        let max_idx = sorted_chunks.last().map(|c| c.index).unwrap_or(0);
        for chunk in &sorted_chunks {
            while chunk.index > next_expected {
                missing_details
                    .push(
                        ::alloc::__export::must_use({
                            ::alloc::fmt::format(
                                format_args!(
                                    "{0:?}({{ Index: {1}, Size: {2} }})",
                                    ForcedFullChunkFill::Full(chunk_size),
                                    next_expected,
                                    chunk_size,
                                ),
                            )
                        }),
                    );
                next_expected += 1;
            }
            let fill = ForcedFullChunkFill::from_size(chunk.data.len(), chunk_size);
            match fill {
                ForcedFullChunkFill::Corrupted(size) => {
                    return Err(
                        rewrite::results::NetabaseError::BlobReconstruction(
                            rewrite::results::BlobReconstructionError::InvalidChunkData(
                                ::alloc::__export::must_use({
                                    ::alloc::fmt::format(
                                        format_args!(
                                            "Corrupted chunk detected: {0:?}({{ Index: {1}, Size: {2} }}). Max allowed size is {3}.",
                                            fill,
                                            chunk.index,
                                            size,
                                            chunk_size,
                                        ),
                                    )
                                }),
                            ),
                        ),
                    );
                }
                ForcedFullChunkFill::Partial(size) if chunk.index < max_idx => {
                    return Err(
                        rewrite::results::NetabaseError::BlobReconstruction(
                            rewrite::results::BlobReconstructionError::InvalidChunkData(
                                ::alloc::__export::must_use({
                                    ::alloc::fmt::format(
                                        format_args!(
                                            "Unexpected partial chunk in middle of stream: {0:?}({{ Index: {1}, Size: {2} }}). Expected {3} bytes.",
                                            fill,
                                            chunk.index,
                                            size,
                                            chunk_size,
                                        ),
                                    )
                                }),
                            ),
                        ),
                    );
                }
                _ => {}
            }
            if chunk.index == next_expected {
                next_expected += 1;
            }
        }
        if !missing_details.is_empty() {
            if let Some(last) = sorted_chunks.last() {
                let fill = ForcedFullChunkFill::from_size(last.data.len(), chunk_size);
                if #[allow(non_exhaustive_omitted_patterns)]
                match fill {
                    ForcedFullChunkFill::Full(_) => true,
                    _ => false,
                } {
                    missing_details
                        .push(
                            ::alloc::__export::must_use({
                                ::alloc::fmt::format(
                                    format_args!(
                                        "... (Stream truncated: last chunk was Full, expected more data after Index {0})",
                                        last.index,
                                    ),
                                )
                            }),
                        );
                }
            }
        }
        if !missing_details.is_empty() {
            return Err(
                rewrite::results::NetabaseError::BlobReconstruction(
                    rewrite::results::BlobReconstructionError::InvalidChunkData(
                        ::alloc::__export::must_use({
                            ::alloc::fmt::format(
                                format_args!(
                                    "Missing chunks: [{0}]. Total chunks present: {1}",
                                    missing_details.join(", "),
                                    sorted_chunks.len(),
                                ),
                            )
                        }),
                    ),
                ),
            );
        }
        let serialized_data: Vec<u8> = sorted_chunks
            .into_iter()
            .flat_map(|c| c.data)
            .collect();
        Ok(
            rkyv::from_bytes::<Self, rkyv::rancor::Error>(&serialized_data)
                .map_err(|e| rewrite::results::NetabaseError::Serialization(
                    ::alloc::__export::must_use({
                        ::alloc::fmt::format(
                            format_args!("rkyv deserialization failed: {0:?}", e),
                        )
                    }),
                ))?,
        )
    }
    fn get_blob(&self) -> &Self::Chunk {
        {
            ::core::panicking::panic_fmt(
                format_args!(
                    "not implemented: {0}",
                    format_args!("get_blob() requires storing a chunk reference"),
                ),
            );
        }
    }
}
impl IntoIterator for ForcedFull {
    type Item = rewrite::results::NetabaseResult<ForcedFullChunk>;
    type IntoIter = std::vec::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        rewrite::traits::structural::blob::NetabaseBlobItem::into_chunks_iter(
            self,
            rewrite::traits::structural::blob::ChunkSize::Default,
        )
    }
}
#[blob(strategy = "partial")]
struct ForcedPartial {
    field1: String,
}
#[automatically_derived]
///An archived [`ForcedPartial`]
#[bytecheck(crate = ::rkyv::bytecheck)]
#[repr(C)]
struct ArchivedForcedPartial
where
    String: ::rkyv::Archive,
{
    ///The archived counterpart of [`ForcedPartial::field1`]
    field1: <String as ::rkyv::Archive>::Archived,
}
#[automatically_derived]
unsafe impl<
    __C: ::rkyv::bytecheck::rancor::Fallible + ?::core::marker::Sized,
> ::rkyv::bytecheck::CheckBytes<__C> for ArchivedForcedPartial
where
    String: ::rkyv::Archive,
    <__C as ::rkyv::bytecheck::rancor::Fallible>::Error: ::rkyv::bytecheck::rancor::Trace,
    <String as ::rkyv::Archive>::Archived: ::rkyv::bytecheck::CheckBytes<__C>,
{
    unsafe fn check_bytes(
        value: *const Self,
        context: &mut __C,
    ) -> ::core::result::Result<
        (),
        <__C as ::rkyv::bytecheck::rancor::Fallible>::Error,
    > {
        <<String as ::rkyv::Archive>::Archived as ::rkyv::bytecheck::CheckBytes<
            __C,
        >>::check_bytes(&raw const (*value).field1, context)
            .map_err(|e| {
                <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Trace>::trace(
                    e,
                    ::rkyv::bytecheck::StructCheckContext {
                        struct_name: "ArchivedForcedPartial",
                        field_name: "field1",
                    },
                )
            })?;
        ::core::result::Result::Ok(())
    }
}
#[automatically_derived]
///The resolver for an archived [`ForcedPartial`]
struct ForcedPartialResolver
where
    String: ::rkyv::Archive,
{
    field1: <String as ::rkyv::Archive>::Resolver,
}
impl ::rkyv::Archive for ForcedPartial
where
    String: ::rkyv::Archive,
{
    type Archived = ArchivedForcedPartial;
    type Resolver = ForcedPartialResolver;
    const COPY_OPTIMIZATION: ::rkyv::traits::CopyOptimization<Self> = unsafe {
        ::rkyv::traits::CopyOptimization::enable_if(
            0 + ::core::mem::size_of::<String>()
                == ::core::mem::size_of::<ForcedPartial>()
                && <String as ::rkyv::Archive>::COPY_OPTIMIZATION.is_enabled()
                && const { builtin # offset_of(ForcedPartial, field1) }
                    == const { builtin # offset_of(ArchivedForcedPartial, field1) },
        )
    };
    #[allow(clippy::unit_arg)]
    fn resolve(&self, resolver: Self::Resolver, out: ::rkyv::Place<Self::Archived>) {
        let field_ptr = unsafe { &raw mut (*out.ptr()).field1 };
        let field_out = unsafe { ::rkyv::Place::from_field_unchecked(out, field_ptr) };
        <String as ::rkyv::Archive>::resolve(&self.field1, resolver.field1, field_out);
    }
}
unsafe impl ::rkyv::traits::Portable for ArchivedForcedPartial
where
    String: ::rkyv::Archive,
    <String as ::rkyv::Archive>::Archived: ::rkyv::traits::Portable,
{}
#[automatically_derived]
impl<__S: ::rkyv::rancor::Fallible + ?Sized> ::rkyv::Serialize<__S> for ForcedPartial
where
    String: ::rkyv::Serialize<__S>,
{
    fn serialize(
        &self,
        serializer: &mut __S,
    ) -> ::core::result::Result<
        <Self as ::rkyv::Archive>::Resolver,
        <__S as ::rkyv::rancor::Fallible>::Error,
    > {
        let __this = self;
        ::core::result::Result::Ok(ForcedPartialResolver {
            field1: <String as ::rkyv::Serialize<
                __S,
            >>::serialize(&__this.field1, serializer)?,
        })
    }
}
#[automatically_derived]
impl<__D: ::rkyv::rancor::Fallible + ?Sized> ::rkyv::Deserialize<ForcedPartial, __D>
for ::rkyv::Archived<ForcedPartial>
where
    String: ::rkyv::Archive,
    <String as ::rkyv::Archive>::Archived: ::rkyv::Deserialize<String, __D>,
{
    fn deserialize(
        &self,
        deserializer: &mut __D,
    ) -> ::core::result::Result<
        ForcedPartial,
        <__D as ::rkyv::rancor::Fallible>::Error,
    > {
        let __this = self;
        ::core::result::Result::Ok(ForcedPartial {
            field1: <<String as ::rkyv::Archive>::Archived as ::rkyv::Deserialize<
                String,
                __D,
            >>::deserialize(&__this.field1, deserializer)?,
        })
    }
}
#[automatically_derived]
impl ::core::fmt::Debug for ForcedPartial {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::debug_struct_field1_finish(
            f,
            "ForcedPartial",
            "field1",
            &&self.field1,
        )
    }
}
#[automatically_derived]
impl ::core::clone::Clone for ForcedPartial {
    #[inline]
    fn clone(&self) -> ForcedPartial {
        ForcedPartial {
            field1: ::core::clone::Clone::clone(&self.field1),
        }
    }
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for ForcedPartial {}
#[automatically_derived]
impl ::core::cmp::PartialEq for ForcedPartial {
    #[inline]
    fn eq(&self, other: &ForcedPartial) -> bool {
        self.field1 == other.field1
    }
}
#[automatically_derived]
impl ::core::cmp::Eq for ForcedPartial {
    #[inline]
    #[doc(hidden)]
    #[coverage(off)]
    fn assert_fields_are_eq(&self) {
        let _: ::core::cmp::AssertParamIsEq<String>;
    }
}
pub struct ForcedPartialField1Chunk {
    pub index: usize,
    pub data: Vec<u8>,
}
#[automatically_derived]
impl ::core::fmt::Debug for ForcedPartialField1Chunk {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::debug_struct_field2_finish(
            f,
            "ForcedPartialField1Chunk",
            "index",
            &self.index,
            "data",
            &&self.data,
        )
    }
}
#[automatically_derived]
impl ::core::clone::Clone for ForcedPartialField1Chunk {
    #[inline]
    fn clone(&self) -> ForcedPartialField1Chunk {
        ForcedPartialField1Chunk {
            index: ::core::clone::Clone::clone(&self.index),
            data: ::core::clone::Clone::clone(&self.data),
        }
    }
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for ForcedPartialField1Chunk {}
#[automatically_derived]
impl ::core::cmp::PartialEq for ForcedPartialField1Chunk {
    #[inline]
    fn eq(&self, other: &ForcedPartialField1Chunk) -> bool {
        self.index == other.index && self.data == other.data
    }
}
#[automatically_derived]
impl ::core::cmp::Eq for ForcedPartialField1Chunk {
    #[inline]
    #[doc(hidden)]
    #[coverage(off)]
    fn assert_fields_are_eq(&self) {
        let _: ::core::cmp::AssertParamIsEq<usize>;
        let _: ::core::cmp::AssertParamIsEq<Vec<u8>>;
    }
}
#[automatically_derived]
impl ::core::cmp::PartialOrd for ForcedPartialField1Chunk {
    #[inline]
    fn partial_cmp(
        &self,
        other: &ForcedPartialField1Chunk,
    ) -> ::core::option::Option<::core::cmp::Ordering> {
        match ::core::cmp::PartialOrd::partial_cmp(&self.index, &other.index) {
            ::core::option::Option::Some(::core::cmp::Ordering::Equal) => {
                ::core::cmp::PartialOrd::partial_cmp(&self.data, &other.data)
            }
            cmp => cmp,
        }
    }
}
#[automatically_derived]
impl ::core::cmp::Ord for ForcedPartialField1Chunk {
    #[inline]
    fn cmp(&self, other: &ForcedPartialField1Chunk) -> ::core::cmp::Ordering {
        match ::core::cmp::Ord::cmp(&self.index, &other.index) {
            ::core::cmp::Ordering::Equal => {
                ::core::cmp::Ord::cmp(&self.data, &other.data)
            }
            cmp => cmp,
        }
    }
}
#[automatically_derived]
///An archived [`ForcedPartialField1Chunk`]
#[bytecheck(crate = ::rkyv::bytecheck)]
#[repr(C)]
pub struct ArchivedForcedPartialField1Chunk
where
    usize: ::rkyv::Archive,
    Vec<u8>: ::rkyv::Archive,
{
    ///The archived counterpart of [`ForcedPartialField1Chunk::index`]
    pub index: <usize as ::rkyv::Archive>::Archived,
    ///The archived counterpart of [`ForcedPartialField1Chunk::data`]
    pub data: <Vec<u8> as ::rkyv::Archive>::Archived,
}
#[automatically_derived]
unsafe impl<
    __C: ::rkyv::bytecheck::rancor::Fallible + ?::core::marker::Sized,
> ::rkyv::bytecheck::CheckBytes<__C> for ArchivedForcedPartialField1Chunk
where
    usize: ::rkyv::Archive,
    Vec<u8>: ::rkyv::Archive,
    <__C as ::rkyv::bytecheck::rancor::Fallible>::Error: ::rkyv::bytecheck::rancor::Trace,
    <usize as ::rkyv::Archive>::Archived: ::rkyv::bytecheck::CheckBytes<__C>,
    <Vec<u8> as ::rkyv::Archive>::Archived: ::rkyv::bytecheck::CheckBytes<__C>,
{
    unsafe fn check_bytes(
        value: *const Self,
        context: &mut __C,
    ) -> ::core::result::Result<
        (),
        <__C as ::rkyv::bytecheck::rancor::Fallible>::Error,
    > {
        <<usize as ::rkyv::Archive>::Archived as ::rkyv::bytecheck::CheckBytes<
            __C,
        >>::check_bytes(&raw const (*value).index, context)
            .map_err(|e| {
                <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Trace>::trace(
                    e,
                    ::rkyv::bytecheck::StructCheckContext {
                        struct_name: "ArchivedForcedPartialField1Chunk",
                        field_name: "index",
                    },
                )
            })?;
        <<Vec<
            u8,
        > as ::rkyv::Archive>::Archived as ::rkyv::bytecheck::CheckBytes<
            __C,
        >>::check_bytes(&raw const (*value).data, context)
            .map_err(|e| {
                <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Trace>::trace(
                    e,
                    ::rkyv::bytecheck::StructCheckContext {
                        struct_name: "ArchivedForcedPartialField1Chunk",
                        field_name: "data",
                    },
                )
            })?;
        ::core::result::Result::Ok(())
    }
}
#[automatically_derived]
///The resolver for an archived [`ForcedPartialField1Chunk`]
pub struct ForcedPartialField1ChunkResolver
where
    usize: ::rkyv::Archive,
    Vec<u8>: ::rkyv::Archive,
{
    index: <usize as ::rkyv::Archive>::Resolver,
    data: <Vec<u8> as ::rkyv::Archive>::Resolver,
}
impl ::rkyv::Archive for ForcedPartialField1Chunk
where
    usize: ::rkyv::Archive,
    Vec<u8>: ::rkyv::Archive,
{
    type Archived = ArchivedForcedPartialField1Chunk;
    type Resolver = ForcedPartialField1ChunkResolver;
    const COPY_OPTIMIZATION: ::rkyv::traits::CopyOptimization<Self> = unsafe {
        ::rkyv::traits::CopyOptimization::enable_if(
            0 + ::core::mem::size_of::<usize>() + ::core::mem::size_of::<Vec<u8>>()
                == ::core::mem::size_of::<ForcedPartialField1Chunk>()
                && <usize as ::rkyv::Archive>::COPY_OPTIMIZATION.is_enabled()
                && const { builtin # offset_of(ForcedPartialField1Chunk, index) }
                    == const {
                        builtin # offset_of(ArchivedForcedPartialField1Chunk, index)
                    } && <Vec<u8> as ::rkyv::Archive>::COPY_OPTIMIZATION.is_enabled()
                && const { builtin # offset_of(ForcedPartialField1Chunk, data) }
                    == const {
                        builtin # offset_of(ArchivedForcedPartialField1Chunk, data)
                    },
        )
    };
    #[allow(clippy::unit_arg)]
    fn resolve(&self, resolver: Self::Resolver, out: ::rkyv::Place<Self::Archived>) {
        let field_ptr = unsafe { &raw mut (*out.ptr()).index };
        let field_out = unsafe { ::rkyv::Place::from_field_unchecked(out, field_ptr) };
        <usize as ::rkyv::Archive>::resolve(&self.index, resolver.index, field_out);
        let field_ptr = unsafe { &raw mut (*out.ptr()).data };
        let field_out = unsafe { ::rkyv::Place::from_field_unchecked(out, field_ptr) };
        <Vec<u8> as ::rkyv::Archive>::resolve(&self.data, resolver.data, field_out);
    }
}
unsafe impl ::rkyv::traits::Portable for ArchivedForcedPartialField1Chunk
where
    usize: ::rkyv::Archive,
    Vec<u8>: ::rkyv::Archive,
    <usize as ::rkyv::Archive>::Archived: ::rkyv::traits::Portable,
    <Vec<u8> as ::rkyv::Archive>::Archived: ::rkyv::traits::Portable,
{}
#[automatically_derived]
impl<__S: ::rkyv::rancor::Fallible + ?Sized> ::rkyv::Serialize<__S>
for ForcedPartialField1Chunk
where
    usize: ::rkyv::Serialize<__S>,
    Vec<u8>: ::rkyv::Serialize<__S>,
{
    fn serialize(
        &self,
        serializer: &mut __S,
    ) -> ::core::result::Result<
        <Self as ::rkyv::Archive>::Resolver,
        <__S as ::rkyv::rancor::Fallible>::Error,
    > {
        let __this = self;
        ::core::result::Result::Ok(ForcedPartialField1ChunkResolver {
            index: <usize as ::rkyv::Serialize<
                __S,
            >>::serialize(&__this.index, serializer)?,
            data: <Vec<
                u8,
            > as ::rkyv::Serialize<__S>>::serialize(&__this.data, serializer)?,
        })
    }
}
#[automatically_derived]
impl<
    __D: ::rkyv::rancor::Fallible + ?Sized,
> ::rkyv::Deserialize<ForcedPartialField1Chunk, __D>
for ::rkyv::Archived<ForcedPartialField1Chunk>
where
    usize: ::rkyv::Archive,
    <usize as ::rkyv::Archive>::Archived: ::rkyv::Deserialize<usize, __D>,
    Vec<u8>: ::rkyv::Archive,
    <Vec<u8> as ::rkyv::Archive>::Archived: ::rkyv::Deserialize<Vec<u8>, __D>,
{
    fn deserialize(
        &self,
        deserializer: &mut __D,
    ) -> ::core::result::Result<
        ForcedPartialField1Chunk,
        <__D as ::rkyv::rancor::Fallible>::Error,
    > {
        let __this = self;
        ::core::result::Result::Ok(ForcedPartialField1Chunk {
            index: <<usize as ::rkyv::Archive>::Archived as ::rkyv::Deserialize<
                usize,
                __D,
            >>::deserialize(&__this.index, deserializer)?,
            data: <<Vec<
                u8,
            > as ::rkyv::Archive>::Archived as ::rkyv::Deserialize<
                Vec<u8>,
                __D,
            >>::deserialize(&__this.data, deserializer)?,
        })
    }
}
impl ::rewrite::traits::structural::blob::BlobItemChunk for ForcedPartialField1Chunk {
    type Index = usize;
    fn get_index(&self) -> &Self::Index {
        &self.index
    }
}
pub enum ForcedPartialChunk {
    Field1(ForcedPartialField1Chunk),
    Missing,
}
#[automatically_derived]
impl ::core::fmt::Debug for ForcedPartialChunk {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match self {
            ForcedPartialChunk::Field1(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "Field1", &__self_0)
            }
            ForcedPartialChunk::Missing => {
                ::core::fmt::Formatter::write_str(f, "Missing")
            }
        }
    }
}
#[automatically_derived]
impl ::core::clone::Clone for ForcedPartialChunk {
    #[inline]
    fn clone(&self) -> ForcedPartialChunk {
        match self {
            ForcedPartialChunk::Field1(__self_0) => {
                ForcedPartialChunk::Field1(::core::clone::Clone::clone(__self_0))
            }
            ForcedPartialChunk::Missing => ForcedPartialChunk::Missing,
        }
    }
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for ForcedPartialChunk {}
#[automatically_derived]
impl ::core::cmp::PartialEq for ForcedPartialChunk {
    #[inline]
    fn eq(&self, other: &ForcedPartialChunk) -> bool {
        let __self_discr = ::core::intrinsics::discriminant_value(self);
        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
        __self_discr == __arg1_discr
            && match (self, other) {
                (
                    ForcedPartialChunk::Field1(__self_0),
                    ForcedPartialChunk::Field1(__arg1_0),
                ) => __self_0 == __arg1_0,
                _ => true,
            }
    }
}
#[automatically_derived]
impl ::core::cmp::Eq for ForcedPartialChunk {
    #[inline]
    #[doc(hidden)]
    #[coverage(off)]
    fn assert_fields_are_eq(&self) {
        let _: ::core::cmp::AssertParamIsEq<ForcedPartialField1Chunk>;
    }
}
#[automatically_derived]
impl ::core::cmp::PartialOrd for ForcedPartialChunk {
    #[inline]
    fn partial_cmp(
        &self,
        other: &ForcedPartialChunk,
    ) -> ::core::option::Option<::core::cmp::Ordering> {
        let __self_discr = ::core::intrinsics::discriminant_value(self);
        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
        match (self, other) {
            (
                ForcedPartialChunk::Field1(__self_0),
                ForcedPartialChunk::Field1(__arg1_0),
            ) => ::core::cmp::PartialOrd::partial_cmp(__self_0, __arg1_0),
            _ => ::core::cmp::PartialOrd::partial_cmp(&__self_discr, &__arg1_discr),
        }
    }
}
#[automatically_derived]
impl ::core::cmp::Ord for ForcedPartialChunk {
    #[inline]
    fn cmp(&self, other: &ForcedPartialChunk) -> ::core::cmp::Ordering {
        let __self_discr = ::core::intrinsics::discriminant_value(self);
        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
        match ::core::cmp::Ord::cmp(&__self_discr, &__arg1_discr) {
            ::core::cmp::Ordering::Equal => {
                match (self, other) {
                    (
                        ForcedPartialChunk::Field1(__self_0),
                        ForcedPartialChunk::Field1(__arg1_0),
                    ) => ::core::cmp::Ord::cmp(__self_0, __arg1_0),
                    _ => ::core::cmp::Ordering::Equal,
                }
            }
            cmp => cmp,
        }
    }
}
#[automatically_derived]
///An archived [`ForcedPartialChunk`]
#[bytecheck(crate = ::rkyv::bytecheck)]
#[repr(u8)]
pub enum ArchivedForcedPartialChunk
where
    ForcedPartialField1Chunk: ::rkyv::Archive,
{
    ///The archived counterpart of [`ForcedPartialChunk::Field1`]
    #[allow(dead_code)]
    Field1(
        ///The archived counterpart of [`ForcedPartialChunk::Field1::0`]
        <ForcedPartialField1Chunk as ::rkyv::Archive>::Archived,
    ),
    ///The archived counterpart of [`ForcedPartialChunk::Missing`]
    #[allow(dead_code)]
    Missing,
}
const _: () = {
    #[repr(u8)]
    enum Tag {
        Field1,
        Missing,
    }
    struct Discriminant;
    #[automatically_derived]
    impl Discriminant {
        #[allow(non_upper_case_globals)]
        const Field1: u8 = Tag::Field1 as u8;
        #[allow(non_upper_case_globals)]
        const Missing: u8 = Tag::Missing as u8;
    }
    #[repr(C)]
    struct VariantField1(
        Tag,
        <ForcedPartialField1Chunk as ::rkyv::Archive>::Archived,
        ::core::marker::PhantomData<ArchivedForcedPartialChunk>,
    )
    where
        ForcedPartialField1Chunk: ::rkyv::Archive;
    #[automatically_derived]
    unsafe impl<
        __C: ::rkyv::bytecheck::rancor::Fallible + ?::core::marker::Sized,
    > ::rkyv::bytecheck::CheckBytes<__C> for ArchivedForcedPartialChunk
    where
        ForcedPartialField1Chunk: ::rkyv::Archive,
        <__C as ::rkyv::bytecheck::rancor::Fallible>::Error: ::rkyv::bytecheck::rancor::Source,
        <ForcedPartialField1Chunk as ::rkyv::Archive>::Archived: ::rkyv::bytecheck::CheckBytes<
            __C,
        >,
    {
        unsafe fn check_bytes(
            value: *const Self,
            context: &mut __C,
        ) -> ::core::result::Result<
            (),
            <__C as ::rkyv::bytecheck::rancor::Fallible>::Error,
        > {
            let tag = *value.cast::<u8>();
            match tag {
                Discriminant::Field1 => {
                    let value = value.cast::<VariantField1>();
                    <<ForcedPartialField1Chunk as ::rkyv::Archive>::Archived as ::rkyv::bytecheck::CheckBytes<
                        __C,
                    >>::check_bytes(&raw const (*value).1, context)
                        .map_err(|e| {
                            <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Trace>::trace(
                                e,
                                ::rkyv::bytecheck::UnnamedEnumVariantCheckContext {
                                    enum_name: "ArchivedForcedPartialChunk",
                                    variant_name: "Field1",
                                    field_index: 1,
                                },
                            )
                        })?;
                }
                Discriminant::Missing => {}
                _ => {
                    return ::core::result::Result::Err(
                        <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Source>::new(::rkyv::bytecheck::InvalidEnumDiscriminantError {
                            enum_name: "ArchivedForcedPartialChunk",
                            invalid_discriminant: tag,
                        }),
                    );
                }
            }
            ::core::result::Result::Ok(())
        }
    }
};
#[automatically_derived]
///The resolver for an archived [`ForcedPartialChunk`]
pub enum ForcedPartialChunkResolver
where
    ForcedPartialField1Chunk: ::rkyv::Archive,
{
    ///The resolver for [`ForcedPartialChunk::Field1`]
    #[allow(dead_code)]
    Field1(<ForcedPartialField1Chunk as ::rkyv::Archive>::Resolver),
    ///The resolver for [`ForcedPartialChunk::Missing`]
    #[allow(dead_code)]
    Missing,
}
const _: () = {
    #[repr(u8)]
    enum ArchivedTag {
        Field1,
        Missing,
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for ArchivedTag {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for ArchivedTag {
        #[inline]
        fn eq(&self, other: &ArchivedTag) -> bool {
            let __self_discr = ::core::intrinsics::discriminant_value(self);
            let __arg1_discr = ::core::intrinsics::discriminant_value(other);
            __self_discr == __arg1_discr
        }
    }
    #[automatically_derived]
    impl ::core::cmp::PartialOrd for ArchivedTag {
        #[inline]
        fn partial_cmp(
            &self,
            other: &ArchivedTag,
        ) -> ::core::option::Option<::core::cmp::Ordering> {
            let __self_discr = ::core::intrinsics::discriminant_value(self);
            let __arg1_discr = ::core::intrinsics::discriminant_value(other);
            ::core::cmp::PartialOrd::partial_cmp(&__self_discr, &__arg1_discr)
        }
    }
    #[repr(C)]
    struct ArchivedVariantField1(
        ArchivedTag,
        <ForcedPartialField1Chunk as ::rkyv::Archive>::Archived,
        ::core::marker::PhantomData<ForcedPartialChunk>,
    )
    where
        ForcedPartialField1Chunk: ::rkyv::Archive;
    impl ::rkyv::Archive for ForcedPartialChunk
    where
        ForcedPartialField1Chunk: ::rkyv::Archive,
    {
        type Archived = ArchivedForcedPartialChunk;
        type Resolver = ForcedPartialChunkResolver;
        #[allow(clippy::unit_arg)]
        fn resolve(
            &self,
            resolver: <Self as ::rkyv::Archive>::Resolver,
            out: ::rkyv::Place<<Self as ::rkyv::Archive>::Archived>,
        ) {
            let __this = self;
            match resolver {
                ForcedPartialChunkResolver::Field1(resolver_0) => {
                    match __this {
                        ForcedPartialChunk::Field1(self_0, ..) => {
                            let out = unsafe {
                                out.cast_unchecked::<ArchivedVariantField1>()
                            };
                            let tag_ptr = unsafe { &raw mut (*out.ptr()).0 };
                            unsafe {
                                tag_ptr.write(ArchivedTag::Field1);
                            }
                            let field_ptr = unsafe { &raw mut (*out.ptr()).1 };
                            let out_field = unsafe {
                                ::rkyv::Place::from_field_unchecked(out, field_ptr)
                            };
                            <ForcedPartialField1Chunk as ::rkyv::Archive>::resolve(
                                self_0,
                                resolver_0,
                                out_field,
                            );
                        }
                        #[allow(unreachable_patterns)]
                        _ => unsafe { ::core::hint::unreachable_unchecked() }
                    }
                }
                ForcedPartialChunkResolver::Missing => {
                    let out = unsafe { out.cast_unchecked::<ArchivedTag>() };
                    unsafe {
                        out.write_unchecked(ArchivedTag::Missing);
                    }
                }
            }
        }
    }
};
unsafe impl ::rkyv::traits::Portable for ArchivedForcedPartialChunk
where
    ForcedPartialField1Chunk: ::rkyv::Archive,
    <ForcedPartialField1Chunk as ::rkyv::Archive>::Archived: ::rkyv::traits::Portable,
{}
#[automatically_derived]
impl<__S: ::rkyv::rancor::Fallible + ?Sized> ::rkyv::Serialize<__S>
for ForcedPartialChunk
where
    ForcedPartialField1Chunk: ::rkyv::Serialize<__S>,
{
    fn serialize(
        &self,
        serializer: &mut __S,
    ) -> ::core::result::Result<
        <Self as ::rkyv::Archive>::Resolver,
        <__S as ::rkyv::rancor::Fallible>::Error,
    > {
        let __this = self;
        ::core::result::Result::Ok(
            match __this {
                ForcedPartialChunk::Field1(_0, ..) => {
                    ForcedPartialChunkResolver::Field1(
                        <ForcedPartialField1Chunk as ::rkyv::Serialize<
                            __S,
                        >>::serialize(_0, serializer)?,
                    )
                }
                ForcedPartialChunk::Missing => ForcedPartialChunkResolver::Missing,
            },
        )
    }
}
#[automatically_derived]
impl<__D: ::rkyv::rancor::Fallible + ?Sized> ::rkyv::Deserialize<ForcedPartialChunk, __D>
for ::rkyv::Archived<ForcedPartialChunk>
where
    ForcedPartialField1Chunk: ::rkyv::Archive,
    <ForcedPartialField1Chunk as ::rkyv::Archive>::Archived: ::rkyv::Deserialize<
        ForcedPartialField1Chunk,
        __D,
    >,
{
    fn deserialize(
        &self,
        deserializer: &mut __D,
    ) -> ::core::result::Result<
        ForcedPartialChunk,
        <__D as ::rkyv::rancor::Fallible>::Error,
    > {
        let __this = self;
        ::core::result::Result::Ok(
            match __this {
                Self::Field1(_0, ..) => {
                    ForcedPartialChunk::Field1(
                        <<ForcedPartialField1Chunk as ::rkyv::Archive>::Archived as ::rkyv::Deserialize<
                            ForcedPartialField1Chunk,
                            __D,
                        >>::deserialize(_0, deserializer)?,
                    )
                }
                Self::Missing => ForcedPartialChunk::Missing,
            },
        )
    }
}
pub enum ForcedPartialChunkFill {
    Full(usize),
    Partial(usize),
    Corrupted(usize),
}
#[automatically_derived]
impl ::core::fmt::Debug for ForcedPartialChunkFill {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match self {
            ForcedPartialChunkFill::Full(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "Full", &__self_0)
            }
            ForcedPartialChunkFill::Partial(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(
                    f,
                    "Partial",
                    &__self_0,
                )
            }
            ForcedPartialChunkFill::Corrupted(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(
                    f,
                    "Corrupted",
                    &__self_0,
                )
            }
        }
    }
}
#[automatically_derived]
#[doc(hidden)]
unsafe impl ::core::clone::TrivialClone for ForcedPartialChunkFill {}
#[automatically_derived]
impl ::core::clone::Clone for ForcedPartialChunkFill {
    #[inline]
    fn clone(&self) -> ForcedPartialChunkFill {
        let _: ::core::clone::AssertParamIsClone<usize>;
        *self
    }
}
#[automatically_derived]
impl ::core::marker::Copy for ForcedPartialChunkFill {}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for ForcedPartialChunkFill {}
#[automatically_derived]
impl ::core::cmp::PartialEq for ForcedPartialChunkFill {
    #[inline]
    fn eq(&self, other: &ForcedPartialChunkFill) -> bool {
        let __self_discr = ::core::intrinsics::discriminant_value(self);
        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
        __self_discr == __arg1_discr
            && match (self, other) {
                (
                    ForcedPartialChunkFill::Full(__self_0),
                    ForcedPartialChunkFill::Full(__arg1_0),
                ) => __self_0 == __arg1_0,
                (
                    ForcedPartialChunkFill::Partial(__self_0),
                    ForcedPartialChunkFill::Partial(__arg1_0),
                ) => __self_0 == __arg1_0,
                (
                    ForcedPartialChunkFill::Corrupted(__self_0),
                    ForcedPartialChunkFill::Corrupted(__arg1_0),
                ) => __self_0 == __arg1_0,
                _ => unsafe { ::core::intrinsics::unreachable() }
            }
    }
}
#[automatically_derived]
impl ::core::cmp::Eq for ForcedPartialChunkFill {
    #[inline]
    #[doc(hidden)]
    #[coverage(off)]
    fn assert_fields_are_eq(&self) {
        let _: ::core::cmp::AssertParamIsEq<usize>;
    }
}
#[automatically_derived]
impl ::core::cmp::PartialOrd for ForcedPartialChunkFill {
    #[inline]
    fn partial_cmp(
        &self,
        other: &ForcedPartialChunkFill,
    ) -> ::core::option::Option<::core::cmp::Ordering> {
        let __self_discr = ::core::intrinsics::discriminant_value(self);
        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
        match (self, other) {
            (
                ForcedPartialChunkFill::Full(__self_0),
                ForcedPartialChunkFill::Full(__arg1_0),
            ) => ::core::cmp::PartialOrd::partial_cmp(__self_0, __arg1_0),
            (
                ForcedPartialChunkFill::Partial(__self_0),
                ForcedPartialChunkFill::Partial(__arg1_0),
            ) => ::core::cmp::PartialOrd::partial_cmp(__self_0, __arg1_0),
            (
                ForcedPartialChunkFill::Corrupted(__self_0),
                ForcedPartialChunkFill::Corrupted(__arg1_0),
            ) => ::core::cmp::PartialOrd::partial_cmp(__self_0, __arg1_0),
            _ => ::core::cmp::PartialOrd::partial_cmp(&__self_discr, &__arg1_discr),
        }
    }
}
#[automatically_derived]
impl ::core::cmp::Ord for ForcedPartialChunkFill {
    #[inline]
    fn cmp(&self, other: &ForcedPartialChunkFill) -> ::core::cmp::Ordering {
        let __self_discr = ::core::intrinsics::discriminant_value(self);
        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
        match ::core::cmp::Ord::cmp(&__self_discr, &__arg1_discr) {
            ::core::cmp::Ordering::Equal => {
                match (self, other) {
                    (
                        ForcedPartialChunkFill::Full(__self_0),
                        ForcedPartialChunkFill::Full(__arg1_0),
                    ) => ::core::cmp::Ord::cmp(__self_0, __arg1_0),
                    (
                        ForcedPartialChunkFill::Partial(__self_0),
                        ForcedPartialChunkFill::Partial(__arg1_0),
                    ) => ::core::cmp::Ord::cmp(__self_0, __arg1_0),
                    (
                        ForcedPartialChunkFill::Corrupted(__self_0),
                        ForcedPartialChunkFill::Corrupted(__arg1_0),
                    ) => ::core::cmp::Ord::cmp(__self_0, __arg1_0),
                    _ => unsafe { ::core::intrinsics::unreachable() }
                }
            }
            cmp => cmp,
        }
    }
}
#[automatically_derived]
///An archived [`ForcedPartialChunkFill`]
#[bytecheck(crate = ::rkyv::bytecheck)]
#[repr(u8)]
pub enum ArchivedForcedPartialChunkFill
where
    usize: ::rkyv::Archive,
    usize: ::rkyv::Archive,
    usize: ::rkyv::Archive,
{
    ///The archived counterpart of [`ForcedPartialChunkFill::Full`]
    #[allow(dead_code)]
    Full(
        ///The archived counterpart of [`ForcedPartialChunkFill::Full::0`]
        <usize as ::rkyv::Archive>::Archived,
    ),
    ///The archived counterpart of [`ForcedPartialChunkFill::Partial`]
    #[allow(dead_code)]
    Partial(
        ///The archived counterpart of [`ForcedPartialChunkFill::Partial::0`]
        <usize as ::rkyv::Archive>::Archived,
    ),
    ///The archived counterpart of [`ForcedPartialChunkFill::Corrupted`]
    #[allow(dead_code)]
    Corrupted(
        ///The archived counterpart of [`ForcedPartialChunkFill::Corrupted::0`]
        <usize as ::rkyv::Archive>::Archived,
    ),
}
const _: () = {
    #[repr(u8)]
    enum Tag {
        Full,
        Partial,
        Corrupted,
    }
    struct Discriminant;
    #[automatically_derived]
    impl Discriminant {
        #[allow(non_upper_case_globals)]
        const Full: u8 = Tag::Full as u8;
        #[allow(non_upper_case_globals)]
        const Partial: u8 = Tag::Partial as u8;
        #[allow(non_upper_case_globals)]
        const Corrupted: u8 = Tag::Corrupted as u8;
    }
    #[repr(C)]
    struct VariantFull(
        Tag,
        <usize as ::rkyv::Archive>::Archived,
        ::core::marker::PhantomData<ArchivedForcedPartialChunkFill>,
    )
    where
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive;
    #[repr(C)]
    struct VariantPartial(
        Tag,
        <usize as ::rkyv::Archive>::Archived,
        ::core::marker::PhantomData<ArchivedForcedPartialChunkFill>,
    )
    where
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive;
    #[repr(C)]
    struct VariantCorrupted(
        Tag,
        <usize as ::rkyv::Archive>::Archived,
        ::core::marker::PhantomData<ArchivedForcedPartialChunkFill>,
    )
    where
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive;
    #[automatically_derived]
    unsafe impl<
        __C: ::rkyv::bytecheck::rancor::Fallible + ?::core::marker::Sized,
    > ::rkyv::bytecheck::CheckBytes<__C> for ArchivedForcedPartialChunkFill
    where
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive,
        <__C as ::rkyv::bytecheck::rancor::Fallible>::Error: ::rkyv::bytecheck::rancor::Source,
        <usize as ::rkyv::Archive>::Archived: ::rkyv::bytecheck::CheckBytes<__C>,
        <usize as ::rkyv::Archive>::Archived: ::rkyv::bytecheck::CheckBytes<__C>,
        <usize as ::rkyv::Archive>::Archived: ::rkyv::bytecheck::CheckBytes<__C>,
    {
        unsafe fn check_bytes(
            value: *const Self,
            context: &mut __C,
        ) -> ::core::result::Result<
            (),
            <__C as ::rkyv::bytecheck::rancor::Fallible>::Error,
        > {
            let tag = *value.cast::<u8>();
            match tag {
                Discriminant::Full => {
                    let value = value.cast::<VariantFull>();
                    <<usize as ::rkyv::Archive>::Archived as ::rkyv::bytecheck::CheckBytes<
                        __C,
                    >>::check_bytes(&raw const (*value).1, context)
                        .map_err(|e| {
                            <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Trace>::trace(
                                e,
                                ::rkyv::bytecheck::UnnamedEnumVariantCheckContext {
                                    enum_name: "ArchivedForcedPartialChunkFill",
                                    variant_name: "Full",
                                    field_index: 1,
                                },
                            )
                        })?;
                }
                Discriminant::Partial => {
                    let value = value.cast::<VariantPartial>();
                    <<usize as ::rkyv::Archive>::Archived as ::rkyv::bytecheck::CheckBytes<
                        __C,
                    >>::check_bytes(&raw const (*value).1, context)
                        .map_err(|e| {
                            <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Trace>::trace(
                                e,
                                ::rkyv::bytecheck::UnnamedEnumVariantCheckContext {
                                    enum_name: "ArchivedForcedPartialChunkFill",
                                    variant_name: "Partial",
                                    field_index: 1,
                                },
                            )
                        })?;
                }
                Discriminant::Corrupted => {
                    let value = value.cast::<VariantCorrupted>();
                    <<usize as ::rkyv::Archive>::Archived as ::rkyv::bytecheck::CheckBytes<
                        __C,
                    >>::check_bytes(&raw const (*value).1, context)
                        .map_err(|e| {
                            <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Trace>::trace(
                                e,
                                ::rkyv::bytecheck::UnnamedEnumVariantCheckContext {
                                    enum_name: "ArchivedForcedPartialChunkFill",
                                    variant_name: "Corrupted",
                                    field_index: 1,
                                },
                            )
                        })?;
                }
                _ => {
                    return ::core::result::Result::Err(
                        <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Source>::new(::rkyv::bytecheck::InvalidEnumDiscriminantError {
                            enum_name: "ArchivedForcedPartialChunkFill",
                            invalid_discriminant: tag,
                        }),
                    );
                }
            }
            ::core::result::Result::Ok(())
        }
    }
};
#[automatically_derived]
///The resolver for an archived [`ForcedPartialChunkFill`]
pub enum ForcedPartialChunkFillResolver
where
    usize: ::rkyv::Archive,
    usize: ::rkyv::Archive,
    usize: ::rkyv::Archive,
{
    ///The resolver for [`ForcedPartialChunkFill::Full`]
    #[allow(dead_code)]
    Full(<usize as ::rkyv::Archive>::Resolver),
    ///The resolver for [`ForcedPartialChunkFill::Partial`]
    #[allow(dead_code)]
    Partial(<usize as ::rkyv::Archive>::Resolver),
    ///The resolver for [`ForcedPartialChunkFill::Corrupted`]
    #[allow(dead_code)]
    Corrupted(<usize as ::rkyv::Archive>::Resolver),
}
const _: () = {
    #[repr(u8)]
    enum ArchivedTag {
        Full,
        Partial,
        Corrupted,
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for ArchivedTag {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for ArchivedTag {
        #[inline]
        fn eq(&self, other: &ArchivedTag) -> bool {
            let __self_discr = ::core::intrinsics::discriminant_value(self);
            let __arg1_discr = ::core::intrinsics::discriminant_value(other);
            __self_discr == __arg1_discr
        }
    }
    #[automatically_derived]
    impl ::core::cmp::PartialOrd for ArchivedTag {
        #[inline]
        fn partial_cmp(
            &self,
            other: &ArchivedTag,
        ) -> ::core::option::Option<::core::cmp::Ordering> {
            let __self_discr = ::core::intrinsics::discriminant_value(self);
            let __arg1_discr = ::core::intrinsics::discriminant_value(other);
            ::core::cmp::PartialOrd::partial_cmp(&__self_discr, &__arg1_discr)
        }
    }
    #[repr(C)]
    struct ArchivedVariantFull(
        ArchivedTag,
        <usize as ::rkyv::Archive>::Archived,
        ::core::marker::PhantomData<ForcedPartialChunkFill>,
    )
    where
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive;
    #[repr(C)]
    struct ArchivedVariantPartial(
        ArchivedTag,
        <usize as ::rkyv::Archive>::Archived,
        ::core::marker::PhantomData<ForcedPartialChunkFill>,
    )
    where
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive;
    #[repr(C)]
    struct ArchivedVariantCorrupted(
        ArchivedTag,
        <usize as ::rkyv::Archive>::Archived,
        ::core::marker::PhantomData<ForcedPartialChunkFill>,
    )
    where
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive;
    impl ::rkyv::Archive for ForcedPartialChunkFill
    where
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive,
    {
        type Archived = ArchivedForcedPartialChunkFill;
        type Resolver = ForcedPartialChunkFillResolver;
        #[allow(clippy::unit_arg)]
        fn resolve(
            &self,
            resolver: <Self as ::rkyv::Archive>::Resolver,
            out: ::rkyv::Place<<Self as ::rkyv::Archive>::Archived>,
        ) {
            let __this = self;
            match resolver {
                ForcedPartialChunkFillResolver::Full(resolver_0) => {
                    match __this {
                        ForcedPartialChunkFill::Full(self_0, ..) => {
                            let out = unsafe {
                                out.cast_unchecked::<ArchivedVariantFull>()
                            };
                            let tag_ptr = unsafe { &raw mut (*out.ptr()).0 };
                            unsafe {
                                tag_ptr.write(ArchivedTag::Full);
                            }
                            let field_ptr = unsafe { &raw mut (*out.ptr()).1 };
                            let out_field = unsafe {
                                ::rkyv::Place::from_field_unchecked(out, field_ptr)
                            };
                            <usize as ::rkyv::Archive>::resolve(
                                self_0,
                                resolver_0,
                                out_field,
                            );
                        }
                        #[allow(unreachable_patterns)]
                        _ => unsafe { ::core::hint::unreachable_unchecked() }
                    }
                }
                ForcedPartialChunkFillResolver::Partial(resolver_0) => {
                    match __this {
                        ForcedPartialChunkFill::Partial(self_0, ..) => {
                            let out = unsafe {
                                out.cast_unchecked::<ArchivedVariantPartial>()
                            };
                            let tag_ptr = unsafe { &raw mut (*out.ptr()).0 };
                            unsafe {
                                tag_ptr.write(ArchivedTag::Partial);
                            }
                            let field_ptr = unsafe { &raw mut (*out.ptr()).1 };
                            let out_field = unsafe {
                                ::rkyv::Place::from_field_unchecked(out, field_ptr)
                            };
                            <usize as ::rkyv::Archive>::resolve(
                                self_0,
                                resolver_0,
                                out_field,
                            );
                        }
                        #[allow(unreachable_patterns)]
                        _ => unsafe { ::core::hint::unreachable_unchecked() }
                    }
                }
                ForcedPartialChunkFillResolver::Corrupted(resolver_0) => {
                    match __this {
                        ForcedPartialChunkFill::Corrupted(self_0, ..) => {
                            let out = unsafe {
                                out.cast_unchecked::<ArchivedVariantCorrupted>()
                            };
                            let tag_ptr = unsafe { &raw mut (*out.ptr()).0 };
                            unsafe {
                                tag_ptr.write(ArchivedTag::Corrupted);
                            }
                            let field_ptr = unsafe { &raw mut (*out.ptr()).1 };
                            let out_field = unsafe {
                                ::rkyv::Place::from_field_unchecked(out, field_ptr)
                            };
                            <usize as ::rkyv::Archive>::resolve(
                                self_0,
                                resolver_0,
                                out_field,
                            );
                        }
                        #[allow(unreachable_patterns)]
                        _ => unsafe { ::core::hint::unreachable_unchecked() }
                    }
                }
            }
        }
    }
};
unsafe impl ::rkyv::traits::Portable for ArchivedForcedPartialChunkFill
where
    usize: ::rkyv::Archive,
    usize: ::rkyv::Archive,
    usize: ::rkyv::Archive,
    <usize as ::rkyv::Archive>::Archived: ::rkyv::traits::Portable,
    <usize as ::rkyv::Archive>::Archived: ::rkyv::traits::Portable,
    <usize as ::rkyv::Archive>::Archived: ::rkyv::traits::Portable,
{}
#[automatically_derived]
impl<__S: ::rkyv::rancor::Fallible + ?Sized> ::rkyv::Serialize<__S>
for ForcedPartialChunkFill
where
    usize: ::rkyv::Serialize<__S>,
    usize: ::rkyv::Serialize<__S>,
    usize: ::rkyv::Serialize<__S>,
{
    fn serialize(
        &self,
        serializer: &mut __S,
    ) -> ::core::result::Result<
        <Self as ::rkyv::Archive>::Resolver,
        <__S as ::rkyv::rancor::Fallible>::Error,
    > {
        let __this = self;
        ::core::result::Result::Ok(
            match __this {
                ForcedPartialChunkFill::Full(_0, ..) => {
                    ForcedPartialChunkFillResolver::Full(
                        <usize as ::rkyv::Serialize<__S>>::serialize(_0, serializer)?,
                    )
                }
                ForcedPartialChunkFill::Partial(_0, ..) => {
                    ForcedPartialChunkFillResolver::Partial(
                        <usize as ::rkyv::Serialize<__S>>::serialize(_0, serializer)?,
                    )
                }
                ForcedPartialChunkFill::Corrupted(_0, ..) => {
                    ForcedPartialChunkFillResolver::Corrupted(
                        <usize as ::rkyv::Serialize<__S>>::serialize(_0, serializer)?,
                    )
                }
            },
        )
    }
}
#[automatically_derived]
impl<
    __D: ::rkyv::rancor::Fallible + ?Sized,
> ::rkyv::Deserialize<ForcedPartialChunkFill, __D>
for ::rkyv::Archived<ForcedPartialChunkFill>
where
    usize: ::rkyv::Archive,
    <usize as ::rkyv::Archive>::Archived: ::rkyv::Deserialize<usize, __D>,
    usize: ::rkyv::Archive,
    <usize as ::rkyv::Archive>::Archived: ::rkyv::Deserialize<usize, __D>,
    usize: ::rkyv::Archive,
    <usize as ::rkyv::Archive>::Archived: ::rkyv::Deserialize<usize, __D>,
{
    fn deserialize(
        &self,
        deserializer: &mut __D,
    ) -> ::core::result::Result<
        ForcedPartialChunkFill,
        <__D as ::rkyv::rancor::Fallible>::Error,
    > {
        let __this = self;
        ::core::result::Result::Ok(
            match __this {
                Self::Full(_0, ..) => {
                    ForcedPartialChunkFill::Full(
                        <<usize as ::rkyv::Archive>::Archived as ::rkyv::Deserialize<
                            usize,
                            __D,
                        >>::deserialize(_0, deserializer)?,
                    )
                }
                Self::Partial(_0, ..) => {
                    ForcedPartialChunkFill::Partial(
                        <<usize as ::rkyv::Archive>::Archived as ::rkyv::Deserialize<
                            usize,
                            __D,
                        >>::deserialize(_0, deserializer)?,
                    )
                }
                Self::Corrupted(_0, ..) => {
                    ForcedPartialChunkFill::Corrupted(
                        <<usize as ::rkyv::Archive>::Archived as ::rkyv::Deserialize<
                            usize,
                            __D,
                        >>::deserialize(_0, deserializer)?,
                    )
                }
            },
        )
    }
}
impl ForcedPartialChunkFill {
    pub fn from_size(actual: usize, expected: usize) -> Self {
        if actual == expected {
            Self::Full(actual)
        } else if actual < expected {
            Self::Partial(actual)
        } else {
            Self::Corrupted(actual)
        }
    }
}
impl rewrite::traits::structural::blob::BlobItemChunk for ForcedPartialChunk {
    type Index = usize;
    fn get_index(&self) -> &Self::Index {
        match self {
            Self::Field1(c) => c.get_index(),
            _ => {
                ::core::panicking::panic_fmt(
                    format_args!("get_index called on missing chunk"),
                );
            }
        }
    }
}
impl rewrite::traits::structural::blob::NetabaseBlobItem for ForcedPartial {
    type Chunk = ForcedPartialChunk;
    type BlobIter = std::vec::IntoIter<rewrite::results::NetabaseResult<Self::Chunk>>;
    const DEFAULT_CHUNK_SIZE: usize = 0usize;
    fn into_chunks(
        self,
        size: rewrite::traits::structural::blob::ChunkSize,
    ) -> Box<dyn Iterator<Item = Self::Chunk>> {
        Box::new(self.into_chunks_iter(size).filter_map(|r| r.ok()))
    }
    fn into_chunks_iter(
        self,
        size: rewrite::traits::structural::blob::ChunkSize,
    ) -> Self::BlobIter {
        let mut all_chunks = Vec::new();
        {
            let serialized_field: rewrite::results::NetabaseResult<Vec<u8>> = rkyv::to_bytes::<
                rkyv::rancor::Error,
            >(&self.field1)
                .map_err(|e| rewrite::results::NetabaseError::Serialization(
                    ::alloc::__export::must_use({
                        ::alloc::fmt::format(
                            format_args!(
                                "rkyv serialization failed for field {0}: {1:?}",
                                "field1",
                                e,
                            ),
                        )
                    }),
                ))
                .map(|d| d.to_vec());
            match serialized_field {
                Ok(data) => {
                    let chunk_size = match size {
                        rewrite::traits::structural::blob::ChunkSize::Default => {
                            let default = 0usize;
                            if default > 0 { default } else { 1024 }
                        }
                        rewrite::traits::structural::blob::ChunkSize::Size(n) => n,
                    };
                    all_chunks
                        .extend(
                            data
                                .chunks(chunk_size)
                                .enumerate()
                                .map(|(index, chunk_data)| {
                                    Ok(
                                        Self::Chunk::Field1(ForcedPartialField1Chunk {
                                            index,
                                            data: chunk_data.to_vec(),
                                        }),
                                    )
                                }),
                        );
                }
                Err(e) => {
                    all_chunks.push(Err(e));
                }
            }
        }
        all_chunks.into_iter()
    }
    fn try_from_chunks(
        chunks: impl Iterator<Item = Self::Chunk>,
        size: rewrite::traits::structural::blob::ChunkSize,
    ) -> rewrite::results::NetabaseResult<Self> {
        let mut chunks_field1 = Vec::new();
        for chunk in chunks {
            match chunk {
                Self::Chunk::Field1(c) => chunks_field1.push(c),
                _ => {}
            }
        }
        let field1 = {
            if chunks_field1.is_empty() {
                return Err(
                    rewrite::results::NetabaseError::BlobReconstruction(
                        rewrite::results::BlobReconstructionError::MissingChunks,
                    ),
                );
            }
            let mut sorted = chunks_field1;
            sorted.sort_by_key(|c| c.index);
            let chunk_size = match size {
                rewrite::traits::structural::blob::ChunkSize::Default => {
                    let default = 0usize;
                    if default > 0 { default } else { 1024 }
                }
                rewrite::traits::structural::blob::ChunkSize::Size(n) => n,
            };
            let mut missing_details = Vec::new();
            let mut next_expected = 0;
            let max_idx = sorted.last().map(|c| c.index).unwrap_or(0);
            for chunk in &sorted {
                while chunk.index > next_expected {
                    missing_details
                        .push(
                            ::alloc::__export::must_use({
                                ::alloc::fmt::format(
                                    format_args!(
                                        "{0:?}({{ Index: {1}, Size: {2} }})",
                                        ForcedPartialChunkFill::Full(chunk_size),
                                        next_expected,
                                        chunk_size,
                                    ),
                                )
                            }),
                        );
                    next_expected += 1;
                }
                let fill = ForcedPartialChunkFill::from_size(
                    chunk.data.len(),
                    chunk_size,
                );
                match fill {
                    ForcedPartialChunkFill::Corrupted(size) => {
                        return Err(
                            rewrite::results::NetabaseError::BlobReconstruction(
                                rewrite::results::BlobReconstructionError::InvalidChunkData(
                                    ::alloc::__export::must_use({
                                        ::alloc::fmt::format(
                                            format_args!(
                                                "Corrupted chunk detected for field {0}: {1:?}({{ Index: {2}, Size: {3} }}). Max allowed size is {4}.",
                                                "field1",
                                                fill,
                                                chunk.index,
                                                size,
                                                chunk_size,
                                            ),
                                        )
                                    }),
                                ),
                            ),
                        );
                    }
                    ForcedPartialChunkFill::Partial(size) if chunk.index < max_idx => {
                        return Err(
                            rewrite::results::NetabaseError::BlobReconstruction(
                                rewrite::results::BlobReconstructionError::InvalidChunkData(
                                    ::alloc::__export::must_use({
                                        ::alloc::fmt::format(
                                            format_args!(
                                                "Unexpected partial chunk in middle of stream for field {0}: {1:?}({{ Index: {2}, Size: {3} }}). Expected {4} bytes.",
                                                "field1",
                                                fill,
                                                chunk.index,
                                                size,
                                                chunk_size,
                                            ),
                                        )
                                    }),
                                ),
                            ),
                        );
                    }
                    _ => {}
                }
                if chunk.index == next_expected {
                    next_expected += 1;
                }
            }
            if !missing_details.is_empty() {
                if let Some(last) = sorted.last() {
                    let fill = ForcedPartialChunkFill::from_size(
                        last.data.len(),
                        chunk_size,
                    );
                    if #[allow(non_exhaustive_omitted_patterns)]
                    match fill {
                        ForcedPartialChunkFill::Full(_) => true,
                        _ => false,
                    } {
                        missing_details
                            .push(
                                ::alloc::__export::must_use({
                                    ::alloc::fmt::format(
                                        format_args!(
                                            "... (Stream truncated for field {0}: last chunk was Full, expected more data after Index {1})",
                                            "field1",
                                            last.index,
                                        ),
                                    )
                                }),
                            );
                    }
                }
            }
            if !missing_details.is_empty() {
                return Err(
                    rewrite::results::NetabaseError::BlobReconstruction(
                        rewrite::results::BlobReconstructionError::InvalidChunkData(
                            ::alloc::__export::must_use({
                                ::alloc::fmt::format(
                                    format_args!(
                                        "Missing chunks for field {0}: [{1}]. Total chunks present: {2}",
                                        "field1",
                                        missing_details.join(", "),
                                        sorted.len(),
                                    ),
                                )
                            }),
                        ),
                    ),
                );
            }
            let data: Vec<u8> = sorted.into_iter().flat_map(|c| c.data).collect();
            rkyv::from_bytes::<String, rkyv::rancor::Error>(&data)
                .map_err(|e| rewrite::results::NetabaseError::Serialization(
                    ::alloc::__export::must_use({
                        ::alloc::fmt::format(
                            format_args!(
                                "rkyv deserialization failed for field {0}: {1:?}",
                                "field1",
                                e,
                            ),
                        )
                    }),
                ))?
        };
        Ok(Self { field1 })
    }
    fn get_blob(&self) -> &Self::Chunk {
        {
            ::core::panicking::panic_fmt(
                format_args!(
                    "not implemented: {0}",
                    format_args!("get_blob() requires storing a chunk reference"),
                ),
            );
        }
    }
}
impl IntoIterator for ForcedPartial {
    type Item = rewrite::results::NetabaseResult<ForcedPartialChunk>;
    type IntoIter = std::vec::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        rewrite::traits::structural::blob::NetabaseBlobItem::into_chunks_iter(
            self,
            rewrite::traits::structural::blob::ChunkSize::Default,
        )
    }
}
#[strategy("partial")]
struct StandaloneStrategy {
    field1: String,
}
#[automatically_derived]
///An archived [`StandaloneStrategy`]
#[bytecheck(crate = ::rkyv::bytecheck)]
#[repr(C)]
struct ArchivedStandaloneStrategy
where
    String: ::rkyv::Archive,
{
    ///The archived counterpart of [`StandaloneStrategy::field1`]
    field1: <String as ::rkyv::Archive>::Archived,
}
#[automatically_derived]
unsafe impl<
    __C: ::rkyv::bytecheck::rancor::Fallible + ?::core::marker::Sized,
> ::rkyv::bytecheck::CheckBytes<__C> for ArchivedStandaloneStrategy
where
    String: ::rkyv::Archive,
    <__C as ::rkyv::bytecheck::rancor::Fallible>::Error: ::rkyv::bytecheck::rancor::Trace,
    <String as ::rkyv::Archive>::Archived: ::rkyv::bytecheck::CheckBytes<__C>,
{
    unsafe fn check_bytes(
        value: *const Self,
        context: &mut __C,
    ) -> ::core::result::Result<
        (),
        <__C as ::rkyv::bytecheck::rancor::Fallible>::Error,
    > {
        <<String as ::rkyv::Archive>::Archived as ::rkyv::bytecheck::CheckBytes<
            __C,
        >>::check_bytes(&raw const (*value).field1, context)
            .map_err(|e| {
                <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Trace>::trace(
                    e,
                    ::rkyv::bytecheck::StructCheckContext {
                        struct_name: "ArchivedStandaloneStrategy",
                        field_name: "field1",
                    },
                )
            })?;
        ::core::result::Result::Ok(())
    }
}
#[automatically_derived]
///The resolver for an archived [`StandaloneStrategy`]
struct StandaloneStrategyResolver
where
    String: ::rkyv::Archive,
{
    field1: <String as ::rkyv::Archive>::Resolver,
}
impl ::rkyv::Archive for StandaloneStrategy
where
    String: ::rkyv::Archive,
{
    type Archived = ArchivedStandaloneStrategy;
    type Resolver = StandaloneStrategyResolver;
    const COPY_OPTIMIZATION: ::rkyv::traits::CopyOptimization<Self> = unsafe {
        ::rkyv::traits::CopyOptimization::enable_if(
            0 + ::core::mem::size_of::<String>()
                == ::core::mem::size_of::<StandaloneStrategy>()
                && <String as ::rkyv::Archive>::COPY_OPTIMIZATION.is_enabled()
                && const { builtin # offset_of(StandaloneStrategy, field1) }
                    == const { builtin # offset_of(ArchivedStandaloneStrategy, field1) },
        )
    };
    #[allow(clippy::unit_arg)]
    fn resolve(&self, resolver: Self::Resolver, out: ::rkyv::Place<Self::Archived>) {
        let field_ptr = unsafe { &raw mut (*out.ptr()).field1 };
        let field_out = unsafe { ::rkyv::Place::from_field_unchecked(out, field_ptr) };
        <String as ::rkyv::Archive>::resolve(&self.field1, resolver.field1, field_out);
    }
}
unsafe impl ::rkyv::traits::Portable for ArchivedStandaloneStrategy
where
    String: ::rkyv::Archive,
    <String as ::rkyv::Archive>::Archived: ::rkyv::traits::Portable,
{}
#[automatically_derived]
impl<__S: ::rkyv::rancor::Fallible + ?Sized> ::rkyv::Serialize<__S>
for StandaloneStrategy
where
    String: ::rkyv::Serialize<__S>,
{
    fn serialize(
        &self,
        serializer: &mut __S,
    ) -> ::core::result::Result<
        <Self as ::rkyv::Archive>::Resolver,
        <__S as ::rkyv::rancor::Fallible>::Error,
    > {
        let __this = self;
        ::core::result::Result::Ok(StandaloneStrategyResolver {
            field1: <String as ::rkyv::Serialize<
                __S,
            >>::serialize(&__this.field1, serializer)?,
        })
    }
}
#[automatically_derived]
impl<__D: ::rkyv::rancor::Fallible + ?Sized> ::rkyv::Deserialize<StandaloneStrategy, __D>
for ::rkyv::Archived<StandaloneStrategy>
where
    String: ::rkyv::Archive,
    <String as ::rkyv::Archive>::Archived: ::rkyv::Deserialize<String, __D>,
{
    fn deserialize(
        &self,
        deserializer: &mut __D,
    ) -> ::core::result::Result<
        StandaloneStrategy,
        <__D as ::rkyv::rancor::Fallible>::Error,
    > {
        let __this = self;
        ::core::result::Result::Ok(StandaloneStrategy {
            field1: <<String as ::rkyv::Archive>::Archived as ::rkyv::Deserialize<
                String,
                __D,
            >>::deserialize(&__this.field1, deserializer)?,
        })
    }
}
#[automatically_derived]
impl ::core::fmt::Debug for StandaloneStrategy {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::debug_struct_field1_finish(
            f,
            "StandaloneStrategy",
            "field1",
            &&self.field1,
        )
    }
}
#[automatically_derived]
impl ::core::clone::Clone for StandaloneStrategy {
    #[inline]
    fn clone(&self) -> StandaloneStrategy {
        StandaloneStrategy {
            field1: ::core::clone::Clone::clone(&self.field1),
        }
    }
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for StandaloneStrategy {}
#[automatically_derived]
impl ::core::cmp::PartialEq for StandaloneStrategy {
    #[inline]
    fn eq(&self, other: &StandaloneStrategy) -> bool {
        self.field1 == other.field1
    }
}
#[automatically_derived]
impl ::core::cmp::Eq for StandaloneStrategy {
    #[inline]
    #[doc(hidden)]
    #[coverage(off)]
    fn assert_fields_are_eq(&self) {
        let _: ::core::cmp::AssertParamIsEq<String>;
    }
}
pub struct StandaloneStrategyChunk {
    pub index: usize,
    pub data: Vec<u8>,
}
#[automatically_derived]
impl ::core::fmt::Debug for StandaloneStrategyChunk {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::debug_struct_field2_finish(
            f,
            "StandaloneStrategyChunk",
            "index",
            &self.index,
            "data",
            &&self.data,
        )
    }
}
#[automatically_derived]
impl ::core::clone::Clone for StandaloneStrategyChunk {
    #[inline]
    fn clone(&self) -> StandaloneStrategyChunk {
        StandaloneStrategyChunk {
            index: ::core::clone::Clone::clone(&self.index),
            data: ::core::clone::Clone::clone(&self.data),
        }
    }
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for StandaloneStrategyChunk {}
#[automatically_derived]
impl ::core::cmp::PartialEq for StandaloneStrategyChunk {
    #[inline]
    fn eq(&self, other: &StandaloneStrategyChunk) -> bool {
        self.index == other.index && self.data == other.data
    }
}
#[automatically_derived]
impl ::core::cmp::Eq for StandaloneStrategyChunk {
    #[inline]
    #[doc(hidden)]
    #[coverage(off)]
    fn assert_fields_are_eq(&self) {
        let _: ::core::cmp::AssertParamIsEq<usize>;
        let _: ::core::cmp::AssertParamIsEq<Vec<u8>>;
    }
}
#[automatically_derived]
impl ::core::cmp::PartialOrd for StandaloneStrategyChunk {
    #[inline]
    fn partial_cmp(
        &self,
        other: &StandaloneStrategyChunk,
    ) -> ::core::option::Option<::core::cmp::Ordering> {
        match ::core::cmp::PartialOrd::partial_cmp(&self.index, &other.index) {
            ::core::option::Option::Some(::core::cmp::Ordering::Equal) => {
                ::core::cmp::PartialOrd::partial_cmp(&self.data, &other.data)
            }
            cmp => cmp,
        }
    }
}
#[automatically_derived]
impl ::core::cmp::Ord for StandaloneStrategyChunk {
    #[inline]
    fn cmp(&self, other: &StandaloneStrategyChunk) -> ::core::cmp::Ordering {
        match ::core::cmp::Ord::cmp(&self.index, &other.index) {
            ::core::cmp::Ordering::Equal => {
                ::core::cmp::Ord::cmp(&self.data, &other.data)
            }
            cmp => cmp,
        }
    }
}
#[automatically_derived]
///An archived [`StandaloneStrategyChunk`]
#[bytecheck(crate = ::rkyv::bytecheck)]
#[repr(C)]
pub struct ArchivedStandaloneStrategyChunk
where
    usize: ::rkyv::Archive,
    Vec<u8>: ::rkyv::Archive,
{
    ///The archived counterpart of [`StandaloneStrategyChunk::index`]
    pub index: <usize as ::rkyv::Archive>::Archived,
    ///The archived counterpart of [`StandaloneStrategyChunk::data`]
    pub data: <Vec<u8> as ::rkyv::Archive>::Archived,
}
#[automatically_derived]
unsafe impl<
    __C: ::rkyv::bytecheck::rancor::Fallible + ?::core::marker::Sized,
> ::rkyv::bytecheck::CheckBytes<__C> for ArchivedStandaloneStrategyChunk
where
    usize: ::rkyv::Archive,
    Vec<u8>: ::rkyv::Archive,
    <__C as ::rkyv::bytecheck::rancor::Fallible>::Error: ::rkyv::bytecheck::rancor::Trace,
    <usize as ::rkyv::Archive>::Archived: ::rkyv::bytecheck::CheckBytes<__C>,
    <Vec<u8> as ::rkyv::Archive>::Archived: ::rkyv::bytecheck::CheckBytes<__C>,
{
    unsafe fn check_bytes(
        value: *const Self,
        context: &mut __C,
    ) -> ::core::result::Result<
        (),
        <__C as ::rkyv::bytecheck::rancor::Fallible>::Error,
    > {
        <<usize as ::rkyv::Archive>::Archived as ::rkyv::bytecheck::CheckBytes<
            __C,
        >>::check_bytes(&raw const (*value).index, context)
            .map_err(|e| {
                <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Trace>::trace(
                    e,
                    ::rkyv::bytecheck::StructCheckContext {
                        struct_name: "ArchivedStandaloneStrategyChunk",
                        field_name: "index",
                    },
                )
            })?;
        <<Vec<
            u8,
        > as ::rkyv::Archive>::Archived as ::rkyv::bytecheck::CheckBytes<
            __C,
        >>::check_bytes(&raw const (*value).data, context)
            .map_err(|e| {
                <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Trace>::trace(
                    e,
                    ::rkyv::bytecheck::StructCheckContext {
                        struct_name: "ArchivedStandaloneStrategyChunk",
                        field_name: "data",
                    },
                )
            })?;
        ::core::result::Result::Ok(())
    }
}
#[automatically_derived]
///The resolver for an archived [`StandaloneStrategyChunk`]
pub struct StandaloneStrategyChunkResolver
where
    usize: ::rkyv::Archive,
    Vec<u8>: ::rkyv::Archive,
{
    index: <usize as ::rkyv::Archive>::Resolver,
    data: <Vec<u8> as ::rkyv::Archive>::Resolver,
}
impl ::rkyv::Archive for StandaloneStrategyChunk
where
    usize: ::rkyv::Archive,
    Vec<u8>: ::rkyv::Archive,
{
    type Archived = ArchivedStandaloneStrategyChunk;
    type Resolver = StandaloneStrategyChunkResolver;
    const COPY_OPTIMIZATION: ::rkyv::traits::CopyOptimization<Self> = unsafe {
        ::rkyv::traits::CopyOptimization::enable_if(
            0 + ::core::mem::size_of::<usize>() + ::core::mem::size_of::<Vec<u8>>()
                == ::core::mem::size_of::<StandaloneStrategyChunk>()
                && <usize as ::rkyv::Archive>::COPY_OPTIMIZATION.is_enabled()
                && const { builtin # offset_of(StandaloneStrategyChunk, index) }
                    == const {
                        builtin # offset_of(ArchivedStandaloneStrategyChunk, index)
                    } && <Vec<u8> as ::rkyv::Archive>::COPY_OPTIMIZATION.is_enabled()
                && const { builtin # offset_of(StandaloneStrategyChunk, data) }
                    == const {
                        builtin # offset_of(ArchivedStandaloneStrategyChunk, data)
                    },
        )
    };
    #[allow(clippy::unit_arg)]
    fn resolve(&self, resolver: Self::Resolver, out: ::rkyv::Place<Self::Archived>) {
        let field_ptr = unsafe { &raw mut (*out.ptr()).index };
        let field_out = unsafe { ::rkyv::Place::from_field_unchecked(out, field_ptr) };
        <usize as ::rkyv::Archive>::resolve(&self.index, resolver.index, field_out);
        let field_ptr = unsafe { &raw mut (*out.ptr()).data };
        let field_out = unsafe { ::rkyv::Place::from_field_unchecked(out, field_ptr) };
        <Vec<u8> as ::rkyv::Archive>::resolve(&self.data, resolver.data, field_out);
    }
}
unsafe impl ::rkyv::traits::Portable for ArchivedStandaloneStrategyChunk
where
    usize: ::rkyv::Archive,
    Vec<u8>: ::rkyv::Archive,
    <usize as ::rkyv::Archive>::Archived: ::rkyv::traits::Portable,
    <Vec<u8> as ::rkyv::Archive>::Archived: ::rkyv::traits::Portable,
{}
#[automatically_derived]
impl<__S: ::rkyv::rancor::Fallible + ?Sized> ::rkyv::Serialize<__S>
for StandaloneStrategyChunk
where
    usize: ::rkyv::Serialize<__S>,
    Vec<u8>: ::rkyv::Serialize<__S>,
{
    fn serialize(
        &self,
        serializer: &mut __S,
    ) -> ::core::result::Result<
        <Self as ::rkyv::Archive>::Resolver,
        <__S as ::rkyv::rancor::Fallible>::Error,
    > {
        let __this = self;
        ::core::result::Result::Ok(StandaloneStrategyChunkResolver {
            index: <usize as ::rkyv::Serialize<
                __S,
            >>::serialize(&__this.index, serializer)?,
            data: <Vec<
                u8,
            > as ::rkyv::Serialize<__S>>::serialize(&__this.data, serializer)?,
        })
    }
}
#[automatically_derived]
impl<
    __D: ::rkyv::rancor::Fallible + ?Sized,
> ::rkyv::Deserialize<StandaloneStrategyChunk, __D>
for ::rkyv::Archived<StandaloneStrategyChunk>
where
    usize: ::rkyv::Archive,
    <usize as ::rkyv::Archive>::Archived: ::rkyv::Deserialize<usize, __D>,
    Vec<u8>: ::rkyv::Archive,
    <Vec<u8> as ::rkyv::Archive>::Archived: ::rkyv::Deserialize<Vec<u8>, __D>,
{
    fn deserialize(
        &self,
        deserializer: &mut __D,
    ) -> ::core::result::Result<
        StandaloneStrategyChunk,
        <__D as ::rkyv::rancor::Fallible>::Error,
    > {
        let __this = self;
        ::core::result::Result::Ok(StandaloneStrategyChunk {
            index: <<usize as ::rkyv::Archive>::Archived as ::rkyv::Deserialize<
                usize,
                __D,
            >>::deserialize(&__this.index, deserializer)?,
            data: <<Vec<
                u8,
            > as ::rkyv::Archive>::Archived as ::rkyv::Deserialize<
                Vec<u8>,
                __D,
            >>::deserialize(&__this.data, deserializer)?,
        })
    }
}
impl ::rewrite::traits::structural::blob::BlobItemChunk for StandaloneStrategyChunk {
    type Index = usize;
    fn get_index(&self) -> &Self::Index {
        &self.index
    }
}
pub enum StandaloneStrategyChunkFill {
    Full(usize),
    Partial(usize),
    Corrupted(usize),
}
#[automatically_derived]
impl ::core::fmt::Debug for StandaloneStrategyChunkFill {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match self {
            StandaloneStrategyChunkFill::Full(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "Full", &__self_0)
            }
            StandaloneStrategyChunkFill::Partial(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(
                    f,
                    "Partial",
                    &__self_0,
                )
            }
            StandaloneStrategyChunkFill::Corrupted(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(
                    f,
                    "Corrupted",
                    &__self_0,
                )
            }
        }
    }
}
#[automatically_derived]
#[doc(hidden)]
unsafe impl ::core::clone::TrivialClone for StandaloneStrategyChunkFill {}
#[automatically_derived]
impl ::core::clone::Clone for StandaloneStrategyChunkFill {
    #[inline]
    fn clone(&self) -> StandaloneStrategyChunkFill {
        let _: ::core::clone::AssertParamIsClone<usize>;
        *self
    }
}
#[automatically_derived]
impl ::core::marker::Copy for StandaloneStrategyChunkFill {}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for StandaloneStrategyChunkFill {}
#[automatically_derived]
impl ::core::cmp::PartialEq for StandaloneStrategyChunkFill {
    #[inline]
    fn eq(&self, other: &StandaloneStrategyChunkFill) -> bool {
        let __self_discr = ::core::intrinsics::discriminant_value(self);
        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
        __self_discr == __arg1_discr
            && match (self, other) {
                (
                    StandaloneStrategyChunkFill::Full(__self_0),
                    StandaloneStrategyChunkFill::Full(__arg1_0),
                ) => __self_0 == __arg1_0,
                (
                    StandaloneStrategyChunkFill::Partial(__self_0),
                    StandaloneStrategyChunkFill::Partial(__arg1_0),
                ) => __self_0 == __arg1_0,
                (
                    StandaloneStrategyChunkFill::Corrupted(__self_0),
                    StandaloneStrategyChunkFill::Corrupted(__arg1_0),
                ) => __self_0 == __arg1_0,
                _ => unsafe { ::core::intrinsics::unreachable() }
            }
    }
}
#[automatically_derived]
impl ::core::cmp::Eq for StandaloneStrategyChunkFill {
    #[inline]
    #[doc(hidden)]
    #[coverage(off)]
    fn assert_fields_are_eq(&self) {
        let _: ::core::cmp::AssertParamIsEq<usize>;
    }
}
#[automatically_derived]
impl ::core::cmp::PartialOrd for StandaloneStrategyChunkFill {
    #[inline]
    fn partial_cmp(
        &self,
        other: &StandaloneStrategyChunkFill,
    ) -> ::core::option::Option<::core::cmp::Ordering> {
        let __self_discr = ::core::intrinsics::discriminant_value(self);
        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
        match (self, other) {
            (
                StandaloneStrategyChunkFill::Full(__self_0),
                StandaloneStrategyChunkFill::Full(__arg1_0),
            ) => ::core::cmp::PartialOrd::partial_cmp(__self_0, __arg1_0),
            (
                StandaloneStrategyChunkFill::Partial(__self_0),
                StandaloneStrategyChunkFill::Partial(__arg1_0),
            ) => ::core::cmp::PartialOrd::partial_cmp(__self_0, __arg1_0),
            (
                StandaloneStrategyChunkFill::Corrupted(__self_0),
                StandaloneStrategyChunkFill::Corrupted(__arg1_0),
            ) => ::core::cmp::PartialOrd::partial_cmp(__self_0, __arg1_0),
            _ => ::core::cmp::PartialOrd::partial_cmp(&__self_discr, &__arg1_discr),
        }
    }
}
#[automatically_derived]
impl ::core::cmp::Ord for StandaloneStrategyChunkFill {
    #[inline]
    fn cmp(&self, other: &StandaloneStrategyChunkFill) -> ::core::cmp::Ordering {
        let __self_discr = ::core::intrinsics::discriminant_value(self);
        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
        match ::core::cmp::Ord::cmp(&__self_discr, &__arg1_discr) {
            ::core::cmp::Ordering::Equal => {
                match (self, other) {
                    (
                        StandaloneStrategyChunkFill::Full(__self_0),
                        StandaloneStrategyChunkFill::Full(__arg1_0),
                    ) => ::core::cmp::Ord::cmp(__self_0, __arg1_0),
                    (
                        StandaloneStrategyChunkFill::Partial(__self_0),
                        StandaloneStrategyChunkFill::Partial(__arg1_0),
                    ) => ::core::cmp::Ord::cmp(__self_0, __arg1_0),
                    (
                        StandaloneStrategyChunkFill::Corrupted(__self_0),
                        StandaloneStrategyChunkFill::Corrupted(__arg1_0),
                    ) => ::core::cmp::Ord::cmp(__self_0, __arg1_0),
                    _ => unsafe { ::core::intrinsics::unreachable() }
                }
            }
            cmp => cmp,
        }
    }
}
#[automatically_derived]
///An archived [`StandaloneStrategyChunkFill`]
#[bytecheck(crate = ::rkyv::bytecheck)]
#[repr(u8)]
pub enum ArchivedStandaloneStrategyChunkFill
where
    usize: ::rkyv::Archive,
    usize: ::rkyv::Archive,
    usize: ::rkyv::Archive,
{
    ///The archived counterpart of [`StandaloneStrategyChunkFill::Full`]
    #[allow(dead_code)]
    Full(
        ///The archived counterpart of [`StandaloneStrategyChunkFill::Full::0`]
        <usize as ::rkyv::Archive>::Archived,
    ),
    ///The archived counterpart of [`StandaloneStrategyChunkFill::Partial`]
    #[allow(dead_code)]
    Partial(
        ///The archived counterpart of [`StandaloneStrategyChunkFill::Partial::0`]
        <usize as ::rkyv::Archive>::Archived,
    ),
    ///The archived counterpart of [`StandaloneStrategyChunkFill::Corrupted`]
    #[allow(dead_code)]
    Corrupted(
        ///The archived counterpart of [`StandaloneStrategyChunkFill::Corrupted::0`]
        <usize as ::rkyv::Archive>::Archived,
    ),
}
const _: () = {
    #[repr(u8)]
    enum Tag {
        Full,
        Partial,
        Corrupted,
    }
    struct Discriminant;
    #[automatically_derived]
    impl Discriminant {
        #[allow(non_upper_case_globals)]
        const Full: u8 = Tag::Full as u8;
        #[allow(non_upper_case_globals)]
        const Partial: u8 = Tag::Partial as u8;
        #[allow(non_upper_case_globals)]
        const Corrupted: u8 = Tag::Corrupted as u8;
    }
    #[repr(C)]
    struct VariantFull(
        Tag,
        <usize as ::rkyv::Archive>::Archived,
        ::core::marker::PhantomData<ArchivedStandaloneStrategyChunkFill>,
    )
    where
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive;
    #[repr(C)]
    struct VariantPartial(
        Tag,
        <usize as ::rkyv::Archive>::Archived,
        ::core::marker::PhantomData<ArchivedStandaloneStrategyChunkFill>,
    )
    where
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive;
    #[repr(C)]
    struct VariantCorrupted(
        Tag,
        <usize as ::rkyv::Archive>::Archived,
        ::core::marker::PhantomData<ArchivedStandaloneStrategyChunkFill>,
    )
    where
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive;
    #[automatically_derived]
    unsafe impl<
        __C: ::rkyv::bytecheck::rancor::Fallible + ?::core::marker::Sized,
    > ::rkyv::bytecheck::CheckBytes<__C> for ArchivedStandaloneStrategyChunkFill
    where
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive,
        <__C as ::rkyv::bytecheck::rancor::Fallible>::Error: ::rkyv::bytecheck::rancor::Source,
        <usize as ::rkyv::Archive>::Archived: ::rkyv::bytecheck::CheckBytes<__C>,
        <usize as ::rkyv::Archive>::Archived: ::rkyv::bytecheck::CheckBytes<__C>,
        <usize as ::rkyv::Archive>::Archived: ::rkyv::bytecheck::CheckBytes<__C>,
    {
        unsafe fn check_bytes(
            value: *const Self,
            context: &mut __C,
        ) -> ::core::result::Result<
            (),
            <__C as ::rkyv::bytecheck::rancor::Fallible>::Error,
        > {
            let tag = *value.cast::<u8>();
            match tag {
                Discriminant::Full => {
                    let value = value.cast::<VariantFull>();
                    <<usize as ::rkyv::Archive>::Archived as ::rkyv::bytecheck::CheckBytes<
                        __C,
                    >>::check_bytes(&raw const (*value).1, context)
                        .map_err(|e| {
                            <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Trace>::trace(
                                e,
                                ::rkyv::bytecheck::UnnamedEnumVariantCheckContext {
                                    enum_name: "ArchivedStandaloneStrategyChunkFill",
                                    variant_name: "Full",
                                    field_index: 1,
                                },
                            )
                        })?;
                }
                Discriminant::Partial => {
                    let value = value.cast::<VariantPartial>();
                    <<usize as ::rkyv::Archive>::Archived as ::rkyv::bytecheck::CheckBytes<
                        __C,
                    >>::check_bytes(&raw const (*value).1, context)
                        .map_err(|e| {
                            <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Trace>::trace(
                                e,
                                ::rkyv::bytecheck::UnnamedEnumVariantCheckContext {
                                    enum_name: "ArchivedStandaloneStrategyChunkFill",
                                    variant_name: "Partial",
                                    field_index: 1,
                                },
                            )
                        })?;
                }
                Discriminant::Corrupted => {
                    let value = value.cast::<VariantCorrupted>();
                    <<usize as ::rkyv::Archive>::Archived as ::rkyv::bytecheck::CheckBytes<
                        __C,
                    >>::check_bytes(&raw const (*value).1, context)
                        .map_err(|e| {
                            <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Trace>::trace(
                                e,
                                ::rkyv::bytecheck::UnnamedEnumVariantCheckContext {
                                    enum_name: "ArchivedStandaloneStrategyChunkFill",
                                    variant_name: "Corrupted",
                                    field_index: 1,
                                },
                            )
                        })?;
                }
                _ => {
                    return ::core::result::Result::Err(
                        <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Source>::new(::rkyv::bytecheck::InvalidEnumDiscriminantError {
                            enum_name: "ArchivedStandaloneStrategyChunkFill",
                            invalid_discriminant: tag,
                        }),
                    );
                }
            }
            ::core::result::Result::Ok(())
        }
    }
};
#[automatically_derived]
///The resolver for an archived [`StandaloneStrategyChunkFill`]
pub enum StandaloneStrategyChunkFillResolver
where
    usize: ::rkyv::Archive,
    usize: ::rkyv::Archive,
    usize: ::rkyv::Archive,
{
    ///The resolver for [`StandaloneStrategyChunkFill::Full`]
    #[allow(dead_code)]
    Full(<usize as ::rkyv::Archive>::Resolver),
    ///The resolver for [`StandaloneStrategyChunkFill::Partial`]
    #[allow(dead_code)]
    Partial(<usize as ::rkyv::Archive>::Resolver),
    ///The resolver for [`StandaloneStrategyChunkFill::Corrupted`]
    #[allow(dead_code)]
    Corrupted(<usize as ::rkyv::Archive>::Resolver),
}
const _: () = {
    #[repr(u8)]
    enum ArchivedTag {
        Full,
        Partial,
        Corrupted,
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for ArchivedTag {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for ArchivedTag {
        #[inline]
        fn eq(&self, other: &ArchivedTag) -> bool {
            let __self_discr = ::core::intrinsics::discriminant_value(self);
            let __arg1_discr = ::core::intrinsics::discriminant_value(other);
            __self_discr == __arg1_discr
        }
    }
    #[automatically_derived]
    impl ::core::cmp::PartialOrd for ArchivedTag {
        #[inline]
        fn partial_cmp(
            &self,
            other: &ArchivedTag,
        ) -> ::core::option::Option<::core::cmp::Ordering> {
            let __self_discr = ::core::intrinsics::discriminant_value(self);
            let __arg1_discr = ::core::intrinsics::discriminant_value(other);
            ::core::cmp::PartialOrd::partial_cmp(&__self_discr, &__arg1_discr)
        }
    }
    #[repr(C)]
    struct ArchivedVariantFull(
        ArchivedTag,
        <usize as ::rkyv::Archive>::Archived,
        ::core::marker::PhantomData<StandaloneStrategyChunkFill>,
    )
    where
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive;
    #[repr(C)]
    struct ArchivedVariantPartial(
        ArchivedTag,
        <usize as ::rkyv::Archive>::Archived,
        ::core::marker::PhantomData<StandaloneStrategyChunkFill>,
    )
    where
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive;
    #[repr(C)]
    struct ArchivedVariantCorrupted(
        ArchivedTag,
        <usize as ::rkyv::Archive>::Archived,
        ::core::marker::PhantomData<StandaloneStrategyChunkFill>,
    )
    where
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive;
    impl ::rkyv::Archive for StandaloneStrategyChunkFill
    where
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive,
        usize: ::rkyv::Archive,
    {
        type Archived = ArchivedStandaloneStrategyChunkFill;
        type Resolver = StandaloneStrategyChunkFillResolver;
        #[allow(clippy::unit_arg)]
        fn resolve(
            &self,
            resolver: <Self as ::rkyv::Archive>::Resolver,
            out: ::rkyv::Place<<Self as ::rkyv::Archive>::Archived>,
        ) {
            let __this = self;
            match resolver {
                StandaloneStrategyChunkFillResolver::Full(resolver_0) => {
                    match __this {
                        StandaloneStrategyChunkFill::Full(self_0, ..) => {
                            let out = unsafe {
                                out.cast_unchecked::<ArchivedVariantFull>()
                            };
                            let tag_ptr = unsafe { &raw mut (*out.ptr()).0 };
                            unsafe {
                                tag_ptr.write(ArchivedTag::Full);
                            }
                            let field_ptr = unsafe { &raw mut (*out.ptr()).1 };
                            let out_field = unsafe {
                                ::rkyv::Place::from_field_unchecked(out, field_ptr)
                            };
                            <usize as ::rkyv::Archive>::resolve(
                                self_0,
                                resolver_0,
                                out_field,
                            );
                        }
                        #[allow(unreachable_patterns)]
                        _ => unsafe { ::core::hint::unreachable_unchecked() }
                    }
                }
                StandaloneStrategyChunkFillResolver::Partial(resolver_0) => {
                    match __this {
                        StandaloneStrategyChunkFill::Partial(self_0, ..) => {
                            let out = unsafe {
                                out.cast_unchecked::<ArchivedVariantPartial>()
                            };
                            let tag_ptr = unsafe { &raw mut (*out.ptr()).0 };
                            unsafe {
                                tag_ptr.write(ArchivedTag::Partial);
                            }
                            let field_ptr = unsafe { &raw mut (*out.ptr()).1 };
                            let out_field = unsafe {
                                ::rkyv::Place::from_field_unchecked(out, field_ptr)
                            };
                            <usize as ::rkyv::Archive>::resolve(
                                self_0,
                                resolver_0,
                                out_field,
                            );
                        }
                        #[allow(unreachable_patterns)]
                        _ => unsafe { ::core::hint::unreachable_unchecked() }
                    }
                }
                StandaloneStrategyChunkFillResolver::Corrupted(resolver_0) => {
                    match __this {
                        StandaloneStrategyChunkFill::Corrupted(self_0, ..) => {
                            let out = unsafe {
                                out.cast_unchecked::<ArchivedVariantCorrupted>()
                            };
                            let tag_ptr = unsafe { &raw mut (*out.ptr()).0 };
                            unsafe {
                                tag_ptr.write(ArchivedTag::Corrupted);
                            }
                            let field_ptr = unsafe { &raw mut (*out.ptr()).1 };
                            let out_field = unsafe {
                                ::rkyv::Place::from_field_unchecked(out, field_ptr)
                            };
                            <usize as ::rkyv::Archive>::resolve(
                                self_0,
                                resolver_0,
                                out_field,
                            );
                        }
                        #[allow(unreachable_patterns)]
                        _ => unsafe { ::core::hint::unreachable_unchecked() }
                    }
                }
            }
        }
    }
};
unsafe impl ::rkyv::traits::Portable for ArchivedStandaloneStrategyChunkFill
where
    usize: ::rkyv::Archive,
    usize: ::rkyv::Archive,
    usize: ::rkyv::Archive,
    <usize as ::rkyv::Archive>::Archived: ::rkyv::traits::Portable,
    <usize as ::rkyv::Archive>::Archived: ::rkyv::traits::Portable,
    <usize as ::rkyv::Archive>::Archived: ::rkyv::traits::Portable,
{}
#[automatically_derived]
impl<__S: ::rkyv::rancor::Fallible + ?Sized> ::rkyv::Serialize<__S>
for StandaloneStrategyChunkFill
where
    usize: ::rkyv::Serialize<__S>,
    usize: ::rkyv::Serialize<__S>,
    usize: ::rkyv::Serialize<__S>,
{
    fn serialize(
        &self,
        serializer: &mut __S,
    ) -> ::core::result::Result<
        <Self as ::rkyv::Archive>::Resolver,
        <__S as ::rkyv::rancor::Fallible>::Error,
    > {
        let __this = self;
        ::core::result::Result::Ok(
            match __this {
                StandaloneStrategyChunkFill::Full(_0, ..) => {
                    StandaloneStrategyChunkFillResolver::Full(
                        <usize as ::rkyv::Serialize<__S>>::serialize(_0, serializer)?,
                    )
                }
                StandaloneStrategyChunkFill::Partial(_0, ..) => {
                    StandaloneStrategyChunkFillResolver::Partial(
                        <usize as ::rkyv::Serialize<__S>>::serialize(_0, serializer)?,
                    )
                }
                StandaloneStrategyChunkFill::Corrupted(_0, ..) => {
                    StandaloneStrategyChunkFillResolver::Corrupted(
                        <usize as ::rkyv::Serialize<__S>>::serialize(_0, serializer)?,
                    )
                }
            },
        )
    }
}
#[automatically_derived]
impl<
    __D: ::rkyv::rancor::Fallible + ?Sized,
> ::rkyv::Deserialize<StandaloneStrategyChunkFill, __D>
for ::rkyv::Archived<StandaloneStrategyChunkFill>
where
    usize: ::rkyv::Archive,
    <usize as ::rkyv::Archive>::Archived: ::rkyv::Deserialize<usize, __D>,
    usize: ::rkyv::Archive,
    <usize as ::rkyv::Archive>::Archived: ::rkyv::Deserialize<usize, __D>,
    usize: ::rkyv::Archive,
    <usize as ::rkyv::Archive>::Archived: ::rkyv::Deserialize<usize, __D>,
{
    fn deserialize(
        &self,
        deserializer: &mut __D,
    ) -> ::core::result::Result<
        StandaloneStrategyChunkFill,
        <__D as ::rkyv::rancor::Fallible>::Error,
    > {
        let __this = self;
        ::core::result::Result::Ok(
            match __this {
                Self::Full(_0, ..) => {
                    StandaloneStrategyChunkFill::Full(
                        <<usize as ::rkyv::Archive>::Archived as ::rkyv::Deserialize<
                            usize,
                            __D,
                        >>::deserialize(_0, deserializer)?,
                    )
                }
                Self::Partial(_0, ..) => {
                    StandaloneStrategyChunkFill::Partial(
                        <<usize as ::rkyv::Archive>::Archived as ::rkyv::Deserialize<
                            usize,
                            __D,
                        >>::deserialize(_0, deserializer)?,
                    )
                }
                Self::Corrupted(_0, ..) => {
                    StandaloneStrategyChunkFill::Corrupted(
                        <<usize as ::rkyv::Archive>::Archived as ::rkyv::Deserialize<
                            usize,
                            __D,
                        >>::deserialize(_0, deserializer)?,
                    )
                }
            },
        )
    }
}
impl StandaloneStrategyChunkFill {
    pub fn from_size(actual: usize, expected: usize) -> Self {
        if actual == expected {
            Self::Full(actual)
        } else if actual < expected {
            Self::Partial(actual)
        } else {
            Self::Corrupted(actual)
        }
    }
}
impl rewrite::traits::structural::blob::NetabaseBlobItem for StandaloneStrategy {
    type Chunk = StandaloneStrategyChunk;
    type BlobIter = std::vec::IntoIter<rewrite::results::NetabaseResult<Self::Chunk>>;
    const DEFAULT_CHUNK_SIZE: usize = 0usize;
    fn into_chunks(
        self,
        size: rewrite::traits::structural::blob::ChunkSize,
    ) -> Box<dyn Iterator<Item = Self::Chunk>> {
        Box::new(self.into_chunks_iter(size).filter_map(|r| r.ok()))
    }
    fn into_chunks_iter(
        self,
        size: rewrite::traits::structural::blob::ChunkSize,
    ) -> Self::BlobIter {
        let serialized_data: rewrite::results::NetabaseResult<Vec<u8>> = (|| -> rewrite::results::NetabaseResult<
            Vec<u8>,
        > {
            Ok(
                rkyv::to_bytes::<rkyv::rancor::Error>(&self)
                    .map_err(|e| rewrite::results::NetabaseError::Serialization(
                        ::alloc::__export::must_use({
                            ::alloc::fmt::format(
                                format_args!("rkyv serialization failed: {0:?}", e),
                            )
                        }),
                    ))?
                    .to_vec(),
            )
        })();
        let chunk_size = match size {
            rewrite::traits::structural::blob::ChunkSize::Default => {
                if Self::DEFAULT_CHUNK_SIZE > 0 {
                    Self::DEFAULT_CHUNK_SIZE
                } else {
                    1024
                }
            }
            rewrite::traits::structural::blob::ChunkSize::Size(n) => n,
        };
        match serialized_data {
            Ok(data) => {
                data.chunks(chunk_size)
                    .enumerate()
                    .map(|(index, chunk_data)| {
                        Ok(Self::Chunk {
                            index,
                            data: chunk_data.to_vec(),
                        })
                    })
                    .collect::<Vec<_>>()
                    .into_iter()
            }
            Err(e) => {
                ::alloc::boxed::box_assume_init_into_vec_unsafe(
                        ::alloc::intrinsics::write_box_via_move(
                            ::alloc::boxed::Box::new_uninit(),
                            [Err(e)],
                        ),
                    )
                    .into_iter()
            }
        }
    }
    fn try_from_chunks(
        chunks: impl Iterator<Item = Self::Chunk>,
        size: rewrite::traits::structural::blob::ChunkSize,
    ) -> rewrite::results::NetabaseResult<Self> {
        let mut sorted_chunks: Vec<_> = chunks.collect();
        sorted_chunks.sort_by_key(|c| c.index);
        if sorted_chunks.is_empty() {
            return Err(
                rewrite::results::NetabaseError::BlobReconstruction(
                    rewrite::results::BlobReconstructionError::MissingChunks,
                ),
            );
        }
        let chunk_size = match size {
            rewrite::traits::structural::blob::ChunkSize::Default => {
                if Self::DEFAULT_CHUNK_SIZE > 0 {
                    Self::DEFAULT_CHUNK_SIZE
                } else {
                    1024
                }
            }
            rewrite::traits::structural::blob::ChunkSize::Size(n) => n,
        };
        let mut missing_details = Vec::new();
        let mut next_expected = 0;
        let max_idx = sorted_chunks.last().map(|c| c.index).unwrap_or(0);
        for chunk in &sorted_chunks {
            while chunk.index > next_expected {
                missing_details
                    .push(
                        ::alloc::__export::must_use({
                            ::alloc::fmt::format(
                                format_args!(
                                    "{0:?}({{ Index: {1}, Size: {2} }})",
                                    StandaloneStrategyChunkFill::Full(chunk_size),
                                    next_expected,
                                    chunk_size,
                                ),
                            )
                        }),
                    );
                next_expected += 1;
            }
            let fill = StandaloneStrategyChunkFill::from_size(
                chunk.data.len(),
                chunk_size,
            );
            match fill {
                StandaloneStrategyChunkFill::Corrupted(size) => {
                    return Err(
                        rewrite::results::NetabaseError::BlobReconstruction(
                            rewrite::results::BlobReconstructionError::InvalidChunkData(
                                ::alloc::__export::must_use({
                                    ::alloc::fmt::format(
                                        format_args!(
                                            "Corrupted chunk detected: {0:?}({{ Index: {1}, Size: {2} }}). Max allowed size is {3}.",
                                            fill,
                                            chunk.index,
                                            size,
                                            chunk_size,
                                        ),
                                    )
                                }),
                            ),
                        ),
                    );
                }
                StandaloneStrategyChunkFill::Partial(size) if chunk.index < max_idx => {
                    return Err(
                        rewrite::results::NetabaseError::BlobReconstruction(
                            rewrite::results::BlobReconstructionError::InvalidChunkData(
                                ::alloc::__export::must_use({
                                    ::alloc::fmt::format(
                                        format_args!(
                                            "Unexpected partial chunk in middle of stream: {0:?}({{ Index: {1}, Size: {2} }}). Expected {3} bytes.",
                                            fill,
                                            chunk.index,
                                            size,
                                            chunk_size,
                                        ),
                                    )
                                }),
                            ),
                        ),
                    );
                }
                _ => {}
            }
            if chunk.index == next_expected {
                next_expected += 1;
            }
        }
        if !missing_details.is_empty() {
            if let Some(last) = sorted_chunks.last() {
                let fill = StandaloneStrategyChunkFill::from_size(
                    last.data.len(),
                    chunk_size,
                );
                if #[allow(non_exhaustive_omitted_patterns)]
                match fill {
                    StandaloneStrategyChunkFill::Full(_) => true,
                    _ => false,
                } {
                    missing_details
                        .push(
                            ::alloc::__export::must_use({
                                ::alloc::fmt::format(
                                    format_args!(
                                        "... (Stream truncated: last chunk was Full, expected more data after Index {0})",
                                        last.index,
                                    ),
                                )
                            }),
                        );
                }
            }
        }
        if !missing_details.is_empty() {
            return Err(
                rewrite::results::NetabaseError::BlobReconstruction(
                    rewrite::results::BlobReconstructionError::InvalidChunkData(
                        ::alloc::__export::must_use({
                            ::alloc::fmt::format(
                                format_args!(
                                    "Missing chunks: [{0}]. Total chunks present: {1}",
                                    missing_details.join(", "),
                                    sorted_chunks.len(),
                                ),
                            )
                        }),
                    ),
                ),
            );
        }
        let serialized_data: Vec<u8> = sorted_chunks
            .into_iter()
            .flat_map(|c| c.data)
            .collect();
        Ok(
            rkyv::from_bytes::<Self, rkyv::rancor::Error>(&serialized_data)
                .map_err(|e| rewrite::results::NetabaseError::Serialization(
                    ::alloc::__export::must_use({
                        ::alloc::fmt::format(
                            format_args!("rkyv deserialization failed: {0:?}", e),
                        )
                    }),
                ))?,
        )
    }
    fn get_blob(&self) -> &Self::Chunk {
        {
            ::core::panicking::panic_fmt(
                format_args!(
                    "not implemented: {0}",
                    format_args!("get_blob() requires storing a chunk reference"),
                ),
            );
        }
    }
}
impl IntoIterator for StandaloneStrategy {
    type Item = rewrite::results::NetabaseResult<StandaloneStrategyChunk>;
    type IntoIter = std::vec::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        rewrite::traits::structural::blob::NetabaseBlobItem::into_chunks_iter(
            self,
            rewrite::traits::structural::blob::ChunkSize::Default,
        )
    }
}
mod tests {
    use super::*;
    extern crate test;
    #[rustc_test_marker = "tests::test_partial_field_blob"]
    #[doc(hidden)]
    pub const test_partial_field_blob: test::TestDescAndFn = test::TestDescAndFn {
        desc: test::TestDesc {
            name: test::StaticTestName("tests::test_partial_field_blob"),
            ignore: false,
            ignore_message: ::core::option::Option::None,
            source_file: "tests/blob_comprehensive_test.rs",
            start_line: 95usize,
            start_col: 8usize,
            end_line: 95usize,
            end_col: 31usize,
            compile_fail: false,
            no_run: false,
            should_panic: test::ShouldPanic::No,
            test_type: test::TestType::IntegrationTest,
        },
        testfn: test::StaticTestFn(
            #[coverage(off)]
            || test::assert_test_result(test_partial_field_blob()),
        ),
    };
    fn test_partial_field_blob() {
        let blob = PartialFieldBlob {
            header: "Short Header".to_string(),
            payload: ::alloc::vec::from_elem(0u8, 500),
        };
        let chunks: Vec<_> = blob.clone().into_chunks(ChunkSize::Default).collect();
        match (&chunks.len(), &3) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    let kind = ::core::panicking::AssertKind::Eq;
                    ::core::panicking::assert_failed(
                        kind,
                        &*left_val,
                        &*right_val,
                        ::core::option::Option::None,
                    );
                }
            }
        };
        let mut header_chunks = 0;
        let mut payload_chunks = 0;
        for chunk in &chunks {
            match chunk {
                PartialFieldBlobChunk::Header(_) => header_chunks += 1,
                PartialFieldBlobChunk::Payload(_) => payload_chunks += 1,
                _ => {
                    ::core::panicking::panic_fmt(
                        format_args!("Unexpected chunk variant"),
                    );
                }
            }
        }
        match (&header_chunks, &1) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    let kind = ::core::panicking::AssertKind::Eq;
                    ::core::panicking::assert_failed(
                        kind,
                        &*left_val,
                        &*right_val,
                        ::core::option::Option::None,
                    );
                }
            }
        };
        match (&payload_chunks, &2) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    let kind = ::core::panicking::AssertKind::Eq;
                    ::core::panicking::assert_failed(
                        kind,
                        &*left_val,
                        &*right_val,
                        ::core::option::Option::None,
                    );
                }
            }
        };
        let reconstructed = PartialFieldBlob::try_from_chunks(
                chunks.into_iter(),
                ChunkSize::Default,
            )
            .expect("Failed to reconstruct PartialFieldBlob");
        match (&blob, &reconstructed) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    let kind = ::core::panicking::AssertKind::Eq;
                    ::core::panicking::assert_failed(
                        kind,
                        &*left_val,
                        &*right_val,
                        ::core::option::Option::None,
                    );
                }
            }
        };
    }
    extern crate test;
    #[rustc_test_marker = "tests::test_nested_blob_partial"]
    #[doc(hidden)]
    pub const test_nested_blob_partial: test::TestDescAndFn = test::TestDescAndFn {
        desc: test::TestDesc {
            name: test::StaticTestName("tests::test_nested_blob_partial"),
            ignore: false,
            ignore_message: ::core::option::Option::None,
            source_file: "tests/blob_comprehensive_test.rs",
            start_line: 130usize,
            start_col: 8usize,
            end_line: 130usize,
            end_col: 32usize,
            compile_fail: false,
            no_run: false,
            should_panic: test::ShouldPanic::No,
            test_type: test::TestType::IntegrationTest,
        },
        testfn: test::StaticTestFn(
            #[coverage(off)]
            || test::assert_test_result(test_nested_blob_partial()),
        ),
    };
    fn test_nested_blob_partial() {
        let blob = ParentBlob {
            child: NestedBlob { id: 12345 },
        };
        let chunks: Vec<_> = blob.clone().into_chunks(ChunkSize::Default).collect();
        if !!chunks.is_empty() {
            ::core::panicking::panic("assertion failed: !chunks.is_empty()")
        }
        let reconstructed = ParentBlob::try_from_chunks(
                chunks.into_iter(),
                ChunkSize::Default,
            )
            .expect("Failed to reconstruct ParentBlob");
        match (&blob, &reconstructed) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    let kind = ::core::panicking::AssertKind::Eq;
                    ::core::panicking::assert_failed(
                        kind,
                        &*left_val,
                        &*right_val,
                        ::core::option::Option::None,
                    );
                }
            }
        };
    }
    extern crate test;
    #[rustc_test_marker = "tests::test_full_enum_blob"]
    #[doc(hidden)]
    pub const test_full_enum_blob: test::TestDescAndFn = test::TestDescAndFn {
        desc: test::TestDesc {
            name: test::StaticTestName("tests::test_full_enum_blob"),
            ignore: false,
            ignore_message: ::core::option::Option::None,
            source_file: "tests/blob_comprehensive_test.rs",
            start_line: 144usize,
            start_col: 8usize,
            end_line: 144usize,
            end_col: 27usize,
            compile_fail: false,
            no_run: false,
            should_panic: test::ShouldPanic::No,
            test_type: test::TestType::IntegrationTest,
        },
        testfn: test::StaticTestFn(
            #[coverage(off)]
            || test::assert_test_result(test_full_enum_blob()),
        ),
    };
    fn test_full_enum_blob() {
        let blob = EnumBlob::VariantB { x: 1, y: 2 };
        let chunks: Vec<_> = blob.clone().into_chunks(ChunkSize::Size(128)).collect();
        if !!chunks.is_empty() {
            ::core::panicking::panic("assertion failed: !chunks.is_empty()")
        }
        let reconstructed = EnumBlob::try_from_chunks(
                chunks.into_iter(),
                ChunkSize::Size(128),
            )
            .expect("Failed to reconstruct EnumBlob");
        match (&blob, &reconstructed) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    let kind = ::core::panicking::AssertKind::Eq;
                    ::core::panicking::assert_failed(
                        kind,
                        &*left_val,
                        &*right_val,
                        ::core::option::Option::None,
                    );
                }
            }
        };
    }
    extern crate test;
    #[rustc_test_marker = "tests::test_error_corrupted_chunk_size"]
    #[doc(hidden)]
    pub const test_error_corrupted_chunk_size: test::TestDescAndFn = test::TestDescAndFn {
        desc: test::TestDesc {
            name: test::StaticTestName("tests::test_error_corrupted_chunk_size"),
            ignore: false,
            ignore_message: ::core::option::Option::None,
            source_file: "tests/blob_comprehensive_test.rs",
            start_line: 156usize,
            start_col: 8usize,
            end_line: 156usize,
            end_col: 39usize,
            compile_fail: false,
            no_run: false,
            should_panic: test::ShouldPanic::No,
            test_type: test::TestType::IntegrationTest,
        },
        testfn: test::StaticTestFn(
            #[coverage(off)]
            || test::assert_test_result(test_error_corrupted_chunk_size()),
        ),
    };
    fn test_error_corrupted_chunk_size() {
        struct Simple {
            data: Vec<u8>,
        }
        #[automatically_derived]
        ///An archived [`Simple`]
        #[bytecheck(crate = ::rkyv::bytecheck)]
        #[repr(C)]
        struct ArchivedSimple
        where
            Vec<u8>: ::rkyv::Archive,
        {
            ///The archived counterpart of [`Simple::data`]
            data: <Vec<u8> as ::rkyv::Archive>::Archived,
        }
        #[automatically_derived]
        unsafe impl<
            __C: ::rkyv::bytecheck::rancor::Fallible + ?::core::marker::Sized,
        > ::rkyv::bytecheck::CheckBytes<__C> for ArchivedSimple
        where
            Vec<u8>: ::rkyv::Archive,
            <__C as ::rkyv::bytecheck::rancor::Fallible>::Error: ::rkyv::bytecheck::rancor::Trace,
            <Vec<u8> as ::rkyv::Archive>::Archived: ::rkyv::bytecheck::CheckBytes<__C>,
        {
            unsafe fn check_bytes(
                value: *const Self,
                context: &mut __C,
            ) -> ::core::result::Result<
                (),
                <__C as ::rkyv::bytecheck::rancor::Fallible>::Error,
            > {
                <<Vec<
                    u8,
                > as ::rkyv::Archive>::Archived as ::rkyv::bytecheck::CheckBytes<
                    __C,
                >>::check_bytes(&raw const (*value).data, context)
                    .map_err(|e| {
                        <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Trace>::trace(
                            e,
                            ::rkyv::bytecheck::StructCheckContext {
                                struct_name: "ArchivedSimple",
                                field_name: "data",
                            },
                        )
                    })?;
                ::core::result::Result::Ok(())
            }
        }
        #[automatically_derived]
        ///The resolver for an archived [`Simple`]
        struct SimpleResolver
        where
            Vec<u8>: ::rkyv::Archive,
        {
            data: <Vec<u8> as ::rkyv::Archive>::Resolver,
        }
        impl ::rkyv::Archive for Simple
        where
            Vec<u8>: ::rkyv::Archive,
        {
            type Archived = ArchivedSimple;
            type Resolver = SimpleResolver;
            const COPY_OPTIMIZATION: ::rkyv::traits::CopyOptimization<Self> = unsafe {
                ::rkyv::traits::CopyOptimization::enable_if(
                    0 + ::core::mem::size_of::<Vec<u8>>()
                        == ::core::mem::size_of::<Simple>()
                        && <Vec<u8> as ::rkyv::Archive>::COPY_OPTIMIZATION.is_enabled()
                        && const { builtin # offset_of(Simple, data) }
                            == const { builtin # offset_of(ArchivedSimple, data) },
                )
            };
            #[allow(clippy::unit_arg)]
            fn resolve(
                &self,
                resolver: Self::Resolver,
                out: ::rkyv::Place<Self::Archived>,
            ) {
                let field_ptr = unsafe { &raw mut (*out.ptr()).data };
                let field_out = unsafe {
                    ::rkyv::Place::from_field_unchecked(out, field_ptr)
                };
                <Vec<
                    u8,
                > as ::rkyv::Archive>::resolve(&self.data, resolver.data, field_out);
            }
        }
        unsafe impl ::rkyv::traits::Portable for ArchivedSimple
        where
            Vec<u8>: ::rkyv::Archive,
            <Vec<u8> as ::rkyv::Archive>::Archived: ::rkyv::traits::Portable,
        {}
        #[automatically_derived]
        impl<__S: ::rkyv::rancor::Fallible + ?Sized> ::rkyv::Serialize<__S> for Simple
        where
            Vec<u8>: ::rkyv::Serialize<__S>,
        {
            fn serialize(
                &self,
                serializer: &mut __S,
            ) -> ::core::result::Result<
                <Self as ::rkyv::Archive>::Resolver,
                <__S as ::rkyv::rancor::Fallible>::Error,
            > {
                let __this = self;
                ::core::result::Result::Ok(SimpleResolver {
                    data: <Vec<
                        u8,
                    > as ::rkyv::Serialize<__S>>::serialize(&__this.data, serializer)?,
                })
            }
        }
        #[automatically_derived]
        impl<__D: ::rkyv::rancor::Fallible + ?Sized> ::rkyv::Deserialize<Simple, __D>
        for ::rkyv::Archived<Simple>
        where
            Vec<u8>: ::rkyv::Archive,
            <Vec<u8> as ::rkyv::Archive>::Archived: ::rkyv::Deserialize<Vec<u8>, __D>,
        {
            fn deserialize(
                &self,
                deserializer: &mut __D,
            ) -> ::core::result::Result<
                Simple,
                <__D as ::rkyv::rancor::Fallible>::Error,
            > {
                let __this = self;
                ::core::result::Result::Ok(Simple {
                    data: <<Vec<
                        u8,
                    > as ::rkyv::Archive>::Archived as ::rkyv::Deserialize<
                        Vec<u8>,
                        __D,
                    >>::deserialize(&__this.data, deserializer)?,
                })
            }
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for Simple {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field1_finish(
                    f,
                    "Simple",
                    "data",
                    &&self.data,
                )
            }
        }
        #[automatically_derived]
        impl ::core::clone::Clone for Simple {
            #[inline]
            fn clone(&self) -> Simple {
                Simple {
                    data: ::core::clone::Clone::clone(&self.data),
                }
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq for Simple {}
        #[automatically_derived]
        impl ::core::cmp::PartialEq for Simple {
            #[inline]
            fn eq(&self, other: &Simple) -> bool {
                self.data == other.data
            }
        }
        #[automatically_derived]
        impl ::core::cmp::Eq for Simple {
            #[inline]
            #[doc(hidden)]
            #[coverage(off)]
            fn assert_fields_are_eq(&self) {
                let _: ::core::cmp::AssertParamIsEq<Vec<u8>>;
            }
        }
        pub struct SimpleChunk {
            pub index: usize,
            pub data: Vec<u8>,
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for SimpleChunk {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field2_finish(
                    f,
                    "SimpleChunk",
                    "index",
                    &self.index,
                    "data",
                    &&self.data,
                )
            }
        }
        #[automatically_derived]
        impl ::core::clone::Clone for SimpleChunk {
            #[inline]
            fn clone(&self) -> SimpleChunk {
                SimpleChunk {
                    index: ::core::clone::Clone::clone(&self.index),
                    data: ::core::clone::Clone::clone(&self.data),
                }
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq for SimpleChunk {}
        #[automatically_derived]
        impl ::core::cmp::PartialEq for SimpleChunk {
            #[inline]
            fn eq(&self, other: &SimpleChunk) -> bool {
                self.index == other.index && self.data == other.data
            }
        }
        #[automatically_derived]
        impl ::core::cmp::Eq for SimpleChunk {
            #[inline]
            #[doc(hidden)]
            #[coverage(off)]
            fn assert_fields_are_eq(&self) {
                let _: ::core::cmp::AssertParamIsEq<usize>;
                let _: ::core::cmp::AssertParamIsEq<Vec<u8>>;
            }
        }
        #[automatically_derived]
        impl ::core::cmp::PartialOrd for SimpleChunk {
            #[inline]
            fn partial_cmp(
                &self,
                other: &SimpleChunk,
            ) -> ::core::option::Option<::core::cmp::Ordering> {
                match ::core::cmp::PartialOrd::partial_cmp(&self.index, &other.index) {
                    ::core::option::Option::Some(::core::cmp::Ordering::Equal) => {
                        ::core::cmp::PartialOrd::partial_cmp(&self.data, &other.data)
                    }
                    cmp => cmp,
                }
            }
        }
        #[automatically_derived]
        impl ::core::cmp::Ord for SimpleChunk {
            #[inline]
            fn cmp(&self, other: &SimpleChunk) -> ::core::cmp::Ordering {
                match ::core::cmp::Ord::cmp(&self.index, &other.index) {
                    ::core::cmp::Ordering::Equal => {
                        ::core::cmp::Ord::cmp(&self.data, &other.data)
                    }
                    cmp => cmp,
                }
            }
        }
        #[automatically_derived]
        ///An archived [`SimpleChunk`]
        #[bytecheck(crate = ::rkyv::bytecheck)]
        #[repr(C)]
        pub struct ArchivedSimpleChunk
        where
            usize: ::rkyv::Archive,
            Vec<u8>: ::rkyv::Archive,
        {
            ///The archived counterpart of [`SimpleChunk::index`]
            pub index: <usize as ::rkyv::Archive>::Archived,
            ///The archived counterpart of [`SimpleChunk::data`]
            pub data: <Vec<u8> as ::rkyv::Archive>::Archived,
        }
        #[automatically_derived]
        unsafe impl<
            __C: ::rkyv::bytecheck::rancor::Fallible + ?::core::marker::Sized,
        > ::rkyv::bytecheck::CheckBytes<__C> for ArchivedSimpleChunk
        where
            usize: ::rkyv::Archive,
            Vec<u8>: ::rkyv::Archive,
            <__C as ::rkyv::bytecheck::rancor::Fallible>::Error: ::rkyv::bytecheck::rancor::Trace,
            <usize as ::rkyv::Archive>::Archived: ::rkyv::bytecheck::CheckBytes<__C>,
            <Vec<u8> as ::rkyv::Archive>::Archived: ::rkyv::bytecheck::CheckBytes<__C>,
        {
            unsafe fn check_bytes(
                value: *const Self,
                context: &mut __C,
            ) -> ::core::result::Result<
                (),
                <__C as ::rkyv::bytecheck::rancor::Fallible>::Error,
            > {
                <<usize as ::rkyv::Archive>::Archived as ::rkyv::bytecheck::CheckBytes<
                    __C,
                >>::check_bytes(&raw const (*value).index, context)
                    .map_err(|e| {
                        <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Trace>::trace(
                            e,
                            ::rkyv::bytecheck::StructCheckContext {
                                struct_name: "ArchivedSimpleChunk",
                                field_name: "index",
                            },
                        )
                    })?;
                <<Vec<
                    u8,
                > as ::rkyv::Archive>::Archived as ::rkyv::bytecheck::CheckBytes<
                    __C,
                >>::check_bytes(&raw const (*value).data, context)
                    .map_err(|e| {
                        <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Trace>::trace(
                            e,
                            ::rkyv::bytecheck::StructCheckContext {
                                struct_name: "ArchivedSimpleChunk",
                                field_name: "data",
                            },
                        )
                    })?;
                ::core::result::Result::Ok(())
            }
        }
        #[automatically_derived]
        ///The resolver for an archived [`SimpleChunk`]
        pub struct SimpleChunkResolver
        where
            usize: ::rkyv::Archive,
            Vec<u8>: ::rkyv::Archive,
        {
            index: <usize as ::rkyv::Archive>::Resolver,
            data: <Vec<u8> as ::rkyv::Archive>::Resolver,
        }
        impl ::rkyv::Archive for SimpleChunk
        where
            usize: ::rkyv::Archive,
            Vec<u8>: ::rkyv::Archive,
        {
            type Archived = ArchivedSimpleChunk;
            type Resolver = SimpleChunkResolver;
            const COPY_OPTIMIZATION: ::rkyv::traits::CopyOptimization<Self> = unsafe {
                ::rkyv::traits::CopyOptimization::enable_if(
                    0 + ::core::mem::size_of::<usize>()
                        + ::core::mem::size_of::<Vec<u8>>()
                        == ::core::mem::size_of::<SimpleChunk>()
                        && <usize as ::rkyv::Archive>::COPY_OPTIMIZATION.is_enabled()
                        && const { builtin # offset_of(SimpleChunk, index) }
                            == const { builtin # offset_of(ArchivedSimpleChunk, index) }
                        && <Vec<u8> as ::rkyv::Archive>::COPY_OPTIMIZATION.is_enabled()
                        && const { builtin # offset_of(SimpleChunk, data) }
                            == const { builtin # offset_of(ArchivedSimpleChunk, data) },
                )
            };
            #[allow(clippy::unit_arg)]
            fn resolve(
                &self,
                resolver: Self::Resolver,
                out: ::rkyv::Place<Self::Archived>,
            ) {
                let field_ptr = unsafe { &raw mut (*out.ptr()).index };
                let field_out = unsafe {
                    ::rkyv::Place::from_field_unchecked(out, field_ptr)
                };
                <usize as ::rkyv::Archive>::resolve(
                    &self.index,
                    resolver.index,
                    field_out,
                );
                let field_ptr = unsafe { &raw mut (*out.ptr()).data };
                let field_out = unsafe {
                    ::rkyv::Place::from_field_unchecked(out, field_ptr)
                };
                <Vec<
                    u8,
                > as ::rkyv::Archive>::resolve(&self.data, resolver.data, field_out);
            }
        }
        unsafe impl ::rkyv::traits::Portable for ArchivedSimpleChunk
        where
            usize: ::rkyv::Archive,
            Vec<u8>: ::rkyv::Archive,
            <usize as ::rkyv::Archive>::Archived: ::rkyv::traits::Portable,
            <Vec<u8> as ::rkyv::Archive>::Archived: ::rkyv::traits::Portable,
        {}
        #[automatically_derived]
        impl<__S: ::rkyv::rancor::Fallible + ?Sized> ::rkyv::Serialize<__S>
        for SimpleChunk
        where
            usize: ::rkyv::Serialize<__S>,
            Vec<u8>: ::rkyv::Serialize<__S>,
        {
            fn serialize(
                &self,
                serializer: &mut __S,
            ) -> ::core::result::Result<
                <Self as ::rkyv::Archive>::Resolver,
                <__S as ::rkyv::rancor::Fallible>::Error,
            > {
                let __this = self;
                ::core::result::Result::Ok(SimpleChunkResolver {
                    index: <usize as ::rkyv::Serialize<
                        __S,
                    >>::serialize(&__this.index, serializer)?,
                    data: <Vec<
                        u8,
                    > as ::rkyv::Serialize<__S>>::serialize(&__this.data, serializer)?,
                })
            }
        }
        #[automatically_derived]
        impl<
            __D: ::rkyv::rancor::Fallible + ?Sized,
        > ::rkyv::Deserialize<SimpleChunk, __D> for ::rkyv::Archived<SimpleChunk>
        where
            usize: ::rkyv::Archive,
            <usize as ::rkyv::Archive>::Archived: ::rkyv::Deserialize<usize, __D>,
            Vec<u8>: ::rkyv::Archive,
            <Vec<u8> as ::rkyv::Archive>::Archived: ::rkyv::Deserialize<Vec<u8>, __D>,
        {
            fn deserialize(
                &self,
                deserializer: &mut __D,
            ) -> ::core::result::Result<
                SimpleChunk,
                <__D as ::rkyv::rancor::Fallible>::Error,
            > {
                let __this = self;
                ::core::result::Result::Ok(SimpleChunk {
                    index: <<usize as ::rkyv::Archive>::Archived as ::rkyv::Deserialize<
                        usize,
                        __D,
                    >>::deserialize(&__this.index, deserializer)?,
                    data: <<Vec<
                        u8,
                    > as ::rkyv::Archive>::Archived as ::rkyv::Deserialize<
                        Vec<u8>,
                        __D,
                    >>::deserialize(&__this.data, deserializer)?,
                })
            }
        }
        impl ::rewrite::traits::structural::blob::BlobItemChunk for SimpleChunk {
            type Index = usize;
            fn get_index(&self) -> &Self::Index {
                &self.index
            }
        }
        pub enum SimpleChunkFill {
            Full(usize),
            Partial(usize),
            Corrupted(usize),
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for SimpleChunkFill {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                match self {
                    SimpleChunkFill::Full(__self_0) => {
                        ::core::fmt::Formatter::debug_tuple_field1_finish(
                            f,
                            "Full",
                            &__self_0,
                        )
                    }
                    SimpleChunkFill::Partial(__self_0) => {
                        ::core::fmt::Formatter::debug_tuple_field1_finish(
                            f,
                            "Partial",
                            &__self_0,
                        )
                    }
                    SimpleChunkFill::Corrupted(__self_0) => {
                        ::core::fmt::Formatter::debug_tuple_field1_finish(
                            f,
                            "Corrupted",
                            &__self_0,
                        )
                    }
                }
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        unsafe impl ::core::clone::TrivialClone for SimpleChunkFill {}
        #[automatically_derived]
        impl ::core::clone::Clone for SimpleChunkFill {
            #[inline]
            fn clone(&self) -> SimpleChunkFill {
                let _: ::core::clone::AssertParamIsClone<usize>;
                *self
            }
        }
        #[automatically_derived]
        impl ::core::marker::Copy for SimpleChunkFill {}
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq for SimpleChunkFill {}
        #[automatically_derived]
        impl ::core::cmp::PartialEq for SimpleChunkFill {
            #[inline]
            fn eq(&self, other: &SimpleChunkFill) -> bool {
                let __self_discr = ::core::intrinsics::discriminant_value(self);
                let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                __self_discr == __arg1_discr
                    && match (self, other) {
                        (
                            SimpleChunkFill::Full(__self_0),
                            SimpleChunkFill::Full(__arg1_0),
                        ) => __self_0 == __arg1_0,
                        (
                            SimpleChunkFill::Partial(__self_0),
                            SimpleChunkFill::Partial(__arg1_0),
                        ) => __self_0 == __arg1_0,
                        (
                            SimpleChunkFill::Corrupted(__self_0),
                            SimpleChunkFill::Corrupted(__arg1_0),
                        ) => __self_0 == __arg1_0,
                        _ => unsafe { ::core::intrinsics::unreachable() }
                    }
            }
        }
        #[automatically_derived]
        impl ::core::cmp::Eq for SimpleChunkFill {
            #[inline]
            #[doc(hidden)]
            #[coverage(off)]
            fn assert_fields_are_eq(&self) {
                let _: ::core::cmp::AssertParamIsEq<usize>;
            }
        }
        #[automatically_derived]
        impl ::core::cmp::PartialOrd for SimpleChunkFill {
            #[inline]
            fn partial_cmp(
                &self,
                other: &SimpleChunkFill,
            ) -> ::core::option::Option<::core::cmp::Ordering> {
                let __self_discr = ::core::intrinsics::discriminant_value(self);
                let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                match (self, other) {
                    (
                        SimpleChunkFill::Full(__self_0),
                        SimpleChunkFill::Full(__arg1_0),
                    ) => ::core::cmp::PartialOrd::partial_cmp(__self_0, __arg1_0),
                    (
                        SimpleChunkFill::Partial(__self_0),
                        SimpleChunkFill::Partial(__arg1_0),
                    ) => ::core::cmp::PartialOrd::partial_cmp(__self_0, __arg1_0),
                    (
                        SimpleChunkFill::Corrupted(__self_0),
                        SimpleChunkFill::Corrupted(__arg1_0),
                    ) => ::core::cmp::PartialOrd::partial_cmp(__self_0, __arg1_0),
                    _ => {
                        ::core::cmp::PartialOrd::partial_cmp(
                            &__self_discr,
                            &__arg1_discr,
                        )
                    }
                }
            }
        }
        #[automatically_derived]
        impl ::core::cmp::Ord for SimpleChunkFill {
            #[inline]
            fn cmp(&self, other: &SimpleChunkFill) -> ::core::cmp::Ordering {
                let __self_discr = ::core::intrinsics::discriminant_value(self);
                let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                match ::core::cmp::Ord::cmp(&__self_discr, &__arg1_discr) {
                    ::core::cmp::Ordering::Equal => {
                        match (self, other) {
                            (
                                SimpleChunkFill::Full(__self_0),
                                SimpleChunkFill::Full(__arg1_0),
                            ) => ::core::cmp::Ord::cmp(__self_0, __arg1_0),
                            (
                                SimpleChunkFill::Partial(__self_0),
                                SimpleChunkFill::Partial(__arg1_0),
                            ) => ::core::cmp::Ord::cmp(__self_0, __arg1_0),
                            (
                                SimpleChunkFill::Corrupted(__self_0),
                                SimpleChunkFill::Corrupted(__arg1_0),
                            ) => ::core::cmp::Ord::cmp(__self_0, __arg1_0),
                            _ => unsafe { ::core::intrinsics::unreachable() }
                        }
                    }
                    cmp => cmp,
                }
            }
        }
        #[automatically_derived]
        ///An archived [`SimpleChunkFill`]
        #[bytecheck(crate = ::rkyv::bytecheck)]
        #[repr(u8)]
        pub enum ArchivedSimpleChunkFill
        where
            usize: ::rkyv::Archive,
            usize: ::rkyv::Archive,
            usize: ::rkyv::Archive,
        {
            ///The archived counterpart of [`SimpleChunkFill::Full`]
            #[allow(dead_code)]
            Full(
                ///The archived counterpart of [`SimpleChunkFill::Full::0`]
                <usize as ::rkyv::Archive>::Archived,
            ),
            ///The archived counterpart of [`SimpleChunkFill::Partial`]
            #[allow(dead_code)]
            Partial(
                ///The archived counterpart of [`SimpleChunkFill::Partial::0`]
                <usize as ::rkyv::Archive>::Archived,
            ),
            ///The archived counterpart of [`SimpleChunkFill::Corrupted`]
            #[allow(dead_code)]
            Corrupted(
                ///The archived counterpart of [`SimpleChunkFill::Corrupted::0`]
                <usize as ::rkyv::Archive>::Archived,
            ),
        }
        const _: () = {
            #[repr(u8)]
            enum Tag {
                Full,
                Partial,
                Corrupted,
            }
            struct Discriminant;
            #[automatically_derived]
            impl Discriminant {
                #[allow(non_upper_case_globals)]
                const Full: u8 = Tag::Full as u8;
                #[allow(non_upper_case_globals)]
                const Partial: u8 = Tag::Partial as u8;
                #[allow(non_upper_case_globals)]
                const Corrupted: u8 = Tag::Corrupted as u8;
            }
            #[repr(C)]
            struct VariantFull(
                Tag,
                <usize as ::rkyv::Archive>::Archived,
                ::core::marker::PhantomData<ArchivedSimpleChunkFill>,
            )
            where
                usize: ::rkyv::Archive,
                usize: ::rkyv::Archive,
                usize: ::rkyv::Archive;
            #[repr(C)]
            struct VariantPartial(
                Tag,
                <usize as ::rkyv::Archive>::Archived,
                ::core::marker::PhantomData<ArchivedSimpleChunkFill>,
            )
            where
                usize: ::rkyv::Archive,
                usize: ::rkyv::Archive,
                usize: ::rkyv::Archive;
            #[repr(C)]
            struct VariantCorrupted(
                Tag,
                <usize as ::rkyv::Archive>::Archived,
                ::core::marker::PhantomData<ArchivedSimpleChunkFill>,
            )
            where
                usize: ::rkyv::Archive,
                usize: ::rkyv::Archive,
                usize: ::rkyv::Archive;
            #[automatically_derived]
            unsafe impl<
                __C: ::rkyv::bytecheck::rancor::Fallible + ?::core::marker::Sized,
            > ::rkyv::bytecheck::CheckBytes<__C> for ArchivedSimpleChunkFill
            where
                usize: ::rkyv::Archive,
                usize: ::rkyv::Archive,
                usize: ::rkyv::Archive,
                <__C as ::rkyv::bytecheck::rancor::Fallible>::Error: ::rkyv::bytecheck::rancor::Source,
                <usize as ::rkyv::Archive>::Archived: ::rkyv::bytecheck::CheckBytes<__C>,
                <usize as ::rkyv::Archive>::Archived: ::rkyv::bytecheck::CheckBytes<__C>,
                <usize as ::rkyv::Archive>::Archived: ::rkyv::bytecheck::CheckBytes<__C>,
            {
                unsafe fn check_bytes(
                    value: *const Self,
                    context: &mut __C,
                ) -> ::core::result::Result<
                    (),
                    <__C as ::rkyv::bytecheck::rancor::Fallible>::Error,
                > {
                    let tag = *value.cast::<u8>();
                    match tag {
                        Discriminant::Full => {
                            let value = value.cast::<VariantFull>();
                            <<usize as ::rkyv::Archive>::Archived as ::rkyv::bytecheck::CheckBytes<
                                __C,
                            >>::check_bytes(&raw const (*value).1, context)
                                .map_err(|e| {
                                    <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Trace>::trace(
                                        e,
                                        ::rkyv::bytecheck::UnnamedEnumVariantCheckContext {
                                            enum_name: "ArchivedSimpleChunkFill",
                                            variant_name: "Full",
                                            field_index: 1,
                                        },
                                    )
                                })?;
                        }
                        Discriminant::Partial => {
                            let value = value.cast::<VariantPartial>();
                            <<usize as ::rkyv::Archive>::Archived as ::rkyv::bytecheck::CheckBytes<
                                __C,
                            >>::check_bytes(&raw const (*value).1, context)
                                .map_err(|e| {
                                    <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Trace>::trace(
                                        e,
                                        ::rkyv::bytecheck::UnnamedEnumVariantCheckContext {
                                            enum_name: "ArchivedSimpleChunkFill",
                                            variant_name: "Partial",
                                            field_index: 1,
                                        },
                                    )
                                })?;
                        }
                        Discriminant::Corrupted => {
                            let value = value.cast::<VariantCorrupted>();
                            <<usize as ::rkyv::Archive>::Archived as ::rkyv::bytecheck::CheckBytes<
                                __C,
                            >>::check_bytes(&raw const (*value).1, context)
                                .map_err(|e| {
                                    <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Trace>::trace(
                                        e,
                                        ::rkyv::bytecheck::UnnamedEnumVariantCheckContext {
                                            enum_name: "ArchivedSimpleChunkFill",
                                            variant_name: "Corrupted",
                                            field_index: 1,
                                        },
                                    )
                                })?;
                        }
                        _ => {
                            return ::core::result::Result::Err(
                                <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Source>::new(::rkyv::bytecheck::InvalidEnumDiscriminantError {
                                    enum_name: "ArchivedSimpleChunkFill",
                                    invalid_discriminant: tag,
                                }),
                            );
                        }
                    }
                    ::core::result::Result::Ok(())
                }
            }
        };
        #[automatically_derived]
        ///The resolver for an archived [`SimpleChunkFill`]
        pub enum SimpleChunkFillResolver
        where
            usize: ::rkyv::Archive,
            usize: ::rkyv::Archive,
            usize: ::rkyv::Archive,
        {
            ///The resolver for [`SimpleChunkFill::Full`]
            #[allow(dead_code)]
            Full(<usize as ::rkyv::Archive>::Resolver),
            ///The resolver for [`SimpleChunkFill::Partial`]
            #[allow(dead_code)]
            Partial(<usize as ::rkyv::Archive>::Resolver),
            ///The resolver for [`SimpleChunkFill::Corrupted`]
            #[allow(dead_code)]
            Corrupted(<usize as ::rkyv::Archive>::Resolver),
        }
        const _: () = {
            #[repr(u8)]
            enum ArchivedTag {
                Full,
                Partial,
                Corrupted,
            }
            #[automatically_derived]
            impl ::core::marker::StructuralPartialEq for ArchivedTag {}
            #[automatically_derived]
            impl ::core::cmp::PartialEq for ArchivedTag {
                #[inline]
                fn eq(&self, other: &ArchivedTag) -> bool {
                    let __self_discr = ::core::intrinsics::discriminant_value(self);
                    let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                    __self_discr == __arg1_discr
                }
            }
            #[automatically_derived]
            impl ::core::cmp::PartialOrd for ArchivedTag {
                #[inline]
                fn partial_cmp(
                    &self,
                    other: &ArchivedTag,
                ) -> ::core::option::Option<::core::cmp::Ordering> {
                    let __self_discr = ::core::intrinsics::discriminant_value(self);
                    let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                    ::core::cmp::PartialOrd::partial_cmp(&__self_discr, &__arg1_discr)
                }
            }
            #[repr(C)]
            struct ArchivedVariantFull(
                ArchivedTag,
                <usize as ::rkyv::Archive>::Archived,
                ::core::marker::PhantomData<SimpleChunkFill>,
            )
            where
                usize: ::rkyv::Archive,
                usize: ::rkyv::Archive,
                usize: ::rkyv::Archive;
            #[repr(C)]
            struct ArchivedVariantPartial(
                ArchivedTag,
                <usize as ::rkyv::Archive>::Archived,
                ::core::marker::PhantomData<SimpleChunkFill>,
            )
            where
                usize: ::rkyv::Archive,
                usize: ::rkyv::Archive,
                usize: ::rkyv::Archive;
            #[repr(C)]
            struct ArchivedVariantCorrupted(
                ArchivedTag,
                <usize as ::rkyv::Archive>::Archived,
                ::core::marker::PhantomData<SimpleChunkFill>,
            )
            where
                usize: ::rkyv::Archive,
                usize: ::rkyv::Archive,
                usize: ::rkyv::Archive;
            impl ::rkyv::Archive for SimpleChunkFill
            where
                usize: ::rkyv::Archive,
                usize: ::rkyv::Archive,
                usize: ::rkyv::Archive,
            {
                type Archived = ArchivedSimpleChunkFill;
                type Resolver = SimpleChunkFillResolver;
                #[allow(clippy::unit_arg)]
                fn resolve(
                    &self,
                    resolver: <Self as ::rkyv::Archive>::Resolver,
                    out: ::rkyv::Place<<Self as ::rkyv::Archive>::Archived>,
                ) {
                    let __this = self;
                    match resolver {
                        SimpleChunkFillResolver::Full(resolver_0) => {
                            match __this {
                                SimpleChunkFill::Full(self_0, ..) => {
                                    let out = unsafe {
                                        out.cast_unchecked::<ArchivedVariantFull>()
                                    };
                                    let tag_ptr = unsafe { &raw mut (*out.ptr()).0 };
                                    unsafe {
                                        tag_ptr.write(ArchivedTag::Full);
                                    }
                                    let field_ptr = unsafe { &raw mut (*out.ptr()).1 };
                                    let out_field = unsafe {
                                        ::rkyv::Place::from_field_unchecked(out, field_ptr)
                                    };
                                    <usize as ::rkyv::Archive>::resolve(
                                        self_0,
                                        resolver_0,
                                        out_field,
                                    );
                                }
                                #[allow(unreachable_patterns)]
                                _ => unsafe { ::core::hint::unreachable_unchecked() }
                            }
                        }
                        SimpleChunkFillResolver::Partial(resolver_0) => {
                            match __this {
                                SimpleChunkFill::Partial(self_0, ..) => {
                                    let out = unsafe {
                                        out.cast_unchecked::<ArchivedVariantPartial>()
                                    };
                                    let tag_ptr = unsafe { &raw mut (*out.ptr()).0 };
                                    unsafe {
                                        tag_ptr.write(ArchivedTag::Partial);
                                    }
                                    let field_ptr = unsafe { &raw mut (*out.ptr()).1 };
                                    let out_field = unsafe {
                                        ::rkyv::Place::from_field_unchecked(out, field_ptr)
                                    };
                                    <usize as ::rkyv::Archive>::resolve(
                                        self_0,
                                        resolver_0,
                                        out_field,
                                    );
                                }
                                #[allow(unreachable_patterns)]
                                _ => unsafe { ::core::hint::unreachable_unchecked() }
                            }
                        }
                        SimpleChunkFillResolver::Corrupted(resolver_0) => {
                            match __this {
                                SimpleChunkFill::Corrupted(self_0, ..) => {
                                    let out = unsafe {
                                        out.cast_unchecked::<ArchivedVariantCorrupted>()
                                    };
                                    let tag_ptr = unsafe { &raw mut (*out.ptr()).0 };
                                    unsafe {
                                        tag_ptr.write(ArchivedTag::Corrupted);
                                    }
                                    let field_ptr = unsafe { &raw mut (*out.ptr()).1 };
                                    let out_field = unsafe {
                                        ::rkyv::Place::from_field_unchecked(out, field_ptr)
                                    };
                                    <usize as ::rkyv::Archive>::resolve(
                                        self_0,
                                        resolver_0,
                                        out_field,
                                    );
                                }
                                #[allow(unreachable_patterns)]
                                _ => unsafe { ::core::hint::unreachable_unchecked() }
                            }
                        }
                    }
                }
            }
        };
        unsafe impl ::rkyv::traits::Portable for ArchivedSimpleChunkFill
        where
            usize: ::rkyv::Archive,
            usize: ::rkyv::Archive,
            usize: ::rkyv::Archive,
            <usize as ::rkyv::Archive>::Archived: ::rkyv::traits::Portable,
            <usize as ::rkyv::Archive>::Archived: ::rkyv::traits::Portable,
            <usize as ::rkyv::Archive>::Archived: ::rkyv::traits::Portable,
        {}
        #[automatically_derived]
        impl<__S: ::rkyv::rancor::Fallible + ?Sized> ::rkyv::Serialize<__S>
        for SimpleChunkFill
        where
            usize: ::rkyv::Serialize<__S>,
            usize: ::rkyv::Serialize<__S>,
            usize: ::rkyv::Serialize<__S>,
        {
            fn serialize(
                &self,
                serializer: &mut __S,
            ) -> ::core::result::Result<
                <Self as ::rkyv::Archive>::Resolver,
                <__S as ::rkyv::rancor::Fallible>::Error,
            > {
                let __this = self;
                ::core::result::Result::Ok(
                    match __this {
                        SimpleChunkFill::Full(_0, ..) => {
                            SimpleChunkFillResolver::Full(
                                <usize as ::rkyv::Serialize<
                                    __S,
                                >>::serialize(_0, serializer)?,
                            )
                        }
                        SimpleChunkFill::Partial(_0, ..) => {
                            SimpleChunkFillResolver::Partial(
                                <usize as ::rkyv::Serialize<
                                    __S,
                                >>::serialize(_0, serializer)?,
                            )
                        }
                        SimpleChunkFill::Corrupted(_0, ..) => {
                            SimpleChunkFillResolver::Corrupted(
                                <usize as ::rkyv::Serialize<
                                    __S,
                                >>::serialize(_0, serializer)?,
                            )
                        }
                    },
                )
            }
        }
        #[automatically_derived]
        impl<
            __D: ::rkyv::rancor::Fallible + ?Sized,
        > ::rkyv::Deserialize<SimpleChunkFill, __D> for ::rkyv::Archived<SimpleChunkFill>
        where
            usize: ::rkyv::Archive,
            <usize as ::rkyv::Archive>::Archived: ::rkyv::Deserialize<usize, __D>,
            usize: ::rkyv::Archive,
            <usize as ::rkyv::Archive>::Archived: ::rkyv::Deserialize<usize, __D>,
            usize: ::rkyv::Archive,
            <usize as ::rkyv::Archive>::Archived: ::rkyv::Deserialize<usize, __D>,
        {
            fn deserialize(
                &self,
                deserializer: &mut __D,
            ) -> ::core::result::Result<
                SimpleChunkFill,
                <__D as ::rkyv::rancor::Fallible>::Error,
            > {
                let __this = self;
                ::core::result::Result::Ok(
                    match __this {
                        Self::Full(_0, ..) => {
                            SimpleChunkFill::Full(
                                <<usize as ::rkyv::Archive>::Archived as ::rkyv::Deserialize<
                                    usize,
                                    __D,
                                >>::deserialize(_0, deserializer)?,
                            )
                        }
                        Self::Partial(_0, ..) => {
                            SimpleChunkFill::Partial(
                                <<usize as ::rkyv::Archive>::Archived as ::rkyv::Deserialize<
                                    usize,
                                    __D,
                                >>::deserialize(_0, deserializer)?,
                            )
                        }
                        Self::Corrupted(_0, ..) => {
                            SimpleChunkFill::Corrupted(
                                <<usize as ::rkyv::Archive>::Archived as ::rkyv::Deserialize<
                                    usize,
                                    __D,
                                >>::deserialize(_0, deserializer)?,
                            )
                        }
                    },
                )
            }
        }
        impl SimpleChunkFill {
            pub fn from_size(actual: usize, expected: usize) -> Self {
                if actual == expected {
                    Self::Full(actual)
                } else if actual < expected {
                    Self::Partial(actual)
                } else {
                    Self::Corrupted(actual)
                }
            }
        }
        impl rewrite::traits::structural::blob::NetabaseBlobItem for Simple {
            type Chunk = SimpleChunk;
            type BlobIter = std::vec::IntoIter<
                rewrite::results::NetabaseResult<Self::Chunk>,
            >;
            const DEFAULT_CHUNK_SIZE: usize = 0usize;
            fn into_chunks(
                self,
                size: rewrite::traits::structural::blob::ChunkSize,
            ) -> Box<dyn Iterator<Item = Self::Chunk>> {
                Box::new(self.into_chunks_iter(size).filter_map(|r| r.ok()))
            }
            fn into_chunks_iter(
                self,
                size: rewrite::traits::structural::blob::ChunkSize,
            ) -> Self::BlobIter {
                let serialized_data: rewrite::results::NetabaseResult<Vec<u8>> = (|| -> rewrite::results::NetabaseResult<
                    Vec<u8>,
                > {
                    Ok(
                        rkyv::to_bytes::<rkyv::rancor::Error>(&self)
                            .map_err(|e| rewrite::results::NetabaseError::Serialization(
                                ::alloc::__export::must_use({
                                    ::alloc::fmt::format(
                                        format_args!("rkyv serialization failed: {0:?}", e),
                                    )
                                }),
                            ))?
                            .to_vec(),
                    )
                })();
                let chunk_size = match size {
                    rewrite::traits::structural::blob::ChunkSize::Default => {
                        if Self::DEFAULT_CHUNK_SIZE > 0 {
                            Self::DEFAULT_CHUNK_SIZE
                        } else {
                            1024
                        }
                    }
                    rewrite::traits::structural::blob::ChunkSize::Size(n) => n,
                };
                match serialized_data {
                    Ok(data) => {
                        data.chunks(chunk_size)
                            .enumerate()
                            .map(|(index, chunk_data)| {
                                Ok(Self::Chunk {
                                    index,
                                    data: chunk_data.to_vec(),
                                })
                            })
                            .collect::<Vec<_>>()
                            .into_iter()
                    }
                    Err(e) => {
                        ::alloc::boxed::box_assume_init_into_vec_unsafe(
                                ::alloc::intrinsics::write_box_via_move(
                                    ::alloc::boxed::Box::new_uninit(),
                                    [Err(e)],
                                ),
                            )
                            .into_iter()
                    }
                }
            }
            fn try_from_chunks(
                chunks: impl Iterator<Item = Self::Chunk>,
                size: rewrite::traits::structural::blob::ChunkSize,
            ) -> rewrite::results::NetabaseResult<Self> {
                let mut sorted_chunks: Vec<_> = chunks.collect();
                sorted_chunks.sort_by_key(|c| c.index);
                if sorted_chunks.is_empty() {
                    return Err(
                        rewrite::results::NetabaseError::BlobReconstruction(
                            rewrite::results::BlobReconstructionError::MissingChunks,
                        ),
                    );
                }
                let chunk_size = match size {
                    rewrite::traits::structural::blob::ChunkSize::Default => {
                        if Self::DEFAULT_CHUNK_SIZE > 0 {
                            Self::DEFAULT_CHUNK_SIZE
                        } else {
                            1024
                        }
                    }
                    rewrite::traits::structural::blob::ChunkSize::Size(n) => n,
                };
                let mut missing_details = Vec::new();
                let mut next_expected = 0;
                let max_idx = sorted_chunks.last().map(|c| c.index).unwrap_or(0);
                for chunk in &sorted_chunks {
                    while chunk.index > next_expected {
                        missing_details
                            .push(
                                ::alloc::__export::must_use({
                                    ::alloc::fmt::format(
                                        format_args!(
                                            "{0:?}({{ Index: {1}, Size: {2} }})",
                                            SimpleChunkFill::Full(chunk_size),
                                            next_expected,
                                            chunk_size,
                                        ),
                                    )
                                }),
                            );
                        next_expected += 1;
                    }
                    let fill = SimpleChunkFill::from_size(chunk.data.len(), chunk_size);
                    match fill {
                        SimpleChunkFill::Corrupted(size) => {
                            return Err(
                                rewrite::results::NetabaseError::BlobReconstruction(
                                    rewrite::results::BlobReconstructionError::InvalidChunkData(
                                        ::alloc::__export::must_use({
                                            ::alloc::fmt::format(
                                                format_args!(
                                                    "Corrupted chunk detected: {0:?}({{ Index: {1}, Size: {2} }}). Max allowed size is {3}.",
                                                    fill,
                                                    chunk.index,
                                                    size,
                                                    chunk_size,
                                                ),
                                            )
                                        }),
                                    ),
                                ),
                            );
                        }
                        SimpleChunkFill::Partial(size) if chunk.index < max_idx => {
                            return Err(
                                rewrite::results::NetabaseError::BlobReconstruction(
                                    rewrite::results::BlobReconstructionError::InvalidChunkData(
                                        ::alloc::__export::must_use({
                                            ::alloc::fmt::format(
                                                format_args!(
                                                    "Unexpected partial chunk in middle of stream: {0:?}({{ Index: {1}, Size: {2} }}). Expected {3} bytes.",
                                                    fill,
                                                    chunk.index,
                                                    size,
                                                    chunk_size,
                                                ),
                                            )
                                        }),
                                    ),
                                ),
                            );
                        }
                        _ => {}
                    }
                    if chunk.index == next_expected {
                        next_expected += 1;
                    }
                }
                if !missing_details.is_empty() {
                    if let Some(last) = sorted_chunks.last() {
                        let fill = SimpleChunkFill::from_size(
                            last.data.len(),
                            chunk_size,
                        );
                        if #[allow(non_exhaustive_omitted_patterns)]
                        match fill {
                            SimpleChunkFill::Full(_) => true,
                            _ => false,
                        } {
                            missing_details
                                .push(
                                    ::alloc::__export::must_use({
                                        ::alloc::fmt::format(
                                            format_args!(
                                                "... (Stream truncated: last chunk was Full, expected more data after Index {0})",
                                                last.index,
                                            ),
                                        )
                                    }),
                                );
                        }
                    }
                }
                if !missing_details.is_empty() {
                    return Err(
                        rewrite::results::NetabaseError::BlobReconstruction(
                            rewrite::results::BlobReconstructionError::InvalidChunkData(
                                ::alloc::__export::must_use({
                                    ::alloc::fmt::format(
                                        format_args!(
                                            "Missing chunks: [{0}]. Total chunks present: {1}",
                                            missing_details.join(", "),
                                            sorted_chunks.len(),
                                        ),
                                    )
                                }),
                            ),
                        ),
                    );
                }
                let serialized_data: Vec<u8> = sorted_chunks
                    .into_iter()
                    .flat_map(|c| c.data)
                    .collect();
                Ok(
                    rkyv::from_bytes::<Self, rkyv::rancor::Error>(&serialized_data)
                        .map_err(|e| rewrite::results::NetabaseError::Serialization(
                            ::alloc::__export::must_use({
                                ::alloc::fmt::format(
                                    format_args!("rkyv deserialization failed: {0:?}", e),
                                )
                            }),
                        ))?,
                )
            }
            fn get_blob(&self) -> &Self::Chunk {
                {
                    ::core::panicking::panic_fmt(
                        format_args!(
                            "not implemented: {0}",
                            format_args!("get_blob() requires storing a chunk reference"),
                        ),
                    );
                }
            }
        }
        impl IntoIterator for Simple {
            type Item = rewrite::results::NetabaseResult<SimpleChunk>;
            type IntoIter = std::vec::IntoIter<Self::Item>;
            fn into_iter(self) -> Self::IntoIter {
                rewrite::traits::structural::blob::NetabaseBlobItem::into_chunks_iter(
                    self,
                    rewrite::traits::structural::blob::ChunkSize::Default,
                )
            }
        }
        let blob = Simple {
            data: ::alloc::boxed::box_assume_init_into_vec_unsafe(
                ::alloc::intrinsics::write_box_via_move(
                    ::alloc::boxed::Box::new_uninit(),
                    [1, 2, 3],
                ),
            ),
        };
        let mut chunks: Vec<_> = blob.into_chunks(ChunkSize::Size(64)).collect();
        chunks[0].data.extend(::alloc::vec::from_elem(0u8, 100));
        let result = Simple::try_from_chunks(chunks.into_iter(), ChunkSize::Size(64));
        if !#[allow(non_exhaustive_omitted_patterns)]
        match result {
            Err(
                NetabaseError::BlobReconstruction(
                    BlobReconstructionError::InvalidChunkData(s),
                ),
            ) if s.contains("Corrupted chunk detected") => true,
            _ => false,
        } {
            ::core::panicking::panic(
                "assertion failed: matches!(result,\n    Err(NetabaseError::BlobReconstruction(BlobReconstructionError::InvalidChunkData(s)))\n    if s.contains(\"Corrupted chunk detected\"))",
            )
        }
    }
    extern crate test;
    #[rustc_test_marker = "tests::test_error_partial_chunk_in_middle"]
    #[doc(hidden)]
    pub const test_error_partial_chunk_in_middle: test::TestDescAndFn = test::TestDescAndFn {
        desc: test::TestDesc {
            name: test::StaticTestName("tests::test_error_partial_chunk_in_middle"),
            ignore: false,
            ignore_message: ::core::option::Option::None,
            source_file: "tests/blob_comprehensive_test.rs",
            start_line: 172usize,
            start_col: 8usize,
            end_line: 172usize,
            end_col: 42usize,
            compile_fail: false,
            no_run: false,
            should_panic: test::ShouldPanic::No,
            test_type: test::TestType::IntegrationTest,
        },
        testfn: test::StaticTestFn(
            #[coverage(off)]
            || test::assert_test_result(test_error_partial_chunk_in_middle()),
        ),
    };
    fn test_error_partial_chunk_in_middle() {
        struct Large {
            data: Vec<u8>,
        }
        #[automatically_derived]
        ///An archived [`Large`]
        #[bytecheck(crate = ::rkyv::bytecheck)]
        #[repr(C)]
        struct ArchivedLarge
        where
            Vec<u8>: ::rkyv::Archive,
        {
            ///The archived counterpart of [`Large::data`]
            data: <Vec<u8> as ::rkyv::Archive>::Archived,
        }
        #[automatically_derived]
        unsafe impl<
            __C: ::rkyv::bytecheck::rancor::Fallible + ?::core::marker::Sized,
        > ::rkyv::bytecheck::CheckBytes<__C> for ArchivedLarge
        where
            Vec<u8>: ::rkyv::Archive,
            <__C as ::rkyv::bytecheck::rancor::Fallible>::Error: ::rkyv::bytecheck::rancor::Trace,
            <Vec<u8> as ::rkyv::Archive>::Archived: ::rkyv::bytecheck::CheckBytes<__C>,
        {
            unsafe fn check_bytes(
                value: *const Self,
                context: &mut __C,
            ) -> ::core::result::Result<
                (),
                <__C as ::rkyv::bytecheck::rancor::Fallible>::Error,
            > {
                <<Vec<
                    u8,
                > as ::rkyv::Archive>::Archived as ::rkyv::bytecheck::CheckBytes<
                    __C,
                >>::check_bytes(&raw const (*value).data, context)
                    .map_err(|e| {
                        <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Trace>::trace(
                            e,
                            ::rkyv::bytecheck::StructCheckContext {
                                struct_name: "ArchivedLarge",
                                field_name: "data",
                            },
                        )
                    })?;
                ::core::result::Result::Ok(())
            }
        }
        #[automatically_derived]
        ///The resolver for an archived [`Large`]
        struct LargeResolver
        where
            Vec<u8>: ::rkyv::Archive,
        {
            data: <Vec<u8> as ::rkyv::Archive>::Resolver,
        }
        impl ::rkyv::Archive for Large
        where
            Vec<u8>: ::rkyv::Archive,
        {
            type Archived = ArchivedLarge;
            type Resolver = LargeResolver;
            const COPY_OPTIMIZATION: ::rkyv::traits::CopyOptimization<Self> = unsafe {
                ::rkyv::traits::CopyOptimization::enable_if(
                    0 + ::core::mem::size_of::<Vec<u8>>()
                        == ::core::mem::size_of::<Large>()
                        && <Vec<u8> as ::rkyv::Archive>::COPY_OPTIMIZATION.is_enabled()
                        && const { builtin # offset_of(Large, data) }
                            == const { builtin # offset_of(ArchivedLarge, data) },
                )
            };
            #[allow(clippy::unit_arg)]
            fn resolve(
                &self,
                resolver: Self::Resolver,
                out: ::rkyv::Place<Self::Archived>,
            ) {
                let field_ptr = unsafe { &raw mut (*out.ptr()).data };
                let field_out = unsafe {
                    ::rkyv::Place::from_field_unchecked(out, field_ptr)
                };
                <Vec<
                    u8,
                > as ::rkyv::Archive>::resolve(&self.data, resolver.data, field_out);
            }
        }
        unsafe impl ::rkyv::traits::Portable for ArchivedLarge
        where
            Vec<u8>: ::rkyv::Archive,
            <Vec<u8> as ::rkyv::Archive>::Archived: ::rkyv::traits::Portable,
        {}
        #[automatically_derived]
        impl<__S: ::rkyv::rancor::Fallible + ?Sized> ::rkyv::Serialize<__S> for Large
        where
            Vec<u8>: ::rkyv::Serialize<__S>,
        {
            fn serialize(
                &self,
                serializer: &mut __S,
            ) -> ::core::result::Result<
                <Self as ::rkyv::Archive>::Resolver,
                <__S as ::rkyv::rancor::Fallible>::Error,
            > {
                let __this = self;
                ::core::result::Result::Ok(LargeResolver {
                    data: <Vec<
                        u8,
                    > as ::rkyv::Serialize<__S>>::serialize(&__this.data, serializer)?,
                })
            }
        }
        #[automatically_derived]
        impl<__D: ::rkyv::rancor::Fallible + ?Sized> ::rkyv::Deserialize<Large, __D>
        for ::rkyv::Archived<Large>
        where
            Vec<u8>: ::rkyv::Archive,
            <Vec<u8> as ::rkyv::Archive>::Archived: ::rkyv::Deserialize<Vec<u8>, __D>,
        {
            fn deserialize(
                &self,
                deserializer: &mut __D,
            ) -> ::core::result::Result<
                Large,
                <__D as ::rkyv::rancor::Fallible>::Error,
            > {
                let __this = self;
                ::core::result::Result::Ok(Large {
                    data: <<Vec<
                        u8,
                    > as ::rkyv::Archive>::Archived as ::rkyv::Deserialize<
                        Vec<u8>,
                        __D,
                    >>::deserialize(&__this.data, deserializer)?,
                })
            }
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for Large {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field1_finish(
                    f,
                    "Large",
                    "data",
                    &&self.data,
                )
            }
        }
        #[automatically_derived]
        impl ::core::clone::Clone for Large {
            #[inline]
            fn clone(&self) -> Large {
                Large {
                    data: ::core::clone::Clone::clone(&self.data),
                }
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq for Large {}
        #[automatically_derived]
        impl ::core::cmp::PartialEq for Large {
            #[inline]
            fn eq(&self, other: &Large) -> bool {
                self.data == other.data
            }
        }
        #[automatically_derived]
        impl ::core::cmp::Eq for Large {
            #[inline]
            #[doc(hidden)]
            #[coverage(off)]
            fn assert_fields_are_eq(&self) {
                let _: ::core::cmp::AssertParamIsEq<Vec<u8>>;
            }
        }
        pub struct LargeChunk {
            pub index: usize,
            pub data: Vec<u8>,
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for LargeChunk {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field2_finish(
                    f,
                    "LargeChunk",
                    "index",
                    &self.index,
                    "data",
                    &&self.data,
                )
            }
        }
        #[automatically_derived]
        impl ::core::clone::Clone for LargeChunk {
            #[inline]
            fn clone(&self) -> LargeChunk {
                LargeChunk {
                    index: ::core::clone::Clone::clone(&self.index),
                    data: ::core::clone::Clone::clone(&self.data),
                }
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq for LargeChunk {}
        #[automatically_derived]
        impl ::core::cmp::PartialEq for LargeChunk {
            #[inline]
            fn eq(&self, other: &LargeChunk) -> bool {
                self.index == other.index && self.data == other.data
            }
        }
        #[automatically_derived]
        impl ::core::cmp::Eq for LargeChunk {
            #[inline]
            #[doc(hidden)]
            #[coverage(off)]
            fn assert_fields_are_eq(&self) {
                let _: ::core::cmp::AssertParamIsEq<usize>;
                let _: ::core::cmp::AssertParamIsEq<Vec<u8>>;
            }
        }
        #[automatically_derived]
        impl ::core::cmp::PartialOrd for LargeChunk {
            #[inline]
            fn partial_cmp(
                &self,
                other: &LargeChunk,
            ) -> ::core::option::Option<::core::cmp::Ordering> {
                match ::core::cmp::PartialOrd::partial_cmp(&self.index, &other.index) {
                    ::core::option::Option::Some(::core::cmp::Ordering::Equal) => {
                        ::core::cmp::PartialOrd::partial_cmp(&self.data, &other.data)
                    }
                    cmp => cmp,
                }
            }
        }
        #[automatically_derived]
        impl ::core::cmp::Ord for LargeChunk {
            #[inline]
            fn cmp(&self, other: &LargeChunk) -> ::core::cmp::Ordering {
                match ::core::cmp::Ord::cmp(&self.index, &other.index) {
                    ::core::cmp::Ordering::Equal => {
                        ::core::cmp::Ord::cmp(&self.data, &other.data)
                    }
                    cmp => cmp,
                }
            }
        }
        #[automatically_derived]
        ///An archived [`LargeChunk`]
        #[bytecheck(crate = ::rkyv::bytecheck)]
        #[repr(C)]
        pub struct ArchivedLargeChunk
        where
            usize: ::rkyv::Archive,
            Vec<u8>: ::rkyv::Archive,
        {
            ///The archived counterpart of [`LargeChunk::index`]
            pub index: <usize as ::rkyv::Archive>::Archived,
            ///The archived counterpart of [`LargeChunk::data`]
            pub data: <Vec<u8> as ::rkyv::Archive>::Archived,
        }
        #[automatically_derived]
        unsafe impl<
            __C: ::rkyv::bytecheck::rancor::Fallible + ?::core::marker::Sized,
        > ::rkyv::bytecheck::CheckBytes<__C> for ArchivedLargeChunk
        where
            usize: ::rkyv::Archive,
            Vec<u8>: ::rkyv::Archive,
            <__C as ::rkyv::bytecheck::rancor::Fallible>::Error: ::rkyv::bytecheck::rancor::Trace,
            <usize as ::rkyv::Archive>::Archived: ::rkyv::bytecheck::CheckBytes<__C>,
            <Vec<u8> as ::rkyv::Archive>::Archived: ::rkyv::bytecheck::CheckBytes<__C>,
        {
            unsafe fn check_bytes(
                value: *const Self,
                context: &mut __C,
            ) -> ::core::result::Result<
                (),
                <__C as ::rkyv::bytecheck::rancor::Fallible>::Error,
            > {
                <<usize as ::rkyv::Archive>::Archived as ::rkyv::bytecheck::CheckBytes<
                    __C,
                >>::check_bytes(&raw const (*value).index, context)
                    .map_err(|e| {
                        <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Trace>::trace(
                            e,
                            ::rkyv::bytecheck::StructCheckContext {
                                struct_name: "ArchivedLargeChunk",
                                field_name: "index",
                            },
                        )
                    })?;
                <<Vec<
                    u8,
                > as ::rkyv::Archive>::Archived as ::rkyv::bytecheck::CheckBytes<
                    __C,
                >>::check_bytes(&raw const (*value).data, context)
                    .map_err(|e| {
                        <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Trace>::trace(
                            e,
                            ::rkyv::bytecheck::StructCheckContext {
                                struct_name: "ArchivedLargeChunk",
                                field_name: "data",
                            },
                        )
                    })?;
                ::core::result::Result::Ok(())
            }
        }
        #[automatically_derived]
        ///The resolver for an archived [`LargeChunk`]
        pub struct LargeChunkResolver
        where
            usize: ::rkyv::Archive,
            Vec<u8>: ::rkyv::Archive,
        {
            index: <usize as ::rkyv::Archive>::Resolver,
            data: <Vec<u8> as ::rkyv::Archive>::Resolver,
        }
        impl ::rkyv::Archive for LargeChunk
        where
            usize: ::rkyv::Archive,
            Vec<u8>: ::rkyv::Archive,
        {
            type Archived = ArchivedLargeChunk;
            type Resolver = LargeChunkResolver;
            const COPY_OPTIMIZATION: ::rkyv::traits::CopyOptimization<Self> = unsafe {
                ::rkyv::traits::CopyOptimization::enable_if(
                    0 + ::core::mem::size_of::<usize>()
                        + ::core::mem::size_of::<Vec<u8>>()
                        == ::core::mem::size_of::<LargeChunk>()
                        && <usize as ::rkyv::Archive>::COPY_OPTIMIZATION.is_enabled()
                        && const { builtin # offset_of(LargeChunk, index) }
                            == const { builtin # offset_of(ArchivedLargeChunk, index) }
                        && <Vec<u8> as ::rkyv::Archive>::COPY_OPTIMIZATION.is_enabled()
                        && const { builtin # offset_of(LargeChunk, data) }
                            == const { builtin # offset_of(ArchivedLargeChunk, data) },
                )
            };
            #[allow(clippy::unit_arg)]
            fn resolve(
                &self,
                resolver: Self::Resolver,
                out: ::rkyv::Place<Self::Archived>,
            ) {
                let field_ptr = unsafe { &raw mut (*out.ptr()).index };
                let field_out = unsafe {
                    ::rkyv::Place::from_field_unchecked(out, field_ptr)
                };
                <usize as ::rkyv::Archive>::resolve(
                    &self.index,
                    resolver.index,
                    field_out,
                );
                let field_ptr = unsafe { &raw mut (*out.ptr()).data };
                let field_out = unsafe {
                    ::rkyv::Place::from_field_unchecked(out, field_ptr)
                };
                <Vec<
                    u8,
                > as ::rkyv::Archive>::resolve(&self.data, resolver.data, field_out);
            }
        }
        unsafe impl ::rkyv::traits::Portable for ArchivedLargeChunk
        where
            usize: ::rkyv::Archive,
            Vec<u8>: ::rkyv::Archive,
            <usize as ::rkyv::Archive>::Archived: ::rkyv::traits::Portable,
            <Vec<u8> as ::rkyv::Archive>::Archived: ::rkyv::traits::Portable,
        {}
        #[automatically_derived]
        impl<__S: ::rkyv::rancor::Fallible + ?Sized> ::rkyv::Serialize<__S>
        for LargeChunk
        where
            usize: ::rkyv::Serialize<__S>,
            Vec<u8>: ::rkyv::Serialize<__S>,
        {
            fn serialize(
                &self,
                serializer: &mut __S,
            ) -> ::core::result::Result<
                <Self as ::rkyv::Archive>::Resolver,
                <__S as ::rkyv::rancor::Fallible>::Error,
            > {
                let __this = self;
                ::core::result::Result::Ok(LargeChunkResolver {
                    index: <usize as ::rkyv::Serialize<
                        __S,
                    >>::serialize(&__this.index, serializer)?,
                    data: <Vec<
                        u8,
                    > as ::rkyv::Serialize<__S>>::serialize(&__this.data, serializer)?,
                })
            }
        }
        #[automatically_derived]
        impl<__D: ::rkyv::rancor::Fallible + ?Sized> ::rkyv::Deserialize<LargeChunk, __D>
        for ::rkyv::Archived<LargeChunk>
        where
            usize: ::rkyv::Archive,
            <usize as ::rkyv::Archive>::Archived: ::rkyv::Deserialize<usize, __D>,
            Vec<u8>: ::rkyv::Archive,
            <Vec<u8> as ::rkyv::Archive>::Archived: ::rkyv::Deserialize<Vec<u8>, __D>,
        {
            fn deserialize(
                &self,
                deserializer: &mut __D,
            ) -> ::core::result::Result<
                LargeChunk,
                <__D as ::rkyv::rancor::Fallible>::Error,
            > {
                let __this = self;
                ::core::result::Result::Ok(LargeChunk {
                    index: <<usize as ::rkyv::Archive>::Archived as ::rkyv::Deserialize<
                        usize,
                        __D,
                    >>::deserialize(&__this.index, deserializer)?,
                    data: <<Vec<
                        u8,
                    > as ::rkyv::Archive>::Archived as ::rkyv::Deserialize<
                        Vec<u8>,
                        __D,
                    >>::deserialize(&__this.data, deserializer)?,
                })
            }
        }
        impl ::rewrite::traits::structural::blob::BlobItemChunk for LargeChunk {
            type Index = usize;
            fn get_index(&self) -> &Self::Index {
                &self.index
            }
        }
        pub enum LargeChunkFill {
            Full(usize),
            Partial(usize),
            Corrupted(usize),
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for LargeChunkFill {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                match self {
                    LargeChunkFill::Full(__self_0) => {
                        ::core::fmt::Formatter::debug_tuple_field1_finish(
                            f,
                            "Full",
                            &__self_0,
                        )
                    }
                    LargeChunkFill::Partial(__self_0) => {
                        ::core::fmt::Formatter::debug_tuple_field1_finish(
                            f,
                            "Partial",
                            &__self_0,
                        )
                    }
                    LargeChunkFill::Corrupted(__self_0) => {
                        ::core::fmt::Formatter::debug_tuple_field1_finish(
                            f,
                            "Corrupted",
                            &__self_0,
                        )
                    }
                }
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        unsafe impl ::core::clone::TrivialClone for LargeChunkFill {}
        #[automatically_derived]
        impl ::core::clone::Clone for LargeChunkFill {
            #[inline]
            fn clone(&self) -> LargeChunkFill {
                let _: ::core::clone::AssertParamIsClone<usize>;
                *self
            }
        }
        #[automatically_derived]
        impl ::core::marker::Copy for LargeChunkFill {}
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq for LargeChunkFill {}
        #[automatically_derived]
        impl ::core::cmp::PartialEq for LargeChunkFill {
            #[inline]
            fn eq(&self, other: &LargeChunkFill) -> bool {
                let __self_discr = ::core::intrinsics::discriminant_value(self);
                let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                __self_discr == __arg1_discr
                    && match (self, other) {
                        (
                            LargeChunkFill::Full(__self_0),
                            LargeChunkFill::Full(__arg1_0),
                        ) => __self_0 == __arg1_0,
                        (
                            LargeChunkFill::Partial(__self_0),
                            LargeChunkFill::Partial(__arg1_0),
                        ) => __self_0 == __arg1_0,
                        (
                            LargeChunkFill::Corrupted(__self_0),
                            LargeChunkFill::Corrupted(__arg1_0),
                        ) => __self_0 == __arg1_0,
                        _ => unsafe { ::core::intrinsics::unreachable() }
                    }
            }
        }
        #[automatically_derived]
        impl ::core::cmp::Eq for LargeChunkFill {
            #[inline]
            #[doc(hidden)]
            #[coverage(off)]
            fn assert_fields_are_eq(&self) {
                let _: ::core::cmp::AssertParamIsEq<usize>;
            }
        }
        #[automatically_derived]
        impl ::core::cmp::PartialOrd for LargeChunkFill {
            #[inline]
            fn partial_cmp(
                &self,
                other: &LargeChunkFill,
            ) -> ::core::option::Option<::core::cmp::Ordering> {
                let __self_discr = ::core::intrinsics::discriminant_value(self);
                let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                match (self, other) {
                    (LargeChunkFill::Full(__self_0), LargeChunkFill::Full(__arg1_0)) => {
                        ::core::cmp::PartialOrd::partial_cmp(__self_0, __arg1_0)
                    }
                    (
                        LargeChunkFill::Partial(__self_0),
                        LargeChunkFill::Partial(__arg1_0),
                    ) => ::core::cmp::PartialOrd::partial_cmp(__self_0, __arg1_0),
                    (
                        LargeChunkFill::Corrupted(__self_0),
                        LargeChunkFill::Corrupted(__arg1_0),
                    ) => ::core::cmp::PartialOrd::partial_cmp(__self_0, __arg1_0),
                    _ => {
                        ::core::cmp::PartialOrd::partial_cmp(
                            &__self_discr,
                            &__arg1_discr,
                        )
                    }
                }
            }
        }
        #[automatically_derived]
        impl ::core::cmp::Ord for LargeChunkFill {
            #[inline]
            fn cmp(&self, other: &LargeChunkFill) -> ::core::cmp::Ordering {
                let __self_discr = ::core::intrinsics::discriminant_value(self);
                let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                match ::core::cmp::Ord::cmp(&__self_discr, &__arg1_discr) {
                    ::core::cmp::Ordering::Equal => {
                        match (self, other) {
                            (
                                LargeChunkFill::Full(__self_0),
                                LargeChunkFill::Full(__arg1_0),
                            ) => ::core::cmp::Ord::cmp(__self_0, __arg1_0),
                            (
                                LargeChunkFill::Partial(__self_0),
                                LargeChunkFill::Partial(__arg1_0),
                            ) => ::core::cmp::Ord::cmp(__self_0, __arg1_0),
                            (
                                LargeChunkFill::Corrupted(__self_0),
                                LargeChunkFill::Corrupted(__arg1_0),
                            ) => ::core::cmp::Ord::cmp(__self_0, __arg1_0),
                            _ => unsafe { ::core::intrinsics::unreachable() }
                        }
                    }
                    cmp => cmp,
                }
            }
        }
        #[automatically_derived]
        ///An archived [`LargeChunkFill`]
        #[bytecheck(crate = ::rkyv::bytecheck)]
        #[repr(u8)]
        pub enum ArchivedLargeChunkFill
        where
            usize: ::rkyv::Archive,
            usize: ::rkyv::Archive,
            usize: ::rkyv::Archive,
        {
            ///The archived counterpart of [`LargeChunkFill::Full`]
            #[allow(dead_code)]
            Full(
                ///The archived counterpart of [`LargeChunkFill::Full::0`]
                <usize as ::rkyv::Archive>::Archived,
            ),
            ///The archived counterpart of [`LargeChunkFill::Partial`]
            #[allow(dead_code)]
            Partial(
                ///The archived counterpart of [`LargeChunkFill::Partial::0`]
                <usize as ::rkyv::Archive>::Archived,
            ),
            ///The archived counterpart of [`LargeChunkFill::Corrupted`]
            #[allow(dead_code)]
            Corrupted(
                ///The archived counterpart of [`LargeChunkFill::Corrupted::0`]
                <usize as ::rkyv::Archive>::Archived,
            ),
        }
        const _: () = {
            #[repr(u8)]
            enum Tag {
                Full,
                Partial,
                Corrupted,
            }
            struct Discriminant;
            #[automatically_derived]
            impl Discriminant {
                #[allow(non_upper_case_globals)]
                const Full: u8 = Tag::Full as u8;
                #[allow(non_upper_case_globals)]
                const Partial: u8 = Tag::Partial as u8;
                #[allow(non_upper_case_globals)]
                const Corrupted: u8 = Tag::Corrupted as u8;
            }
            #[repr(C)]
            struct VariantFull(
                Tag,
                <usize as ::rkyv::Archive>::Archived,
                ::core::marker::PhantomData<ArchivedLargeChunkFill>,
            )
            where
                usize: ::rkyv::Archive,
                usize: ::rkyv::Archive,
                usize: ::rkyv::Archive;
            #[repr(C)]
            struct VariantPartial(
                Tag,
                <usize as ::rkyv::Archive>::Archived,
                ::core::marker::PhantomData<ArchivedLargeChunkFill>,
            )
            where
                usize: ::rkyv::Archive,
                usize: ::rkyv::Archive,
                usize: ::rkyv::Archive;
            #[repr(C)]
            struct VariantCorrupted(
                Tag,
                <usize as ::rkyv::Archive>::Archived,
                ::core::marker::PhantomData<ArchivedLargeChunkFill>,
            )
            where
                usize: ::rkyv::Archive,
                usize: ::rkyv::Archive,
                usize: ::rkyv::Archive;
            #[automatically_derived]
            unsafe impl<
                __C: ::rkyv::bytecheck::rancor::Fallible + ?::core::marker::Sized,
            > ::rkyv::bytecheck::CheckBytes<__C> for ArchivedLargeChunkFill
            where
                usize: ::rkyv::Archive,
                usize: ::rkyv::Archive,
                usize: ::rkyv::Archive,
                <__C as ::rkyv::bytecheck::rancor::Fallible>::Error: ::rkyv::bytecheck::rancor::Source,
                <usize as ::rkyv::Archive>::Archived: ::rkyv::bytecheck::CheckBytes<__C>,
                <usize as ::rkyv::Archive>::Archived: ::rkyv::bytecheck::CheckBytes<__C>,
                <usize as ::rkyv::Archive>::Archived: ::rkyv::bytecheck::CheckBytes<__C>,
            {
                unsafe fn check_bytes(
                    value: *const Self,
                    context: &mut __C,
                ) -> ::core::result::Result<
                    (),
                    <__C as ::rkyv::bytecheck::rancor::Fallible>::Error,
                > {
                    let tag = *value.cast::<u8>();
                    match tag {
                        Discriminant::Full => {
                            let value = value.cast::<VariantFull>();
                            <<usize as ::rkyv::Archive>::Archived as ::rkyv::bytecheck::CheckBytes<
                                __C,
                            >>::check_bytes(&raw const (*value).1, context)
                                .map_err(|e| {
                                    <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Trace>::trace(
                                        e,
                                        ::rkyv::bytecheck::UnnamedEnumVariantCheckContext {
                                            enum_name: "ArchivedLargeChunkFill",
                                            variant_name: "Full",
                                            field_index: 1,
                                        },
                                    )
                                })?;
                        }
                        Discriminant::Partial => {
                            let value = value.cast::<VariantPartial>();
                            <<usize as ::rkyv::Archive>::Archived as ::rkyv::bytecheck::CheckBytes<
                                __C,
                            >>::check_bytes(&raw const (*value).1, context)
                                .map_err(|e| {
                                    <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Trace>::trace(
                                        e,
                                        ::rkyv::bytecheck::UnnamedEnumVariantCheckContext {
                                            enum_name: "ArchivedLargeChunkFill",
                                            variant_name: "Partial",
                                            field_index: 1,
                                        },
                                    )
                                })?;
                        }
                        Discriminant::Corrupted => {
                            let value = value.cast::<VariantCorrupted>();
                            <<usize as ::rkyv::Archive>::Archived as ::rkyv::bytecheck::CheckBytes<
                                __C,
                            >>::check_bytes(&raw const (*value).1, context)
                                .map_err(|e| {
                                    <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Trace>::trace(
                                        e,
                                        ::rkyv::bytecheck::UnnamedEnumVariantCheckContext {
                                            enum_name: "ArchivedLargeChunkFill",
                                            variant_name: "Corrupted",
                                            field_index: 1,
                                        },
                                    )
                                })?;
                        }
                        _ => {
                            return ::core::result::Result::Err(
                                <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Source>::new(::rkyv::bytecheck::InvalidEnumDiscriminantError {
                                    enum_name: "ArchivedLargeChunkFill",
                                    invalid_discriminant: tag,
                                }),
                            );
                        }
                    }
                    ::core::result::Result::Ok(())
                }
            }
        };
        #[automatically_derived]
        ///The resolver for an archived [`LargeChunkFill`]
        pub enum LargeChunkFillResolver
        where
            usize: ::rkyv::Archive,
            usize: ::rkyv::Archive,
            usize: ::rkyv::Archive,
        {
            ///The resolver for [`LargeChunkFill::Full`]
            #[allow(dead_code)]
            Full(<usize as ::rkyv::Archive>::Resolver),
            ///The resolver for [`LargeChunkFill::Partial`]
            #[allow(dead_code)]
            Partial(<usize as ::rkyv::Archive>::Resolver),
            ///The resolver for [`LargeChunkFill::Corrupted`]
            #[allow(dead_code)]
            Corrupted(<usize as ::rkyv::Archive>::Resolver),
        }
        const _: () = {
            #[repr(u8)]
            enum ArchivedTag {
                Full,
                Partial,
                Corrupted,
            }
            #[automatically_derived]
            impl ::core::marker::StructuralPartialEq for ArchivedTag {}
            #[automatically_derived]
            impl ::core::cmp::PartialEq for ArchivedTag {
                #[inline]
                fn eq(&self, other: &ArchivedTag) -> bool {
                    let __self_discr = ::core::intrinsics::discriminant_value(self);
                    let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                    __self_discr == __arg1_discr
                }
            }
            #[automatically_derived]
            impl ::core::cmp::PartialOrd for ArchivedTag {
                #[inline]
                fn partial_cmp(
                    &self,
                    other: &ArchivedTag,
                ) -> ::core::option::Option<::core::cmp::Ordering> {
                    let __self_discr = ::core::intrinsics::discriminant_value(self);
                    let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                    ::core::cmp::PartialOrd::partial_cmp(&__self_discr, &__arg1_discr)
                }
            }
            #[repr(C)]
            struct ArchivedVariantFull(
                ArchivedTag,
                <usize as ::rkyv::Archive>::Archived,
                ::core::marker::PhantomData<LargeChunkFill>,
            )
            where
                usize: ::rkyv::Archive,
                usize: ::rkyv::Archive,
                usize: ::rkyv::Archive;
            #[repr(C)]
            struct ArchivedVariantPartial(
                ArchivedTag,
                <usize as ::rkyv::Archive>::Archived,
                ::core::marker::PhantomData<LargeChunkFill>,
            )
            where
                usize: ::rkyv::Archive,
                usize: ::rkyv::Archive,
                usize: ::rkyv::Archive;
            #[repr(C)]
            struct ArchivedVariantCorrupted(
                ArchivedTag,
                <usize as ::rkyv::Archive>::Archived,
                ::core::marker::PhantomData<LargeChunkFill>,
            )
            where
                usize: ::rkyv::Archive,
                usize: ::rkyv::Archive,
                usize: ::rkyv::Archive;
            impl ::rkyv::Archive for LargeChunkFill
            where
                usize: ::rkyv::Archive,
                usize: ::rkyv::Archive,
                usize: ::rkyv::Archive,
            {
                type Archived = ArchivedLargeChunkFill;
                type Resolver = LargeChunkFillResolver;
                #[allow(clippy::unit_arg)]
                fn resolve(
                    &self,
                    resolver: <Self as ::rkyv::Archive>::Resolver,
                    out: ::rkyv::Place<<Self as ::rkyv::Archive>::Archived>,
                ) {
                    let __this = self;
                    match resolver {
                        LargeChunkFillResolver::Full(resolver_0) => {
                            match __this {
                                LargeChunkFill::Full(self_0, ..) => {
                                    let out = unsafe {
                                        out.cast_unchecked::<ArchivedVariantFull>()
                                    };
                                    let tag_ptr = unsafe { &raw mut (*out.ptr()).0 };
                                    unsafe {
                                        tag_ptr.write(ArchivedTag::Full);
                                    }
                                    let field_ptr = unsafe { &raw mut (*out.ptr()).1 };
                                    let out_field = unsafe {
                                        ::rkyv::Place::from_field_unchecked(out, field_ptr)
                                    };
                                    <usize as ::rkyv::Archive>::resolve(
                                        self_0,
                                        resolver_0,
                                        out_field,
                                    );
                                }
                                #[allow(unreachable_patterns)]
                                _ => unsafe { ::core::hint::unreachable_unchecked() }
                            }
                        }
                        LargeChunkFillResolver::Partial(resolver_0) => {
                            match __this {
                                LargeChunkFill::Partial(self_0, ..) => {
                                    let out = unsafe {
                                        out.cast_unchecked::<ArchivedVariantPartial>()
                                    };
                                    let tag_ptr = unsafe { &raw mut (*out.ptr()).0 };
                                    unsafe {
                                        tag_ptr.write(ArchivedTag::Partial);
                                    }
                                    let field_ptr = unsafe { &raw mut (*out.ptr()).1 };
                                    let out_field = unsafe {
                                        ::rkyv::Place::from_field_unchecked(out, field_ptr)
                                    };
                                    <usize as ::rkyv::Archive>::resolve(
                                        self_0,
                                        resolver_0,
                                        out_field,
                                    );
                                }
                                #[allow(unreachable_patterns)]
                                _ => unsafe { ::core::hint::unreachable_unchecked() }
                            }
                        }
                        LargeChunkFillResolver::Corrupted(resolver_0) => {
                            match __this {
                                LargeChunkFill::Corrupted(self_0, ..) => {
                                    let out = unsafe {
                                        out.cast_unchecked::<ArchivedVariantCorrupted>()
                                    };
                                    let tag_ptr = unsafe { &raw mut (*out.ptr()).0 };
                                    unsafe {
                                        tag_ptr.write(ArchivedTag::Corrupted);
                                    }
                                    let field_ptr = unsafe { &raw mut (*out.ptr()).1 };
                                    let out_field = unsafe {
                                        ::rkyv::Place::from_field_unchecked(out, field_ptr)
                                    };
                                    <usize as ::rkyv::Archive>::resolve(
                                        self_0,
                                        resolver_0,
                                        out_field,
                                    );
                                }
                                #[allow(unreachable_patterns)]
                                _ => unsafe { ::core::hint::unreachable_unchecked() }
                            }
                        }
                    }
                }
            }
        };
        unsafe impl ::rkyv::traits::Portable for ArchivedLargeChunkFill
        where
            usize: ::rkyv::Archive,
            usize: ::rkyv::Archive,
            usize: ::rkyv::Archive,
            <usize as ::rkyv::Archive>::Archived: ::rkyv::traits::Portable,
            <usize as ::rkyv::Archive>::Archived: ::rkyv::traits::Portable,
            <usize as ::rkyv::Archive>::Archived: ::rkyv::traits::Portable,
        {}
        #[automatically_derived]
        impl<__S: ::rkyv::rancor::Fallible + ?Sized> ::rkyv::Serialize<__S>
        for LargeChunkFill
        where
            usize: ::rkyv::Serialize<__S>,
            usize: ::rkyv::Serialize<__S>,
            usize: ::rkyv::Serialize<__S>,
        {
            fn serialize(
                &self,
                serializer: &mut __S,
            ) -> ::core::result::Result<
                <Self as ::rkyv::Archive>::Resolver,
                <__S as ::rkyv::rancor::Fallible>::Error,
            > {
                let __this = self;
                ::core::result::Result::Ok(
                    match __this {
                        LargeChunkFill::Full(_0, ..) => {
                            LargeChunkFillResolver::Full(
                                <usize as ::rkyv::Serialize<
                                    __S,
                                >>::serialize(_0, serializer)?,
                            )
                        }
                        LargeChunkFill::Partial(_0, ..) => {
                            LargeChunkFillResolver::Partial(
                                <usize as ::rkyv::Serialize<
                                    __S,
                                >>::serialize(_0, serializer)?,
                            )
                        }
                        LargeChunkFill::Corrupted(_0, ..) => {
                            LargeChunkFillResolver::Corrupted(
                                <usize as ::rkyv::Serialize<
                                    __S,
                                >>::serialize(_0, serializer)?,
                            )
                        }
                    },
                )
            }
        }
        #[automatically_derived]
        impl<
            __D: ::rkyv::rancor::Fallible + ?Sized,
        > ::rkyv::Deserialize<LargeChunkFill, __D> for ::rkyv::Archived<LargeChunkFill>
        where
            usize: ::rkyv::Archive,
            <usize as ::rkyv::Archive>::Archived: ::rkyv::Deserialize<usize, __D>,
            usize: ::rkyv::Archive,
            <usize as ::rkyv::Archive>::Archived: ::rkyv::Deserialize<usize, __D>,
            usize: ::rkyv::Archive,
            <usize as ::rkyv::Archive>::Archived: ::rkyv::Deserialize<usize, __D>,
        {
            fn deserialize(
                &self,
                deserializer: &mut __D,
            ) -> ::core::result::Result<
                LargeChunkFill,
                <__D as ::rkyv::rancor::Fallible>::Error,
            > {
                let __this = self;
                ::core::result::Result::Ok(
                    match __this {
                        Self::Full(_0, ..) => {
                            LargeChunkFill::Full(
                                <<usize as ::rkyv::Archive>::Archived as ::rkyv::Deserialize<
                                    usize,
                                    __D,
                                >>::deserialize(_0, deserializer)?,
                            )
                        }
                        Self::Partial(_0, ..) => {
                            LargeChunkFill::Partial(
                                <<usize as ::rkyv::Archive>::Archived as ::rkyv::Deserialize<
                                    usize,
                                    __D,
                                >>::deserialize(_0, deserializer)?,
                            )
                        }
                        Self::Corrupted(_0, ..) => {
                            LargeChunkFill::Corrupted(
                                <<usize as ::rkyv::Archive>::Archived as ::rkyv::Deserialize<
                                    usize,
                                    __D,
                                >>::deserialize(_0, deserializer)?,
                            )
                        }
                    },
                )
            }
        }
        impl LargeChunkFill {
            pub fn from_size(actual: usize, expected: usize) -> Self {
                if actual == expected {
                    Self::Full(actual)
                } else if actual < expected {
                    Self::Partial(actual)
                } else {
                    Self::Corrupted(actual)
                }
            }
        }
        impl rewrite::traits::structural::blob::NetabaseBlobItem for Large {
            type Chunk = LargeChunk;
            type BlobIter = std::vec::IntoIter<
                rewrite::results::NetabaseResult<Self::Chunk>,
            >;
            const DEFAULT_CHUNK_SIZE: usize = 0usize;
            fn into_chunks(
                self,
                size: rewrite::traits::structural::blob::ChunkSize,
            ) -> Box<dyn Iterator<Item = Self::Chunk>> {
                Box::new(self.into_chunks_iter(size).filter_map(|r| r.ok()))
            }
            fn into_chunks_iter(
                self,
                size: rewrite::traits::structural::blob::ChunkSize,
            ) -> Self::BlobIter {
                let serialized_data: rewrite::results::NetabaseResult<Vec<u8>> = (|| -> rewrite::results::NetabaseResult<
                    Vec<u8>,
                > {
                    Ok(
                        rkyv::to_bytes::<rkyv::rancor::Error>(&self)
                            .map_err(|e| rewrite::results::NetabaseError::Serialization(
                                ::alloc::__export::must_use({
                                    ::alloc::fmt::format(
                                        format_args!("rkyv serialization failed: {0:?}", e),
                                    )
                                }),
                            ))?
                            .to_vec(),
                    )
                })();
                let chunk_size = match size {
                    rewrite::traits::structural::blob::ChunkSize::Default => {
                        if Self::DEFAULT_CHUNK_SIZE > 0 {
                            Self::DEFAULT_CHUNK_SIZE
                        } else {
                            1024
                        }
                    }
                    rewrite::traits::structural::blob::ChunkSize::Size(n) => n,
                };
                match serialized_data {
                    Ok(data) => {
                        data.chunks(chunk_size)
                            .enumerate()
                            .map(|(index, chunk_data)| {
                                Ok(Self::Chunk {
                                    index,
                                    data: chunk_data.to_vec(),
                                })
                            })
                            .collect::<Vec<_>>()
                            .into_iter()
                    }
                    Err(e) => {
                        ::alloc::boxed::box_assume_init_into_vec_unsafe(
                                ::alloc::intrinsics::write_box_via_move(
                                    ::alloc::boxed::Box::new_uninit(),
                                    [Err(e)],
                                ),
                            )
                            .into_iter()
                    }
                }
            }
            fn try_from_chunks(
                chunks: impl Iterator<Item = Self::Chunk>,
                size: rewrite::traits::structural::blob::ChunkSize,
            ) -> rewrite::results::NetabaseResult<Self> {
                let mut sorted_chunks: Vec<_> = chunks.collect();
                sorted_chunks.sort_by_key(|c| c.index);
                if sorted_chunks.is_empty() {
                    return Err(
                        rewrite::results::NetabaseError::BlobReconstruction(
                            rewrite::results::BlobReconstructionError::MissingChunks,
                        ),
                    );
                }
                let chunk_size = match size {
                    rewrite::traits::structural::blob::ChunkSize::Default => {
                        if Self::DEFAULT_CHUNK_SIZE > 0 {
                            Self::DEFAULT_CHUNK_SIZE
                        } else {
                            1024
                        }
                    }
                    rewrite::traits::structural::blob::ChunkSize::Size(n) => n,
                };
                let mut missing_details = Vec::new();
                let mut next_expected = 0;
                let max_idx = sorted_chunks.last().map(|c| c.index).unwrap_or(0);
                for chunk in &sorted_chunks {
                    while chunk.index > next_expected {
                        missing_details
                            .push(
                                ::alloc::__export::must_use({
                                    ::alloc::fmt::format(
                                        format_args!(
                                            "{0:?}({{ Index: {1}, Size: {2} }})",
                                            LargeChunkFill::Full(chunk_size),
                                            next_expected,
                                            chunk_size,
                                        ),
                                    )
                                }),
                            );
                        next_expected += 1;
                    }
                    let fill = LargeChunkFill::from_size(chunk.data.len(), chunk_size);
                    match fill {
                        LargeChunkFill::Corrupted(size) => {
                            return Err(
                                rewrite::results::NetabaseError::BlobReconstruction(
                                    rewrite::results::BlobReconstructionError::InvalidChunkData(
                                        ::alloc::__export::must_use({
                                            ::alloc::fmt::format(
                                                format_args!(
                                                    "Corrupted chunk detected: {0:?}({{ Index: {1}, Size: {2} }}). Max allowed size is {3}.",
                                                    fill,
                                                    chunk.index,
                                                    size,
                                                    chunk_size,
                                                ),
                                            )
                                        }),
                                    ),
                                ),
                            );
                        }
                        LargeChunkFill::Partial(size) if chunk.index < max_idx => {
                            return Err(
                                rewrite::results::NetabaseError::BlobReconstruction(
                                    rewrite::results::BlobReconstructionError::InvalidChunkData(
                                        ::alloc::__export::must_use({
                                            ::alloc::fmt::format(
                                                format_args!(
                                                    "Unexpected partial chunk in middle of stream: {0:?}({{ Index: {1}, Size: {2} }}). Expected {3} bytes.",
                                                    fill,
                                                    chunk.index,
                                                    size,
                                                    chunk_size,
                                                ),
                                            )
                                        }),
                                    ),
                                ),
                            );
                        }
                        _ => {}
                    }
                    if chunk.index == next_expected {
                        next_expected += 1;
                    }
                }
                if !missing_details.is_empty() {
                    if let Some(last) = sorted_chunks.last() {
                        let fill = LargeChunkFill::from_size(
                            last.data.len(),
                            chunk_size,
                        );
                        if #[allow(non_exhaustive_omitted_patterns)]
                        match fill {
                            LargeChunkFill::Full(_) => true,
                            _ => false,
                        } {
                            missing_details
                                .push(
                                    ::alloc::__export::must_use({
                                        ::alloc::fmt::format(
                                            format_args!(
                                                "... (Stream truncated: last chunk was Full, expected more data after Index {0})",
                                                last.index,
                                            ),
                                        )
                                    }),
                                );
                        }
                    }
                }
                if !missing_details.is_empty() {
                    return Err(
                        rewrite::results::NetabaseError::BlobReconstruction(
                            rewrite::results::BlobReconstructionError::InvalidChunkData(
                                ::alloc::__export::must_use({
                                    ::alloc::fmt::format(
                                        format_args!(
                                            "Missing chunks: [{0}]. Total chunks present: {1}",
                                            missing_details.join(", "),
                                            sorted_chunks.len(),
                                        ),
                                    )
                                }),
                            ),
                        ),
                    );
                }
                let serialized_data: Vec<u8> = sorted_chunks
                    .into_iter()
                    .flat_map(|c| c.data)
                    .collect();
                Ok(
                    rkyv::from_bytes::<Self, rkyv::rancor::Error>(&serialized_data)
                        .map_err(|e| rewrite::results::NetabaseError::Serialization(
                            ::alloc::__export::must_use({
                                ::alloc::fmt::format(
                                    format_args!("rkyv deserialization failed: {0:?}", e),
                                )
                            }),
                        ))?,
                )
            }
            fn get_blob(&self) -> &Self::Chunk {
                {
                    ::core::panicking::panic_fmt(
                        format_args!(
                            "not implemented: {0}",
                            format_args!("get_blob() requires storing a chunk reference"),
                        ),
                    );
                }
            }
        }
        impl IntoIterator for Large {
            type Item = rewrite::results::NetabaseResult<LargeChunk>;
            type IntoIter = std::vec::IntoIter<Self::Item>;
            fn into_iter(self) -> Self::IntoIter {
                rewrite::traits::structural::blob::NetabaseBlobItem::into_chunks_iter(
                    self,
                    rewrite::traits::structural::blob::ChunkSize::Default,
                )
            }
        }
        let blob = Large {
            data: ::alloc::vec::from_elem(0u8, 200),
        };
        let mut chunks: Vec<_> = blob.into_chunks(ChunkSize::Size(64)).collect();
        if !(chunks.len() >= 4) {
            ::core::panicking::panic("assertion failed: chunks.len() >= 4")
        }
        chunks[0].data.truncate(32);
        let result = Large::try_from_chunks(chunks.into_iter(), ChunkSize::Size(64));
        if !#[allow(non_exhaustive_omitted_patterns)]
        match result {
            Err(
                NetabaseError::BlobReconstruction(
                    BlobReconstructionError::InvalidChunkData(s),
                ),
            ) if s.contains("Unexpected partial chunk in middle of stream") => true,
            _ => false,
        } {
            ::core::panicking::panic(
                "assertion failed: matches!(result,\n    Err(NetabaseError::BlobReconstruction(BlobReconstructionError::InvalidChunkData(s)))\n    if s.contains(\"Unexpected partial chunk in middle of stream\"))",
            )
        }
    }
    extern crate test;
    #[rustc_test_marker = "tests::test_error_truncated_stream"]
    #[doc(hidden)]
    pub const test_error_truncated_stream: test::TestDescAndFn = test::TestDescAndFn {
        desc: test::TestDesc {
            name: test::StaticTestName("tests::test_error_truncated_stream"),
            ignore: false,
            ignore_message: ::core::option::Option::None,
            source_file: "tests/blob_comprehensive_test.rs",
            start_line: 194usize,
            start_col: 8usize,
            end_line: 194usize,
            end_col: 35usize,
            compile_fail: false,
            no_run: false,
            should_panic: test::ShouldPanic::No,
            test_type: test::TestType::IntegrationTest,
        },
        testfn: test::StaticTestFn(
            #[coverage(off)]
            || test::assert_test_result(test_error_truncated_stream()),
        ),
    };
    fn test_error_truncated_stream() {
        struct Trunc {
            data: Vec<u8>,
        }
        #[automatically_derived]
        ///An archived [`Trunc`]
        #[bytecheck(crate = ::rkyv::bytecheck)]
        #[repr(C)]
        struct ArchivedTrunc
        where
            Vec<u8>: ::rkyv::Archive,
        {
            ///The archived counterpart of [`Trunc::data`]
            data: <Vec<u8> as ::rkyv::Archive>::Archived,
        }
        #[automatically_derived]
        unsafe impl<
            __C: ::rkyv::bytecheck::rancor::Fallible + ?::core::marker::Sized,
        > ::rkyv::bytecheck::CheckBytes<__C> for ArchivedTrunc
        where
            Vec<u8>: ::rkyv::Archive,
            <__C as ::rkyv::bytecheck::rancor::Fallible>::Error: ::rkyv::bytecheck::rancor::Trace,
            <Vec<u8> as ::rkyv::Archive>::Archived: ::rkyv::bytecheck::CheckBytes<__C>,
        {
            unsafe fn check_bytes(
                value: *const Self,
                context: &mut __C,
            ) -> ::core::result::Result<
                (),
                <__C as ::rkyv::bytecheck::rancor::Fallible>::Error,
            > {
                <<Vec<
                    u8,
                > as ::rkyv::Archive>::Archived as ::rkyv::bytecheck::CheckBytes<
                    __C,
                >>::check_bytes(&raw const (*value).data, context)
                    .map_err(|e| {
                        <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Trace>::trace(
                            e,
                            ::rkyv::bytecheck::StructCheckContext {
                                struct_name: "ArchivedTrunc",
                                field_name: "data",
                            },
                        )
                    })?;
                ::core::result::Result::Ok(())
            }
        }
        #[automatically_derived]
        ///The resolver for an archived [`Trunc`]
        struct TruncResolver
        where
            Vec<u8>: ::rkyv::Archive,
        {
            data: <Vec<u8> as ::rkyv::Archive>::Resolver,
        }
        impl ::rkyv::Archive for Trunc
        where
            Vec<u8>: ::rkyv::Archive,
        {
            type Archived = ArchivedTrunc;
            type Resolver = TruncResolver;
            const COPY_OPTIMIZATION: ::rkyv::traits::CopyOptimization<Self> = unsafe {
                ::rkyv::traits::CopyOptimization::enable_if(
                    0 + ::core::mem::size_of::<Vec<u8>>()
                        == ::core::mem::size_of::<Trunc>()
                        && <Vec<u8> as ::rkyv::Archive>::COPY_OPTIMIZATION.is_enabled()
                        && const { builtin # offset_of(Trunc, data) }
                            == const { builtin # offset_of(ArchivedTrunc, data) },
                )
            };
            #[allow(clippy::unit_arg)]
            fn resolve(
                &self,
                resolver: Self::Resolver,
                out: ::rkyv::Place<Self::Archived>,
            ) {
                let field_ptr = unsafe { &raw mut (*out.ptr()).data };
                let field_out = unsafe {
                    ::rkyv::Place::from_field_unchecked(out, field_ptr)
                };
                <Vec<
                    u8,
                > as ::rkyv::Archive>::resolve(&self.data, resolver.data, field_out);
            }
        }
        unsafe impl ::rkyv::traits::Portable for ArchivedTrunc
        where
            Vec<u8>: ::rkyv::Archive,
            <Vec<u8> as ::rkyv::Archive>::Archived: ::rkyv::traits::Portable,
        {}
        #[automatically_derived]
        impl<__S: ::rkyv::rancor::Fallible + ?Sized> ::rkyv::Serialize<__S> for Trunc
        where
            Vec<u8>: ::rkyv::Serialize<__S>,
        {
            fn serialize(
                &self,
                serializer: &mut __S,
            ) -> ::core::result::Result<
                <Self as ::rkyv::Archive>::Resolver,
                <__S as ::rkyv::rancor::Fallible>::Error,
            > {
                let __this = self;
                ::core::result::Result::Ok(TruncResolver {
                    data: <Vec<
                        u8,
                    > as ::rkyv::Serialize<__S>>::serialize(&__this.data, serializer)?,
                })
            }
        }
        #[automatically_derived]
        impl<__D: ::rkyv::rancor::Fallible + ?Sized> ::rkyv::Deserialize<Trunc, __D>
        for ::rkyv::Archived<Trunc>
        where
            Vec<u8>: ::rkyv::Archive,
            <Vec<u8> as ::rkyv::Archive>::Archived: ::rkyv::Deserialize<Vec<u8>, __D>,
        {
            fn deserialize(
                &self,
                deserializer: &mut __D,
            ) -> ::core::result::Result<
                Trunc,
                <__D as ::rkyv::rancor::Fallible>::Error,
            > {
                let __this = self;
                ::core::result::Result::Ok(Trunc {
                    data: <<Vec<
                        u8,
                    > as ::rkyv::Archive>::Archived as ::rkyv::Deserialize<
                        Vec<u8>,
                        __D,
                    >>::deserialize(&__this.data, deserializer)?,
                })
            }
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for Trunc {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field1_finish(
                    f,
                    "Trunc",
                    "data",
                    &&self.data,
                )
            }
        }
        #[automatically_derived]
        impl ::core::clone::Clone for Trunc {
            #[inline]
            fn clone(&self) -> Trunc {
                Trunc {
                    data: ::core::clone::Clone::clone(&self.data),
                }
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq for Trunc {}
        #[automatically_derived]
        impl ::core::cmp::PartialEq for Trunc {
            #[inline]
            fn eq(&self, other: &Trunc) -> bool {
                self.data == other.data
            }
        }
        #[automatically_derived]
        impl ::core::cmp::Eq for Trunc {
            #[inline]
            #[doc(hidden)]
            #[coverage(off)]
            fn assert_fields_are_eq(&self) {
                let _: ::core::cmp::AssertParamIsEq<Vec<u8>>;
            }
        }
        pub struct TruncChunk {
            pub index: usize,
            pub data: Vec<u8>,
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for TruncChunk {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field2_finish(
                    f,
                    "TruncChunk",
                    "index",
                    &self.index,
                    "data",
                    &&self.data,
                )
            }
        }
        #[automatically_derived]
        impl ::core::clone::Clone for TruncChunk {
            #[inline]
            fn clone(&self) -> TruncChunk {
                TruncChunk {
                    index: ::core::clone::Clone::clone(&self.index),
                    data: ::core::clone::Clone::clone(&self.data),
                }
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq for TruncChunk {}
        #[automatically_derived]
        impl ::core::cmp::PartialEq for TruncChunk {
            #[inline]
            fn eq(&self, other: &TruncChunk) -> bool {
                self.index == other.index && self.data == other.data
            }
        }
        #[automatically_derived]
        impl ::core::cmp::Eq for TruncChunk {
            #[inline]
            #[doc(hidden)]
            #[coverage(off)]
            fn assert_fields_are_eq(&self) {
                let _: ::core::cmp::AssertParamIsEq<usize>;
                let _: ::core::cmp::AssertParamIsEq<Vec<u8>>;
            }
        }
        #[automatically_derived]
        impl ::core::cmp::PartialOrd for TruncChunk {
            #[inline]
            fn partial_cmp(
                &self,
                other: &TruncChunk,
            ) -> ::core::option::Option<::core::cmp::Ordering> {
                match ::core::cmp::PartialOrd::partial_cmp(&self.index, &other.index) {
                    ::core::option::Option::Some(::core::cmp::Ordering::Equal) => {
                        ::core::cmp::PartialOrd::partial_cmp(&self.data, &other.data)
                    }
                    cmp => cmp,
                }
            }
        }
        #[automatically_derived]
        impl ::core::cmp::Ord for TruncChunk {
            #[inline]
            fn cmp(&self, other: &TruncChunk) -> ::core::cmp::Ordering {
                match ::core::cmp::Ord::cmp(&self.index, &other.index) {
                    ::core::cmp::Ordering::Equal => {
                        ::core::cmp::Ord::cmp(&self.data, &other.data)
                    }
                    cmp => cmp,
                }
            }
        }
        #[automatically_derived]
        ///An archived [`TruncChunk`]
        #[bytecheck(crate = ::rkyv::bytecheck)]
        #[repr(C)]
        pub struct ArchivedTruncChunk
        where
            usize: ::rkyv::Archive,
            Vec<u8>: ::rkyv::Archive,
        {
            ///The archived counterpart of [`TruncChunk::index`]
            pub index: <usize as ::rkyv::Archive>::Archived,
            ///The archived counterpart of [`TruncChunk::data`]
            pub data: <Vec<u8> as ::rkyv::Archive>::Archived,
        }
        #[automatically_derived]
        unsafe impl<
            __C: ::rkyv::bytecheck::rancor::Fallible + ?::core::marker::Sized,
        > ::rkyv::bytecheck::CheckBytes<__C> for ArchivedTruncChunk
        where
            usize: ::rkyv::Archive,
            Vec<u8>: ::rkyv::Archive,
            <__C as ::rkyv::bytecheck::rancor::Fallible>::Error: ::rkyv::bytecheck::rancor::Trace,
            <usize as ::rkyv::Archive>::Archived: ::rkyv::bytecheck::CheckBytes<__C>,
            <Vec<u8> as ::rkyv::Archive>::Archived: ::rkyv::bytecheck::CheckBytes<__C>,
        {
            unsafe fn check_bytes(
                value: *const Self,
                context: &mut __C,
            ) -> ::core::result::Result<
                (),
                <__C as ::rkyv::bytecheck::rancor::Fallible>::Error,
            > {
                <<usize as ::rkyv::Archive>::Archived as ::rkyv::bytecheck::CheckBytes<
                    __C,
                >>::check_bytes(&raw const (*value).index, context)
                    .map_err(|e| {
                        <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Trace>::trace(
                            e,
                            ::rkyv::bytecheck::StructCheckContext {
                                struct_name: "ArchivedTruncChunk",
                                field_name: "index",
                            },
                        )
                    })?;
                <<Vec<
                    u8,
                > as ::rkyv::Archive>::Archived as ::rkyv::bytecheck::CheckBytes<
                    __C,
                >>::check_bytes(&raw const (*value).data, context)
                    .map_err(|e| {
                        <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Trace>::trace(
                            e,
                            ::rkyv::bytecheck::StructCheckContext {
                                struct_name: "ArchivedTruncChunk",
                                field_name: "data",
                            },
                        )
                    })?;
                ::core::result::Result::Ok(())
            }
        }
        #[automatically_derived]
        ///The resolver for an archived [`TruncChunk`]
        pub struct TruncChunkResolver
        where
            usize: ::rkyv::Archive,
            Vec<u8>: ::rkyv::Archive,
        {
            index: <usize as ::rkyv::Archive>::Resolver,
            data: <Vec<u8> as ::rkyv::Archive>::Resolver,
        }
        impl ::rkyv::Archive for TruncChunk
        where
            usize: ::rkyv::Archive,
            Vec<u8>: ::rkyv::Archive,
        {
            type Archived = ArchivedTruncChunk;
            type Resolver = TruncChunkResolver;
            const COPY_OPTIMIZATION: ::rkyv::traits::CopyOptimization<Self> = unsafe {
                ::rkyv::traits::CopyOptimization::enable_if(
                    0 + ::core::mem::size_of::<usize>()
                        + ::core::mem::size_of::<Vec<u8>>()
                        == ::core::mem::size_of::<TruncChunk>()
                        && <usize as ::rkyv::Archive>::COPY_OPTIMIZATION.is_enabled()
                        && const { builtin # offset_of(TruncChunk, index) }
                            == const { builtin # offset_of(ArchivedTruncChunk, index) }
                        && <Vec<u8> as ::rkyv::Archive>::COPY_OPTIMIZATION.is_enabled()
                        && const { builtin # offset_of(TruncChunk, data) }
                            == const { builtin # offset_of(ArchivedTruncChunk, data) },
                )
            };
            #[allow(clippy::unit_arg)]
            fn resolve(
                &self,
                resolver: Self::Resolver,
                out: ::rkyv::Place<Self::Archived>,
            ) {
                let field_ptr = unsafe { &raw mut (*out.ptr()).index };
                let field_out = unsafe {
                    ::rkyv::Place::from_field_unchecked(out, field_ptr)
                };
                <usize as ::rkyv::Archive>::resolve(
                    &self.index,
                    resolver.index,
                    field_out,
                );
                let field_ptr = unsafe { &raw mut (*out.ptr()).data };
                let field_out = unsafe {
                    ::rkyv::Place::from_field_unchecked(out, field_ptr)
                };
                <Vec<
                    u8,
                > as ::rkyv::Archive>::resolve(&self.data, resolver.data, field_out);
            }
        }
        unsafe impl ::rkyv::traits::Portable for ArchivedTruncChunk
        where
            usize: ::rkyv::Archive,
            Vec<u8>: ::rkyv::Archive,
            <usize as ::rkyv::Archive>::Archived: ::rkyv::traits::Portable,
            <Vec<u8> as ::rkyv::Archive>::Archived: ::rkyv::traits::Portable,
        {}
        #[automatically_derived]
        impl<__S: ::rkyv::rancor::Fallible + ?Sized> ::rkyv::Serialize<__S>
        for TruncChunk
        where
            usize: ::rkyv::Serialize<__S>,
            Vec<u8>: ::rkyv::Serialize<__S>,
        {
            fn serialize(
                &self,
                serializer: &mut __S,
            ) -> ::core::result::Result<
                <Self as ::rkyv::Archive>::Resolver,
                <__S as ::rkyv::rancor::Fallible>::Error,
            > {
                let __this = self;
                ::core::result::Result::Ok(TruncChunkResolver {
                    index: <usize as ::rkyv::Serialize<
                        __S,
                    >>::serialize(&__this.index, serializer)?,
                    data: <Vec<
                        u8,
                    > as ::rkyv::Serialize<__S>>::serialize(&__this.data, serializer)?,
                })
            }
        }
        #[automatically_derived]
        impl<__D: ::rkyv::rancor::Fallible + ?Sized> ::rkyv::Deserialize<TruncChunk, __D>
        for ::rkyv::Archived<TruncChunk>
        where
            usize: ::rkyv::Archive,
            <usize as ::rkyv::Archive>::Archived: ::rkyv::Deserialize<usize, __D>,
            Vec<u8>: ::rkyv::Archive,
            <Vec<u8> as ::rkyv::Archive>::Archived: ::rkyv::Deserialize<Vec<u8>, __D>,
        {
            fn deserialize(
                &self,
                deserializer: &mut __D,
            ) -> ::core::result::Result<
                TruncChunk,
                <__D as ::rkyv::rancor::Fallible>::Error,
            > {
                let __this = self;
                ::core::result::Result::Ok(TruncChunk {
                    index: <<usize as ::rkyv::Archive>::Archived as ::rkyv::Deserialize<
                        usize,
                        __D,
                    >>::deserialize(&__this.index, deserializer)?,
                    data: <<Vec<
                        u8,
                    > as ::rkyv::Archive>::Archived as ::rkyv::Deserialize<
                        Vec<u8>,
                        __D,
                    >>::deserialize(&__this.data, deserializer)?,
                })
            }
        }
        impl ::rewrite::traits::structural::blob::BlobItemChunk for TruncChunk {
            type Index = usize;
            fn get_index(&self) -> &Self::Index {
                &self.index
            }
        }
        pub enum TruncChunkFill {
            Full(usize),
            Partial(usize),
            Corrupted(usize),
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for TruncChunkFill {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                match self {
                    TruncChunkFill::Full(__self_0) => {
                        ::core::fmt::Formatter::debug_tuple_field1_finish(
                            f,
                            "Full",
                            &__self_0,
                        )
                    }
                    TruncChunkFill::Partial(__self_0) => {
                        ::core::fmt::Formatter::debug_tuple_field1_finish(
                            f,
                            "Partial",
                            &__self_0,
                        )
                    }
                    TruncChunkFill::Corrupted(__self_0) => {
                        ::core::fmt::Formatter::debug_tuple_field1_finish(
                            f,
                            "Corrupted",
                            &__self_0,
                        )
                    }
                }
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        unsafe impl ::core::clone::TrivialClone for TruncChunkFill {}
        #[automatically_derived]
        impl ::core::clone::Clone for TruncChunkFill {
            #[inline]
            fn clone(&self) -> TruncChunkFill {
                let _: ::core::clone::AssertParamIsClone<usize>;
                *self
            }
        }
        #[automatically_derived]
        impl ::core::marker::Copy for TruncChunkFill {}
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq for TruncChunkFill {}
        #[automatically_derived]
        impl ::core::cmp::PartialEq for TruncChunkFill {
            #[inline]
            fn eq(&self, other: &TruncChunkFill) -> bool {
                let __self_discr = ::core::intrinsics::discriminant_value(self);
                let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                __self_discr == __arg1_discr
                    && match (self, other) {
                        (
                            TruncChunkFill::Full(__self_0),
                            TruncChunkFill::Full(__arg1_0),
                        ) => __self_0 == __arg1_0,
                        (
                            TruncChunkFill::Partial(__self_0),
                            TruncChunkFill::Partial(__arg1_0),
                        ) => __self_0 == __arg1_0,
                        (
                            TruncChunkFill::Corrupted(__self_0),
                            TruncChunkFill::Corrupted(__arg1_0),
                        ) => __self_0 == __arg1_0,
                        _ => unsafe { ::core::intrinsics::unreachable() }
                    }
            }
        }
        #[automatically_derived]
        impl ::core::cmp::Eq for TruncChunkFill {
            #[inline]
            #[doc(hidden)]
            #[coverage(off)]
            fn assert_fields_are_eq(&self) {
                let _: ::core::cmp::AssertParamIsEq<usize>;
            }
        }
        #[automatically_derived]
        impl ::core::cmp::PartialOrd for TruncChunkFill {
            #[inline]
            fn partial_cmp(
                &self,
                other: &TruncChunkFill,
            ) -> ::core::option::Option<::core::cmp::Ordering> {
                let __self_discr = ::core::intrinsics::discriminant_value(self);
                let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                match (self, other) {
                    (TruncChunkFill::Full(__self_0), TruncChunkFill::Full(__arg1_0)) => {
                        ::core::cmp::PartialOrd::partial_cmp(__self_0, __arg1_0)
                    }
                    (
                        TruncChunkFill::Partial(__self_0),
                        TruncChunkFill::Partial(__arg1_0),
                    ) => ::core::cmp::PartialOrd::partial_cmp(__self_0, __arg1_0),
                    (
                        TruncChunkFill::Corrupted(__self_0),
                        TruncChunkFill::Corrupted(__arg1_0),
                    ) => ::core::cmp::PartialOrd::partial_cmp(__self_0, __arg1_0),
                    _ => {
                        ::core::cmp::PartialOrd::partial_cmp(
                            &__self_discr,
                            &__arg1_discr,
                        )
                    }
                }
            }
        }
        #[automatically_derived]
        impl ::core::cmp::Ord for TruncChunkFill {
            #[inline]
            fn cmp(&self, other: &TruncChunkFill) -> ::core::cmp::Ordering {
                let __self_discr = ::core::intrinsics::discriminant_value(self);
                let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                match ::core::cmp::Ord::cmp(&__self_discr, &__arg1_discr) {
                    ::core::cmp::Ordering::Equal => {
                        match (self, other) {
                            (
                                TruncChunkFill::Full(__self_0),
                                TruncChunkFill::Full(__arg1_0),
                            ) => ::core::cmp::Ord::cmp(__self_0, __arg1_0),
                            (
                                TruncChunkFill::Partial(__self_0),
                                TruncChunkFill::Partial(__arg1_0),
                            ) => ::core::cmp::Ord::cmp(__self_0, __arg1_0),
                            (
                                TruncChunkFill::Corrupted(__self_0),
                                TruncChunkFill::Corrupted(__arg1_0),
                            ) => ::core::cmp::Ord::cmp(__self_0, __arg1_0),
                            _ => unsafe { ::core::intrinsics::unreachable() }
                        }
                    }
                    cmp => cmp,
                }
            }
        }
        #[automatically_derived]
        ///An archived [`TruncChunkFill`]
        #[bytecheck(crate = ::rkyv::bytecheck)]
        #[repr(u8)]
        pub enum ArchivedTruncChunkFill
        where
            usize: ::rkyv::Archive,
            usize: ::rkyv::Archive,
            usize: ::rkyv::Archive,
        {
            ///The archived counterpart of [`TruncChunkFill::Full`]
            #[allow(dead_code)]
            Full(
                ///The archived counterpart of [`TruncChunkFill::Full::0`]
                <usize as ::rkyv::Archive>::Archived,
            ),
            ///The archived counterpart of [`TruncChunkFill::Partial`]
            #[allow(dead_code)]
            Partial(
                ///The archived counterpart of [`TruncChunkFill::Partial::0`]
                <usize as ::rkyv::Archive>::Archived,
            ),
            ///The archived counterpart of [`TruncChunkFill::Corrupted`]
            #[allow(dead_code)]
            Corrupted(
                ///The archived counterpart of [`TruncChunkFill::Corrupted::0`]
                <usize as ::rkyv::Archive>::Archived,
            ),
        }
        const _: () = {
            #[repr(u8)]
            enum Tag {
                Full,
                Partial,
                Corrupted,
            }
            struct Discriminant;
            #[automatically_derived]
            impl Discriminant {
                #[allow(non_upper_case_globals)]
                const Full: u8 = Tag::Full as u8;
                #[allow(non_upper_case_globals)]
                const Partial: u8 = Tag::Partial as u8;
                #[allow(non_upper_case_globals)]
                const Corrupted: u8 = Tag::Corrupted as u8;
            }
            #[repr(C)]
            struct VariantFull(
                Tag,
                <usize as ::rkyv::Archive>::Archived,
                ::core::marker::PhantomData<ArchivedTruncChunkFill>,
            )
            where
                usize: ::rkyv::Archive,
                usize: ::rkyv::Archive,
                usize: ::rkyv::Archive;
            #[repr(C)]
            struct VariantPartial(
                Tag,
                <usize as ::rkyv::Archive>::Archived,
                ::core::marker::PhantomData<ArchivedTruncChunkFill>,
            )
            where
                usize: ::rkyv::Archive,
                usize: ::rkyv::Archive,
                usize: ::rkyv::Archive;
            #[repr(C)]
            struct VariantCorrupted(
                Tag,
                <usize as ::rkyv::Archive>::Archived,
                ::core::marker::PhantomData<ArchivedTruncChunkFill>,
            )
            where
                usize: ::rkyv::Archive,
                usize: ::rkyv::Archive,
                usize: ::rkyv::Archive;
            #[automatically_derived]
            unsafe impl<
                __C: ::rkyv::bytecheck::rancor::Fallible + ?::core::marker::Sized,
            > ::rkyv::bytecheck::CheckBytes<__C> for ArchivedTruncChunkFill
            where
                usize: ::rkyv::Archive,
                usize: ::rkyv::Archive,
                usize: ::rkyv::Archive,
                <__C as ::rkyv::bytecheck::rancor::Fallible>::Error: ::rkyv::bytecheck::rancor::Source,
                <usize as ::rkyv::Archive>::Archived: ::rkyv::bytecheck::CheckBytes<__C>,
                <usize as ::rkyv::Archive>::Archived: ::rkyv::bytecheck::CheckBytes<__C>,
                <usize as ::rkyv::Archive>::Archived: ::rkyv::bytecheck::CheckBytes<__C>,
            {
                unsafe fn check_bytes(
                    value: *const Self,
                    context: &mut __C,
                ) -> ::core::result::Result<
                    (),
                    <__C as ::rkyv::bytecheck::rancor::Fallible>::Error,
                > {
                    let tag = *value.cast::<u8>();
                    match tag {
                        Discriminant::Full => {
                            let value = value.cast::<VariantFull>();
                            <<usize as ::rkyv::Archive>::Archived as ::rkyv::bytecheck::CheckBytes<
                                __C,
                            >>::check_bytes(&raw const (*value).1, context)
                                .map_err(|e| {
                                    <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Trace>::trace(
                                        e,
                                        ::rkyv::bytecheck::UnnamedEnumVariantCheckContext {
                                            enum_name: "ArchivedTruncChunkFill",
                                            variant_name: "Full",
                                            field_index: 1,
                                        },
                                    )
                                })?;
                        }
                        Discriminant::Partial => {
                            let value = value.cast::<VariantPartial>();
                            <<usize as ::rkyv::Archive>::Archived as ::rkyv::bytecheck::CheckBytes<
                                __C,
                            >>::check_bytes(&raw const (*value).1, context)
                                .map_err(|e| {
                                    <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Trace>::trace(
                                        e,
                                        ::rkyv::bytecheck::UnnamedEnumVariantCheckContext {
                                            enum_name: "ArchivedTruncChunkFill",
                                            variant_name: "Partial",
                                            field_index: 1,
                                        },
                                    )
                                })?;
                        }
                        Discriminant::Corrupted => {
                            let value = value.cast::<VariantCorrupted>();
                            <<usize as ::rkyv::Archive>::Archived as ::rkyv::bytecheck::CheckBytes<
                                __C,
                            >>::check_bytes(&raw const (*value).1, context)
                                .map_err(|e| {
                                    <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Trace>::trace(
                                        e,
                                        ::rkyv::bytecheck::UnnamedEnumVariantCheckContext {
                                            enum_name: "ArchivedTruncChunkFill",
                                            variant_name: "Corrupted",
                                            field_index: 1,
                                        },
                                    )
                                })?;
                        }
                        _ => {
                            return ::core::result::Result::Err(
                                <<__C as ::rkyv::bytecheck::rancor::Fallible>::Error as ::rkyv::bytecheck::rancor::Source>::new(::rkyv::bytecheck::InvalidEnumDiscriminantError {
                                    enum_name: "ArchivedTruncChunkFill",
                                    invalid_discriminant: tag,
                                }),
                            );
                        }
                    }
                    ::core::result::Result::Ok(())
                }
            }
        };
        #[automatically_derived]
        ///The resolver for an archived [`TruncChunkFill`]
        pub enum TruncChunkFillResolver
        where
            usize: ::rkyv::Archive,
            usize: ::rkyv::Archive,
            usize: ::rkyv::Archive,
        {
            ///The resolver for [`TruncChunkFill::Full`]
            #[allow(dead_code)]
            Full(<usize as ::rkyv::Archive>::Resolver),
            ///The resolver for [`TruncChunkFill::Partial`]
            #[allow(dead_code)]
            Partial(<usize as ::rkyv::Archive>::Resolver),
            ///The resolver for [`TruncChunkFill::Corrupted`]
            #[allow(dead_code)]
            Corrupted(<usize as ::rkyv::Archive>::Resolver),
        }
        const _: () = {
            #[repr(u8)]
            enum ArchivedTag {
                Full,
                Partial,
                Corrupted,
            }
            #[automatically_derived]
            impl ::core::marker::StructuralPartialEq for ArchivedTag {}
            #[automatically_derived]
            impl ::core::cmp::PartialEq for ArchivedTag {
                #[inline]
                fn eq(&self, other: &ArchivedTag) -> bool {
                    let __self_discr = ::core::intrinsics::discriminant_value(self);
                    let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                    __self_discr == __arg1_discr
                }
            }
            #[automatically_derived]
            impl ::core::cmp::PartialOrd for ArchivedTag {
                #[inline]
                fn partial_cmp(
                    &self,
                    other: &ArchivedTag,
                ) -> ::core::option::Option<::core::cmp::Ordering> {
                    let __self_discr = ::core::intrinsics::discriminant_value(self);
                    let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                    ::core::cmp::PartialOrd::partial_cmp(&__self_discr, &__arg1_discr)
                }
            }
            #[repr(C)]
            struct ArchivedVariantFull(
                ArchivedTag,
                <usize as ::rkyv::Archive>::Archived,
                ::core::marker::PhantomData<TruncChunkFill>,
            )
            where
                usize: ::rkyv::Archive,
                usize: ::rkyv::Archive,
                usize: ::rkyv::Archive;
            #[repr(C)]
            struct ArchivedVariantPartial(
                ArchivedTag,
                <usize as ::rkyv::Archive>::Archived,
                ::core::marker::PhantomData<TruncChunkFill>,
            )
            where
                usize: ::rkyv::Archive,
                usize: ::rkyv::Archive,
                usize: ::rkyv::Archive;
            #[repr(C)]
            struct ArchivedVariantCorrupted(
                ArchivedTag,
                <usize as ::rkyv::Archive>::Archived,
                ::core::marker::PhantomData<TruncChunkFill>,
            )
            where
                usize: ::rkyv::Archive,
                usize: ::rkyv::Archive,
                usize: ::rkyv::Archive;
            impl ::rkyv::Archive for TruncChunkFill
            where
                usize: ::rkyv::Archive,
                usize: ::rkyv::Archive,
                usize: ::rkyv::Archive,
            {
                type Archived = ArchivedTruncChunkFill;
                type Resolver = TruncChunkFillResolver;
                #[allow(clippy::unit_arg)]
                fn resolve(
                    &self,
                    resolver: <Self as ::rkyv::Archive>::Resolver,
                    out: ::rkyv::Place<<Self as ::rkyv::Archive>::Archived>,
                ) {
                    let __this = self;
                    match resolver {
                        TruncChunkFillResolver::Full(resolver_0) => {
                            match __this {
                                TruncChunkFill::Full(self_0, ..) => {
                                    let out = unsafe {
                                        out.cast_unchecked::<ArchivedVariantFull>()
                                    };
                                    let tag_ptr = unsafe { &raw mut (*out.ptr()).0 };
                                    unsafe {
                                        tag_ptr.write(ArchivedTag::Full);
                                    }
                                    let field_ptr = unsafe { &raw mut (*out.ptr()).1 };
                                    let out_field = unsafe {
                                        ::rkyv::Place::from_field_unchecked(out, field_ptr)
                                    };
                                    <usize as ::rkyv::Archive>::resolve(
                                        self_0,
                                        resolver_0,
                                        out_field,
                                    );
                                }
                                #[allow(unreachable_patterns)]
                                _ => unsafe { ::core::hint::unreachable_unchecked() }
                            }
                        }
                        TruncChunkFillResolver::Partial(resolver_0) => {
                            match __this {
                                TruncChunkFill::Partial(self_0, ..) => {
                                    let out = unsafe {
                                        out.cast_unchecked::<ArchivedVariantPartial>()
                                    };
                                    let tag_ptr = unsafe { &raw mut (*out.ptr()).0 };
                                    unsafe {
                                        tag_ptr.write(ArchivedTag::Partial);
                                    }
                                    let field_ptr = unsafe { &raw mut (*out.ptr()).1 };
                                    let out_field = unsafe {
                                        ::rkyv::Place::from_field_unchecked(out, field_ptr)
                                    };
                                    <usize as ::rkyv::Archive>::resolve(
                                        self_0,
                                        resolver_0,
                                        out_field,
                                    );
                                }
                                #[allow(unreachable_patterns)]
                                _ => unsafe { ::core::hint::unreachable_unchecked() }
                            }
                        }
                        TruncChunkFillResolver::Corrupted(resolver_0) => {
                            match __this {
                                TruncChunkFill::Corrupted(self_0, ..) => {
                                    let out = unsafe {
                                        out.cast_unchecked::<ArchivedVariantCorrupted>()
                                    };
                                    let tag_ptr = unsafe { &raw mut (*out.ptr()).0 };
                                    unsafe {
                                        tag_ptr.write(ArchivedTag::Corrupted);
                                    }
                                    let field_ptr = unsafe { &raw mut (*out.ptr()).1 };
                                    let out_field = unsafe {
                                        ::rkyv::Place::from_field_unchecked(out, field_ptr)
                                    };
                                    <usize as ::rkyv::Archive>::resolve(
                                        self_0,
                                        resolver_0,
                                        out_field,
                                    );
                                }
                                #[allow(unreachable_patterns)]
                                _ => unsafe { ::core::hint::unreachable_unchecked() }
                            }
                        }
                    }
                }
            }
        };
        unsafe impl ::rkyv::traits::Portable for ArchivedTruncChunkFill
        where
            usize: ::rkyv::Archive,
            usize: ::rkyv::Archive,
            usize: ::rkyv::Archive,
            <usize as ::rkyv::Archive>::Archived: ::rkyv::traits::Portable,
            <usize as ::rkyv::Archive>::Archived: ::rkyv::traits::Portable,
            <usize as ::rkyv::Archive>::Archived: ::rkyv::traits::Portable,
        {}
        #[automatically_derived]
        impl<__S: ::rkyv::rancor::Fallible + ?Sized> ::rkyv::Serialize<__S>
        for TruncChunkFill
        where
            usize: ::rkyv::Serialize<__S>,
            usize: ::rkyv::Serialize<__S>,
            usize: ::rkyv::Serialize<__S>,
        {
            fn serialize(
                &self,
                serializer: &mut __S,
            ) -> ::core::result::Result<
                <Self as ::rkyv::Archive>::Resolver,
                <__S as ::rkyv::rancor::Fallible>::Error,
            > {
                let __this = self;
                ::core::result::Result::Ok(
                    match __this {
                        TruncChunkFill::Full(_0, ..) => {
                            TruncChunkFillResolver::Full(
                                <usize as ::rkyv::Serialize<
                                    __S,
                                >>::serialize(_0, serializer)?,
                            )
                        }
                        TruncChunkFill::Partial(_0, ..) => {
                            TruncChunkFillResolver::Partial(
                                <usize as ::rkyv::Serialize<
                                    __S,
                                >>::serialize(_0, serializer)?,
                            )
                        }
                        TruncChunkFill::Corrupted(_0, ..) => {
                            TruncChunkFillResolver::Corrupted(
                                <usize as ::rkyv::Serialize<
                                    __S,
                                >>::serialize(_0, serializer)?,
                            )
                        }
                    },
                )
            }
        }
        #[automatically_derived]
        impl<
            __D: ::rkyv::rancor::Fallible + ?Sized,
        > ::rkyv::Deserialize<TruncChunkFill, __D> for ::rkyv::Archived<TruncChunkFill>
        where
            usize: ::rkyv::Archive,
            <usize as ::rkyv::Archive>::Archived: ::rkyv::Deserialize<usize, __D>,
            usize: ::rkyv::Archive,
            <usize as ::rkyv::Archive>::Archived: ::rkyv::Deserialize<usize, __D>,
            usize: ::rkyv::Archive,
            <usize as ::rkyv::Archive>::Archived: ::rkyv::Deserialize<usize, __D>,
        {
            fn deserialize(
                &self,
                deserializer: &mut __D,
            ) -> ::core::result::Result<
                TruncChunkFill,
                <__D as ::rkyv::rancor::Fallible>::Error,
            > {
                let __this = self;
                ::core::result::Result::Ok(
                    match __this {
                        Self::Full(_0, ..) => {
                            TruncChunkFill::Full(
                                <<usize as ::rkyv::Archive>::Archived as ::rkyv::Deserialize<
                                    usize,
                                    __D,
                                >>::deserialize(_0, deserializer)?,
                            )
                        }
                        Self::Partial(_0, ..) => {
                            TruncChunkFill::Partial(
                                <<usize as ::rkyv::Archive>::Archived as ::rkyv::Deserialize<
                                    usize,
                                    __D,
                                >>::deserialize(_0, deserializer)?,
                            )
                        }
                        Self::Corrupted(_0, ..) => {
                            TruncChunkFill::Corrupted(
                                <<usize as ::rkyv::Archive>::Archived as ::rkyv::Deserialize<
                                    usize,
                                    __D,
                                >>::deserialize(_0, deserializer)?,
                            )
                        }
                    },
                )
            }
        }
        impl TruncChunkFill {
            pub fn from_size(actual: usize, expected: usize) -> Self {
                if actual == expected {
                    Self::Full(actual)
                } else if actual < expected {
                    Self::Partial(actual)
                } else {
                    Self::Corrupted(actual)
                }
            }
        }
        impl rewrite::traits::structural::blob::NetabaseBlobItem for Trunc {
            type Chunk = TruncChunk;
            type BlobIter = std::vec::IntoIter<
                rewrite::results::NetabaseResult<Self::Chunk>,
            >;
            const DEFAULT_CHUNK_SIZE: usize = 0usize;
            fn into_chunks(
                self,
                size: rewrite::traits::structural::blob::ChunkSize,
            ) -> Box<dyn Iterator<Item = Self::Chunk>> {
                Box::new(self.into_chunks_iter(size).filter_map(|r| r.ok()))
            }
            fn into_chunks_iter(
                self,
                size: rewrite::traits::structural::blob::ChunkSize,
            ) -> Self::BlobIter {
                let serialized_data: rewrite::results::NetabaseResult<Vec<u8>> = (|| -> rewrite::results::NetabaseResult<
                    Vec<u8>,
                > {
                    Ok(
                        rkyv::to_bytes::<rkyv::rancor::Error>(&self)
                            .map_err(|e| rewrite::results::NetabaseError::Serialization(
                                ::alloc::__export::must_use({
                                    ::alloc::fmt::format(
                                        format_args!("rkyv serialization failed: {0:?}", e),
                                    )
                                }),
                            ))?
                            .to_vec(),
                    )
                })();
                let chunk_size = match size {
                    rewrite::traits::structural::blob::ChunkSize::Default => {
                        if Self::DEFAULT_CHUNK_SIZE > 0 {
                            Self::DEFAULT_CHUNK_SIZE
                        } else {
                            1024
                        }
                    }
                    rewrite::traits::structural::blob::ChunkSize::Size(n) => n,
                };
                match serialized_data {
                    Ok(data) => {
                        data.chunks(chunk_size)
                            .enumerate()
                            .map(|(index, chunk_data)| {
                                Ok(Self::Chunk {
                                    index,
                                    data: chunk_data.to_vec(),
                                })
                            })
                            .collect::<Vec<_>>()
                            .into_iter()
                    }
                    Err(e) => {
                        ::alloc::boxed::box_assume_init_into_vec_unsafe(
                                ::alloc::intrinsics::write_box_via_move(
                                    ::alloc::boxed::Box::new_uninit(),
                                    [Err(e)],
                                ),
                            )
                            .into_iter()
                    }
                }
            }
            fn try_from_chunks(
                chunks: impl Iterator<Item = Self::Chunk>,
                size: rewrite::traits::structural::blob::ChunkSize,
            ) -> rewrite::results::NetabaseResult<Self> {
                let mut sorted_chunks: Vec<_> = chunks.collect();
                sorted_chunks.sort_by_key(|c| c.index);
                if sorted_chunks.is_empty() {
                    return Err(
                        rewrite::results::NetabaseError::BlobReconstruction(
                            rewrite::results::BlobReconstructionError::MissingChunks,
                        ),
                    );
                }
                let chunk_size = match size {
                    rewrite::traits::structural::blob::ChunkSize::Default => {
                        if Self::DEFAULT_CHUNK_SIZE > 0 {
                            Self::DEFAULT_CHUNK_SIZE
                        } else {
                            1024
                        }
                    }
                    rewrite::traits::structural::blob::ChunkSize::Size(n) => n,
                };
                let mut missing_details = Vec::new();
                let mut next_expected = 0;
                let max_idx = sorted_chunks.last().map(|c| c.index).unwrap_or(0);
                for chunk in &sorted_chunks {
                    while chunk.index > next_expected {
                        missing_details
                            .push(
                                ::alloc::__export::must_use({
                                    ::alloc::fmt::format(
                                        format_args!(
                                            "{0:?}({{ Index: {1}, Size: {2} }})",
                                            TruncChunkFill::Full(chunk_size),
                                            next_expected,
                                            chunk_size,
                                        ),
                                    )
                                }),
                            );
                        next_expected += 1;
                    }
                    let fill = TruncChunkFill::from_size(chunk.data.len(), chunk_size);
                    match fill {
                        TruncChunkFill::Corrupted(size) => {
                            return Err(
                                rewrite::results::NetabaseError::BlobReconstruction(
                                    rewrite::results::BlobReconstructionError::InvalidChunkData(
                                        ::alloc::__export::must_use({
                                            ::alloc::fmt::format(
                                                format_args!(
                                                    "Corrupted chunk detected: {0:?}({{ Index: {1}, Size: {2} }}). Max allowed size is {3}.",
                                                    fill,
                                                    chunk.index,
                                                    size,
                                                    chunk_size,
                                                ),
                                            )
                                        }),
                                    ),
                                ),
                            );
                        }
                        TruncChunkFill::Partial(size) if chunk.index < max_idx => {
                            return Err(
                                rewrite::results::NetabaseError::BlobReconstruction(
                                    rewrite::results::BlobReconstructionError::InvalidChunkData(
                                        ::alloc::__export::must_use({
                                            ::alloc::fmt::format(
                                                format_args!(
                                                    "Unexpected partial chunk in middle of stream: {0:?}({{ Index: {1}, Size: {2} }}). Expected {3} bytes.",
                                                    fill,
                                                    chunk.index,
                                                    size,
                                                    chunk_size,
                                                ),
                                            )
                                        }),
                                    ),
                                ),
                            );
                        }
                        _ => {}
                    }
                    if chunk.index == next_expected {
                        next_expected += 1;
                    }
                }
                if !missing_details.is_empty() {
                    if let Some(last) = sorted_chunks.last() {
                        let fill = TruncChunkFill::from_size(
                            last.data.len(),
                            chunk_size,
                        );
                        if #[allow(non_exhaustive_omitted_patterns)]
                        match fill {
                            TruncChunkFill::Full(_) => true,
                            _ => false,
                        } {
                            missing_details
                                .push(
                                    ::alloc::__export::must_use({
                                        ::alloc::fmt::format(
                                            format_args!(
                                                "... (Stream truncated: last chunk was Full, expected more data after Index {0})",
                                                last.index,
                                            ),
                                        )
                                    }),
                                );
                        }
                    }
                }
                if !missing_details.is_empty() {
                    return Err(
                        rewrite::results::NetabaseError::BlobReconstruction(
                            rewrite::results::BlobReconstructionError::InvalidChunkData(
                                ::alloc::__export::must_use({
                                    ::alloc::fmt::format(
                                        format_args!(
                                            "Missing chunks: [{0}]. Total chunks present: {1}",
                                            missing_details.join(", "),
                                            sorted_chunks.len(),
                                        ),
                                    )
                                }),
                            ),
                        ),
                    );
                }
                let serialized_data: Vec<u8> = sorted_chunks
                    .into_iter()
                    .flat_map(|c| c.data)
                    .collect();
                Ok(
                    rkyv::from_bytes::<Self, rkyv::rancor::Error>(&serialized_data)
                        .map_err(|e| rewrite::results::NetabaseError::Serialization(
                            ::alloc::__export::must_use({
                                ::alloc::fmt::format(
                                    format_args!("rkyv deserialization failed: {0:?}", e),
                                )
                            }),
                        ))?,
                )
            }
            fn get_blob(&self) -> &Self::Chunk {
                {
                    ::core::panicking::panic_fmt(
                        format_args!(
                            "not implemented: {0}",
                            format_args!("get_blob() requires storing a chunk reference"),
                        ),
                    );
                }
            }
        }
        impl IntoIterator for Trunc {
            type Item = rewrite::results::NetabaseResult<TruncChunk>;
            type IntoIter = std::vec::IntoIter<Self::Item>;
            fn into_iter(self) -> Self::IntoIter {
                rewrite::traits::structural::blob::NetabaseBlobItem::into_chunks_iter(
                    self,
                    rewrite::traits::structural::blob::ChunkSize::Default,
                )
            }
        }
        let blob = Trunc {
            data: ::alloc::vec::from_elem(0u8, 1000),
        };
        let mut chunks: Vec<_> = blob.into_chunks(ChunkSize::Size(64)).collect();
        let n = chunks.len();
        chunks.truncate(n - 2);
        chunks.remove(1);
        let result = Trunc::try_from_chunks(chunks.into_iter(), ChunkSize::Size(64));
        if let Err(
            NetabaseError::BlobReconstruction(
                BlobReconstructionError::InvalidChunkData(ref s),
            ),
        ) = result {
            if !s.contains("Stream truncated") {
                {
                    ::core::panicking::panic_fmt(
                        format_args!(
                            "Error message did not contain \'Stream truncated\': {0}",
                            s,
                        ),
                    );
                }
            }
        } else {
            {
                ::core::panicking::panic_fmt(
                    format_args!("Expected InvalidChunkData error, got: {0:?}", result),
                );
            };
        }
    }
    extern crate test;
    #[rustc_test_marker = "tests::test_partial_struct_missing_field_chunks"]
    #[doc(hidden)]
    pub const test_partial_struct_missing_field_chunks: test::TestDescAndFn = test::TestDescAndFn {
        desc: test::TestDesc {
            name: test::StaticTestName(
                "tests::test_partial_struct_missing_field_chunks",
            ),
            ignore: false,
            ignore_message: ::core::option::Option::None,
            source_file: "tests/blob_comprehensive_test.rs",
            start_line: 216usize,
            start_col: 8usize,
            end_line: 216usize,
            end_col: 48usize,
            compile_fail: false,
            no_run: false,
            should_panic: test::ShouldPanic::No,
            test_type: test::TestType::IntegrationTest,
        },
        testfn: test::StaticTestFn(
            #[coverage(off)]
            || test::assert_test_result(test_partial_struct_missing_field_chunks()),
        ),
    };
    fn test_partial_struct_missing_field_chunks() {
        let blob = PartialFieldBlob {
            header: "H".to_string(),
            payload: ::alloc::boxed::box_assume_init_into_vec_unsafe(
                ::alloc::intrinsics::write_box_via_move(
                    ::alloc::boxed::Box::new_uninit(),
                    [1, 2, 3],
                ),
            ),
        };
        let chunks: Vec<_> = blob.into_chunks(ChunkSize::Default).collect();
        let filtered_chunks = chunks
            .into_iter()
            .filter(|c| {
                #[allow(non_exhaustive_omitted_patterns)]
                match c {
                    PartialFieldBlobChunk::Header(_) => true,
                    _ => false,
                }
            });
        let result = PartialFieldBlob::try_from_chunks(
            filtered_chunks,
            ChunkSize::Default,
        );
        if !#[allow(non_exhaustive_omitted_patterns)]
        match result {
            Err(
                NetabaseError::BlobReconstruction(BlobReconstructionError::MissingChunks),
            ) => true,
            _ => false,
        } {
            ::core::panicking::panic(
                "assertion failed: matches!(result,\n    Err(NetabaseError::BlobReconstruction(BlobReconstructionError::MissingChunks)))",
            )
        }
    }
    extern crate test;
    #[rustc_test_marker = "tests::test_simple_streaming"]
    #[doc(hidden)]
    pub const test_simple_streaming: test::TestDescAndFn = test::TestDescAndFn {
        desc: test::TestDesc {
            name: test::StaticTestName("tests::test_simple_streaming"),
            ignore: false,
            ignore_message: ::core::option::Option::None,
            source_file: "tests/blob_comprehensive_test.rs",
            start_line: 233usize,
            start_col: 8usize,
            end_line: 233usize,
            end_col: 29usize,
            compile_fail: false,
            no_run: false,
            should_panic: test::ShouldPanic::No,
            test_type: test::TestType::IntegrationTest,
        },
        testfn: test::StaticTestFn(
            #[coverage(off)]
            || test::assert_test_result(test_simple_streaming()),
        ),
    };
    fn test_simple_streaming() {
        let blob = SimpleStreamingBlob {
            data: ::alloc::boxed::box_assume_init_into_vec_unsafe(
                ::alloc::intrinsics::write_box_via_move(
                    ::alloc::boxed::Box::new_uninit(),
                    [1, 2, 3, 4, 5],
                ),
            ),
        };
        let iter = blob.clone().into_chunks_iter(ChunkSize::Size(2));
        let results: Vec<NetabaseResult<_>> = iter.collect();
        if !(results.len() >= 3) {
            ::core::panicking::panic("assertion failed: results.len() >= 3")
        }
        for res in &results {
            if !res.is_ok() {
                ::core::panicking::panic("assertion failed: res.is_ok()")
            }
        }
        let chunks = results.into_iter().map(|r| r.unwrap());
        let reconstructed = SimpleStreamingBlob::try_from_chunks(
                chunks,
                ChunkSize::Size(2),
            )
            .expect("Failed to reconstruct");
        match (&blob, &reconstructed) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    let kind = ::core::panicking::AssertKind::Eq;
                    ::core::panicking::assert_failed(
                        kind,
                        &*left_val,
                        &*right_val,
                        ::core::option::Option::None,
                    );
                }
            }
        };
    }
    extern crate test;
    #[rustc_test_marker = "tests::test_partial_blob_streaming"]
    #[doc(hidden)]
    pub const test_partial_blob_streaming: test::TestDescAndFn = test::TestDescAndFn {
        desc: test::TestDesc {
            name: test::StaticTestName("tests::test_partial_blob_streaming"),
            ignore: false,
            ignore_message: ::core::option::Option::None,
            source_file: "tests/blob_comprehensive_test.rs",
            start_line: 256usize,
            start_col: 8usize,
            end_line: 256usize,
            end_col: 35usize,
            compile_fail: false,
            no_run: false,
            should_panic: test::ShouldPanic::No,
            test_type: test::TestType::IntegrationTest,
        },
        testfn: test::StaticTestFn(
            #[coverage(off)]
            || test::assert_test_result(test_partial_blob_streaming()),
        ),
    };
    fn test_partial_blob_streaming() {
        let blob = PartialFieldBlob {
            header: "Hello".to_string(),
            payload: ::alloc::vec::from_elem(0u8, 100),
        };
        let iter = blob.clone().into_chunks_iter(ChunkSize::Default);
        let results: Vec<NetabaseResult<_>> = iter.collect();
        match (&results.len(), &2) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    let kind = ::core::panicking::AssertKind::Eq;
                    ::core::panicking::assert_failed(
                        kind,
                        &*left_val,
                        &*right_val,
                        ::core::option::Option::None,
                    );
                }
            }
        };
        for res in &results {
            if !res.is_ok() {
                ::core::panicking::panic("assertion failed: res.is_ok()")
            }
        }
        let chunks = results.into_iter().map(|r| r.unwrap());
        let reconstructed = PartialFieldBlob::try_from_chunks(chunks, ChunkSize::Default)
            .expect("Failed to reconstruct");
        match (&blob, &reconstructed) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    let kind = ::core::panicking::AssertKind::Eq;
                    ::core::panicking::assert_failed(
                        kind,
                        &*left_val,
                        &*right_val,
                        ::core::option::Option::None,
                    );
                }
            }
        };
    }
    extern crate test;
    #[rustc_test_marker = "tests::test_into_iterator"]
    #[doc(hidden)]
    pub const test_into_iterator: test::TestDescAndFn = test::TestDescAndFn {
        desc: test::TestDesc {
            name: test::StaticTestName("tests::test_into_iterator"),
            ignore: false,
            ignore_message: ::core::option::Option::None,
            source_file: "tests/blob_comprehensive_test.rs",
            start_line: 283usize,
            start_col: 8usize,
            end_line: 283usize,
            end_col: 26usize,
            compile_fail: false,
            no_run: false,
            should_panic: test::ShouldPanic::No,
            test_type: test::TestType::IntegrationTest,
        },
        testfn: test::StaticTestFn(
            #[coverage(off)]
            || test::assert_test_result(test_into_iterator()),
        ),
    };
    fn test_into_iterator() {
        let blob = SimpleStreamingBlob {
            data: ::alloc::boxed::box_assume_init_into_vec_unsafe(
                ::alloc::intrinsics::write_box_via_move(
                    ::alloc::boxed::Box::new_uninit(),
                    [1, 2, 3, 4, 5],
                ),
            ),
        };
        let mut results = Vec::new();
        for res in blob.clone() {
            results.push(res);
        }
        if !(results.len() >= 1) {
            ::core::panicking::panic("assertion failed: results.len() >= 1")
        }
        for res in &results {
            if !res.is_ok() {
                ::core::panicking::panic("assertion failed: res.is_ok()")
            }
        }
        let chunks = results.into_iter().map(|r| r.unwrap());
        let reconstructed = SimpleStreamingBlob::try_from_chunks(
                chunks,
                ChunkSize::Default,
            )
            .expect("Failed to reconstruct from IntoIterator");
        match (&blob, &reconstructed) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    let kind = ::core::panicking::AssertKind::Eq;
                    ::core::panicking::assert_failed(
                        kind,
                        &*left_val,
                        &*right_val,
                        ::core::option::Option::None,
                    );
                }
            }
        };
    }
    extern crate test;
    #[rustc_test_marker = "tests::test_generic_blob"]
    #[doc(hidden)]
    pub const test_generic_blob: test::TestDescAndFn = test::TestDescAndFn {
        desc: test::TestDesc {
            name: test::StaticTestName("tests::test_generic_blob"),
            ignore: false,
            ignore_message: ::core::option::Option::None,
            source_file: "tests/blob_comprehensive_test.rs",
            start_line: 308usize,
            start_col: 8usize,
            end_line: 308usize,
            end_col: 25usize,
            compile_fail: false,
            no_run: false,
            should_panic: test::ShouldPanic::No,
            test_type: test::TestType::IntegrationTest,
        },
        testfn: test::StaticTestFn(
            #[coverage(off)]
            || test::assert_test_result(test_generic_blob()),
        ),
    };
    fn test_generic_blob() {
        let blob = GenericBlob {
            data: "Generic Data".to_string(),
        };
        let chunks: Vec<_> = blob.clone().into_chunks(ChunkSize::Size(4)).collect();
        if !(chunks.len() >= 1) {
            ::core::panicking::panic("assertion failed: chunks.len() >= 1")
        }
        let reconstructed = GenericBlob::<
            String,
        >::try_from_chunks(chunks.into_iter(), ChunkSize::Size(4))
            .expect("Failed to reconstruct GenericBlob");
        match (&blob, &reconstructed) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    let kind = ::core::panicking::AssertKind::Eq;
                    ::core::panicking::assert_failed(
                        kind,
                        &*left_val,
                        &*right_val,
                        ::core::option::Option::None,
                    );
                }
            }
        };
    }
    extern crate test;
    #[rustc_test_marker = "tests::test_partial_enum_full"]
    #[doc(hidden)]
    pub const test_partial_enum_full: test::TestDescAndFn = test::TestDescAndFn {
        desc: test::TestDesc {
            name: test::StaticTestName("tests::test_partial_enum_full"),
            ignore: false,
            ignore_message: ::core::option::Option::None,
            source_file: "tests/blob_comprehensive_test.rs",
            start_line: 323usize,
            start_col: 8usize,
            end_line: 323usize,
            end_col: 30usize,
            compile_fail: false,
            no_run: false,
            should_panic: test::ShouldPanic::No,
            test_type: test::TestType::IntegrationTest,
        },
        testfn: test::StaticTestFn(
            #[coverage(off)]
            || test::assert_test_result(test_partial_enum_full()),
        ),
    };
    fn test_partial_enum_full() {
        let blob = PartialComplexEnum::Full("Hello Full".to_string());
        let chunks: Vec<_> = blob.clone().into_chunks(ChunkSize::Default).collect();
        if !(chunks.len() >= 1) {
            ::core::panicking::panic("assertion failed: chunks.len() >= 1")
        }
        let reconstructed = PartialComplexEnum::try_from_chunks(
                chunks.into_iter(),
                ChunkSize::Default,
            )
            .expect("Failed to reconstruct PartialComplexEnum::Full");
        match (&blob, &reconstructed) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    let kind = ::core::panicking::AssertKind::Eq;
                    ::core::panicking::assert_failed(
                        kind,
                        &*left_val,
                        &*right_val,
                        ::core::option::Option::None,
                    );
                }
            }
        };
    }
    extern crate test;
    #[rustc_test_marker = "tests::test_partial_enum_partial"]
    #[doc(hidden)]
    pub const test_partial_enum_partial: test::TestDescAndFn = test::TestDescAndFn {
        desc: test::TestDesc {
            name: test::StaticTestName("tests::test_partial_enum_partial"),
            ignore: false,
            ignore_message: ::core::option::Option::None,
            source_file: "tests/blob_comprehensive_test.rs",
            start_line: 336usize,
            start_col: 8usize,
            end_line: 336usize,
            end_col: 33usize,
            compile_fail: false,
            no_run: false,
            should_panic: test::ShouldPanic::No,
            test_type: test::TestType::IntegrationTest,
        },
        testfn: test::StaticTestFn(
            #[coverage(off)]
            || test::assert_test_result(test_partial_enum_partial()),
        ),
    };
    fn test_partial_enum_partial() {
        let blob = PartialComplexEnum::Partial {
            meta: "Metadata".to_string(),
            payload: ::alloc::vec::from_elem(0u8, 100),
        };
        let chunks: Vec<_> = blob.clone().into_chunks(ChunkSize::Default).collect();
        match (&chunks.len(), &3) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    let kind = ::core::panicking::AssertKind::Eq;
                    ::core::panicking::assert_failed(
                        kind,
                        &*left_val,
                        &*right_val,
                        ::core::option::Option::None,
                    );
                }
            }
        };
        let reconstructed = PartialComplexEnum::try_from_chunks(
                chunks.into_iter(),
                ChunkSize::Default,
            )
            .expect("Failed to reconstruct PartialComplexEnum::Partial");
        match (&blob, &reconstructed) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    let kind = ::core::panicking::AssertKind::Eq;
                    ::core::panicking::assert_failed(
                        kind,
                        &*left_val,
                        &*right_val,
                        ::core::option::Option::None,
                    );
                }
            }
        };
    }
    extern crate test;
    #[rustc_test_marker = "tests::test_forced_full"]
    #[doc(hidden)]
    pub const test_forced_full: test::TestDescAndFn = test::TestDescAndFn {
        desc: test::TestDesc {
            name: test::StaticTestName("tests::test_forced_full"),
            ignore: false,
            ignore_message: ::core::option::Option::None,
            source_file: "tests/blob_comprehensive_test.rs",
            start_line: 355usize,
            start_col: 8usize,
            end_line: 355usize,
            end_col: 24usize,
            compile_fail: false,
            no_run: false,
            should_panic: test::ShouldPanic::No,
            test_type: test::TestType::IntegrationTest,
        },
        testfn: test::StaticTestFn(
            #[coverage(off)]
            || test::assert_test_result(test_forced_full()),
        ),
    };
    fn test_forced_full() {
        let blob = ForcedFull {
            field1: "Hello".to_string(),
        };
        let chunks: Vec<_> = blob.into_chunks(ChunkSize::Size(1024)).collect();
        match (&chunks.len(), &1) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    let kind = ::core::panicking::AssertKind::Eq;
                    ::core::panicking::assert_failed(
                        kind,
                        &*left_val,
                        &*right_val,
                        ::core::option::Option::None,
                    );
                }
            }
        };
    }
    extern crate test;
    #[rustc_test_marker = "tests::test_forced_partial"]
    #[doc(hidden)]
    pub const test_forced_partial: test::TestDescAndFn = test::TestDescAndFn {
        desc: test::TestDesc {
            name: test::StaticTestName("tests::test_forced_partial"),
            ignore: false,
            ignore_message: ::core::option::Option::None,
            source_file: "tests/blob_comprehensive_test.rs",
            start_line: 362usize,
            start_col: 8usize,
            end_line: 362usize,
            end_col: 27usize,
            compile_fail: false,
            no_run: false,
            should_panic: test::ShouldPanic::No,
            test_type: test::TestType::IntegrationTest,
        },
        testfn: test::StaticTestFn(
            #[coverage(off)]
            || test::assert_test_result(test_forced_partial()),
        ),
    };
    fn test_forced_partial() {
        let blob = ForcedPartial {
            field1: "Hello".to_string(),
        };
        let chunks: Vec<_> = blob.into_chunks(ChunkSize::Default).collect();
        if !(chunks.len() >= 1) {
            ::core::panicking::panic("assertion failed: chunks.len() >= 1")
        }
    }
    extern crate test;
    #[rustc_test_marker = "tests::test_standalone_strategy"]
    #[doc(hidden)]
    pub const test_standalone_strategy: test::TestDescAndFn = test::TestDescAndFn {
        desc: test::TestDesc {
            name: test::StaticTestName("tests::test_standalone_strategy"),
            ignore: false,
            ignore_message: ::core::option::Option::None,
            source_file: "tests/blob_comprehensive_test.rs",
            start_line: 369usize,
            start_col: 8usize,
            end_line: 369usize,
            end_col: 32usize,
            compile_fail: false,
            no_run: false,
            should_panic: test::ShouldPanic::No,
            test_type: test::TestType::IntegrationTest,
        },
        testfn: test::StaticTestFn(
            #[coverage(off)]
            || test::assert_test_result(test_standalone_strategy()),
        ),
    };
    fn test_standalone_strategy() {
        let blob = StandaloneStrategy {
            field1: "Hello".to_string(),
        };
        let chunks: Vec<_> = blob.into_chunks(ChunkSize::Default).collect();
        if !(chunks.len() >= 1) {
            ::core::panicking::panic("assertion failed: chunks.len() >= 1")
        }
    }
}
#[rustc_main]
#[coverage(off)]
#[doc(hidden)]
pub fn main() -> () {
    extern crate test;
    test::test_main_static(
        &[
            &test_error_corrupted_chunk_size,
            &test_error_partial_chunk_in_middle,
            &test_error_truncated_stream,
            &test_forced_full,
            &test_forced_partial,
            &test_full_enum_blob,
            &test_generic_blob,
            &test_into_iterator,
            &test_nested_blob_partial,
            &test_partial_blob_streaming,
            &test_partial_enum_full,
            &test_partial_enum_partial,
            &test_partial_field_blob,
            &test_partial_struct_missing_field_chunks,
            &test_simple_streaming,
            &test_standalone_strategy,
        ],
    )
}
