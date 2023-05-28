#![allow(non_snake_case)]

use std::mem::{self, size_of};
use std::{alloc, marker::PhantomData};

use pasta_curves::Ep;
use rkyv::Serialize;
use rkyv::{
    ser::{ScratchSpace, Serializer},
    vec::{ArchivedVec, RawArchivedVec, VecResolver},
    with::{ArchiveWith, DeserializeWith, SerializeWith},
    Archive, Deserialize, DeserializeUnsized, Fallible,
};

pub struct Raw<T> {
    _t: PhantomData<T>,
}

impl<T, A: Archive> ArchiveWith<T> for Raw<A> {
    type Archived = A::Archived;
    type Resolver = A::Resolver;

    #[inline]
    unsafe fn resolve_with(
        field: &T,
        pos: usize,
        resolver: Self::Resolver,
        out: *mut Self::Archived,
    ) {
        let field = (field as *const T) as *const A;
        (*field).resolve(pos, resolver, out);
    }
}

impl<T, S, A> SerializeWith<T, S> for Raw<A>
where
    A: Serialize<S>,
    S: Serializer + ?Sized,
{
    fn serialize_with(field: &T, serializer: &mut S) -> Result<Self::Resolver, S::Error> {
        let field = (field as *const T) as *const A;
        unsafe { (*field).serialize(serializer) }
    }
}

impl<T, D, A> DeserializeWith<A::Archived, T, D> for Raw<A>
where
    A: Archive,
    A::Archived: Deserialize<A, D>,
    D: Fallible + ?Sized,
{
    fn deserialize_with(field: &A::Archived, deserializer: &mut D) -> Result<T, D::Error> {
        unsafe {
            let a: A = field.deserialize(deserializer)?;
            let a_clone = mem::ManuallyDrop::new(a); // uhhh... arcane things
            let t: T = mem::transmute_copy(&a_clone); // quite possibly the most dangerous thing you can do :P
            Ok(t)
        }
    }
}
