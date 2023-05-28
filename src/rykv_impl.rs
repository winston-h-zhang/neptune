use ff::PrimeField;
use rkyv::{
    out_field,
    ser::Serializer,
    vec::ArchivedVec,
    with::{self, ArchiveWith, DeserializeWith, SerializeWith, With},
    Archive, Archived, Deserialize, Fallible, Resolver, Serialize,
};
use std::fmt;
use std::marker::PhantomData;

use crate::{hash_type::ArchivedHashType, mds::ArchivedMdsMatrices, ArchivedStrength, Arity};
use crate::{
    hash_type::{CType, HashType},
    matrix::Matrix,
    mds::{MdsMatrices, RawMatrix, RawVec, SparseMatrix},
    unsafe_rkyv::Raw,
    Strength,
};
use crate::{mds::ArchivedSparseMatrix, poseidon::PoseidonConstants};

pub struct ArchivedCType;

impl<F: PrimeField, A: Arity<F>> ArchiveWith<CType<F, A>> for ArchivedCType {
    type Archived = <Option<u64> as Archive>::Archived;
    type Resolver = <Option<u64> as Archive>::Resolver;

    #[inline]
    unsafe fn resolve_with(
        field: &CType<F, A>,
        pos: usize,
        resolver: Self::Resolver,
        out: *mut Self::Archived,
    ) {
        match field {
            CType::Arbitrary(id) => Some(*id).resolve(pos, resolver, out),
            CType::_Phantom(_) => unimplemented!(),
        }
    }
}

impl<F: PrimeField, A: Arity<F>, S> SerializeWith<CType<F, A>, S> for ArchivedCType
where
    S: Serializer + ?Sized,
{
    fn serialize_with(field: &CType<F, A>, serializer: &mut S) -> Result<Self::Resolver, S::Error> {
        match field {
            CType::Arbitrary(id) => rkyv::Serialize::serialize(&Some(*id), serializer),
            CType::_Phantom(_) => unimplemented!(), // rkyv::Serialize::serialize(&None::<u64>, serializer),
        }
    }
}

impl<F: PrimeField, A: Arity<F>, D>
    DeserializeWith<<Option<u64> as Archive>::Archived, CType<F, A>, D> for ArchivedCType
where
    D: Fallible + ?Sized,
{
    fn deserialize_with(
        field: &<Option<u64> as Archive>::Archived,
        deserializer: &mut D,
    ) -> Result<CType<F, A>, D::Error> {
        match rkyv::Deserialize::<Option<u64>, D>::deserialize(field, deserializer)? {
            Some(id) => Ok(CType::Arbitrary(id)),
            None => unimplemented!(),
        }
    }
}

/// An archived [`PoseidonConstants`]
pub struct ArchivedPoseidonConstants<F, A>
where
    F: PrimeField,
    A: Arity<F>,
    MdsMatrices<F>: Archive,
    With<Vec<F>, RawVec<F>>: Archive,
    With<Matrix<F>, RawMatrix<F>>: Archive,
    Vec<SparseMatrix<F>>: Archive,
    Strength: Archive,
    With<F, Raw<F::Repr>>: Archive,
    usize: Archive,
    HashType<F, A>: Archive,
    PhantomData<A>: Archive,
{
    /// The archived counterpart of [`PoseidonConstants::mds_matrices`]
    pub mds_matrices: Archived<MdsMatrices<F>>,
    /// The archived counterpart of [`PoseidonConstants::compressed_round_constants`]
    pub compressed_round_constants: Archived<With<Vec<F>, RawVec<F>>>,
    /// The archived counterpart of [`PoseidonConstants::pre_sparse_matrix`]
    pub pre_sparse_matrix: Archived<With<Matrix<F>, RawMatrix<F>>>,
    /// The archived counterpart of [`PoseidonConstants::sparse_matrixes`]
    pub sparse_matrixes: Archived<Vec<SparseMatrix<F>>>,
    /// The archived counterpart of [`PoseidonConstants::strength`]
    pub strength: Archived<Strength>,
    /// The archived counterpart of [`PoseidonConstants::domain_tag`]
    pub domain_tag: Archived<With<F, Raw<F::Repr>>>,
    /// The archived counterpart of [`PoseidonConstants::full_rounds`]
    pub full_rounds: Archived<usize>,
    /// The archived counterpart of [`PoseidonConstants::partial_rounds`]
    pub partial_rounds: Archived<usize>,
    /// The archived counterpart of [`PoseidonConstants::hash_type`]
    pub hash_type: Archived<HashType<F, A>>,
    /// The archived counterpart of [`PoseidonConstants::_a`]
    pub(crate) _a: Archived<PhantomData<A>>,
}

/// The resolver for an archived [`PoseidonConstants`]
pub struct PoseidonConstantsResolver<F, A>
where
    F: PrimeField,
    A: Arity<F>,
    MdsMatrices<F>: Archive,
    With<Vec<F>, RawVec<F>>: Archive,
    With<Matrix<F>, RawMatrix<F>>: Archive,
    Vec<SparseMatrix<F>>: Archive,
    Strength: Archive,
    With<F, Raw<F::Repr>>: Archive,
    usize: Archive,
    HashType<F, A>: Archive,
    PhantomData<A>: Archive,
{
    mds_matrices: Resolver<MdsMatrices<F>>,
    compressed_round_constants: Resolver<With<Vec<F>, RawVec<F>>>,
    pre_sparse_matrix: Resolver<With<Matrix<F>, RawMatrix<F>>>,
    sparse_matrixes: Resolver<Vec<SparseMatrix<F>>>,
    strength: Resolver<Strength>,
    domain_tag: Resolver<With<F, Raw<F::Repr>>>,
    full_rounds: Resolver<usize>,
    partial_rounds: Resolver<usize>,
    hash_type: Resolver<HashType<F, A>>,
    _a: Resolver<PhantomData<A>>,
}
impl<F, A> Archive for PoseidonConstants<F, A>
where
    F: PrimeField,
    A: Arity<F>,
    MdsMatrices<F>: Archive,
    With<Vec<F>, RawVec<F>>: Archive,
    With<Matrix<F>, RawMatrix<F>>: Archive,
    Vec<SparseMatrix<F>>: Archive,
    Strength: Archive,
    With<F, Raw<F::Repr>>: Archive,
    usize: Archive,
    HashType<F, A>: Archive,
    PhantomData<A>: Archive,
{
    type Archived = ArchivedPoseidonConstants<F, A>;
    type Resolver = PoseidonConstantsResolver<F, A>;
    #[allow(clippy::unit_arg)]
    #[inline]
    unsafe fn resolve(&self, pos: usize, resolver: Self::Resolver, out: *mut Self::Archived) {
        let (fp, fo) = {
            #[allow(unused_unsafe)]
            unsafe {
                let fo = &mut (*out).mds_matrices as *mut <MdsMatrices<F> as Archive>::Archived;
                (fo.cast::<u8>().offset_from(out.cast::<u8>()) as usize, fo)
            }
        };
        Archive::resolve(&self.mds_matrices, pos + fp, resolver.mds_matrices, fo);
        let (fp, fo) = {
            #[allow(unused_unsafe)]
            unsafe {
                let fo = &mut (*out).compressed_round_constants
                    as *mut <With<Vec<F>, Raw<Vec<F::Repr>>> as Archive>::Archived;
                (fo.cast::<u8>().offset_from(out.cast::<u8>()) as usize, fo)
            }
        };
        Archive::resolve(
            With::<_, RawVec<F>>::cast(&self.compressed_round_constants),
            pos + fp,
            resolver.compressed_round_constants,
            fo,
        );
        let (fp, fo) = {
            #[allow(unused_unsafe)]
            unsafe {
                let fo = &mut (*out).pre_sparse_matrix
                    as *mut <With<Vec<Vec<F>>, Raw<Vec<Vec<F::Repr>>>> as Archive>::Archived;
                (fo.cast::<u8>().offset_from(out.cast::<u8>()) as usize, fo)
            }
        };
        Archive::resolve(
            With::<_, RawMatrix<F>>::cast(&self.pre_sparse_matrix),
            pos + fp,
            resolver.pre_sparse_matrix,
            fo,
        );
        let (fp, fo) = {
            #[allow(unused_unsafe)]
            unsafe {
                let fo =
                    &mut (*out).sparse_matrixes as *mut <Vec<SparseMatrix<F>> as Archive>::Archived;
                (fo.cast::<u8>().offset_from(out.cast::<u8>()) as usize, fo)
            }
        };
        Archive::resolve(
            &self.sparse_matrixes,
            pos + fp,
            resolver.sparse_matrixes,
            fo,
        );
        let (fp, fo) = {
            #[allow(unused_unsafe)]
            unsafe {
                let fo = &mut (*out).strength as *mut ArchivedStrength;
                (fo.cast::<u8>().offset_from(out.cast::<u8>()) as usize, fo)
            }
        };
        Archive::resolve(&self.strength, pos + fp, resolver.strength, fo);
        let (fp, fo) = {
            #[allow(unused_unsafe)]
            unsafe {
                let fo =
                    &mut (*out).domain_tag as *mut <With<F, Raw<F::Repr>> as Archive>::Archived;
                (fo.cast::<u8>().offset_from(out.cast::<u8>()) as usize, fo)
            }
        };
        Archive::resolve(
            With::<_, Raw<F::Repr>>::cast(&self.domain_tag),
            pos + fp,
            resolver.domain_tag,
            fo,
        );
        let (fp, fo) = {
            #[allow(unused_unsafe)]
            unsafe {
                let fo = &mut (*out).full_rounds as *mut u64;
                (fo.cast::<u8>().offset_from(out.cast::<u8>()) as usize, fo)
            }
        };
        Archive::resolve(&self.full_rounds, pos + fp, resolver.full_rounds, fo);
        let (fp, fo) = {
            #[allow(unused_unsafe)]
            unsafe {
                let fo = &mut (*out).partial_rounds as *mut u64;
                (fo.cast::<u8>().offset_from(out.cast::<u8>()) as usize, fo)
            }
        };
        Archive::resolve(&self.partial_rounds, pos + fp, resolver.partial_rounds, fo);
        let (fp, fo) = {
            #[allow(unused_unsafe)]
            unsafe {
                let fo = &mut (*out).hash_type as *mut <HashType<F, A> as Archive>::Archived;
                (fo.cast::<u8>().offset_from(out.cast::<u8>()) as usize, fo)
            }
        };
        Archive::resolve(&self.hash_type, pos + fp, resolver.hash_type, fo);
        let (fp, fo) = {
            #[allow(unused_unsafe)]
            unsafe {
                let fo = &mut (*out)._a as *mut <PhantomData<A> as Archive>::Archived;
                (fo.cast::<u8>().offset_from(out.cast::<u8>()) as usize, fo)
            }
        };
        Archive::resolve(&self._a, pos + fp, resolver._a, fo);
    }
}

impl<S: Fallible + ?Sized, F, A> Serialize<S> for PoseidonConstants<F, A>
where
    F: PrimeField,
    A: Arity<F>,
    MdsMatrices<F>: Serialize<S>,
    With<Option<Vec<F>>, Raw<Option<Vec<F::Repr>>>>: Serialize<S>,
    With<Vec<F>, RawVec<F>>: Serialize<S>,
    With<Matrix<F>, RawMatrix<F>>: Serialize<S>,
    Vec<SparseMatrix<F>>: Serialize<S>,
    Strength: Serialize<S>,
    With<F, Raw<F::Repr>>: Serialize<S>,
    usize: Serialize<S>,
    HashType<F, A>: Serialize<S>,
    PhantomData<A>: Serialize<S>,
{
    #[inline]
    fn serialize(&self, serializer: &mut S) -> Result<Self::Resolver, S::Error> {
        Ok(PoseidonConstantsResolver {
            mds_matrices: Serialize::<S>::serialize(&self.mds_matrices, serializer)?,
            compressed_round_constants: Serialize::<S>::serialize(
                With::<_, RawVec<F>>::cast(&self.compressed_round_constants),
                serializer,
            )?,
            pre_sparse_matrix: Serialize::<S>::serialize(
                With::<_, RawMatrix<F>>::cast(&self.pre_sparse_matrix),
                serializer,
            )?,
            sparse_matrixes: Serialize::<S>::serialize(&self.sparse_matrixes, serializer)?,
            strength: Serialize::<S>::serialize(&self.strength, serializer)?,
            domain_tag: Serialize::<S>::serialize(
                With::<_, Raw<F::Repr>>::cast(&self.domain_tag),
                serializer,
            )?,
            full_rounds: Serialize::<S>::serialize(&self.full_rounds, serializer)?,
            partial_rounds: Serialize::<S>::serialize(&self.partial_rounds, serializer)?,
            hash_type: Serialize::<S>::serialize(&self.hash_type, serializer)?,
            _a: Serialize::<S>::serialize(&self._a, serializer)?,
        })
    }
}

impl<D: Fallible + ?Sized, F, A> Deserialize<PoseidonConstants<F, A>, D>
    for Archived<PoseidonConstants<F, A>>
where
    F: PrimeField,
    A: Arity<F>,
    MdsMatrices<F>: Archive,
    Archived<MdsMatrices<F>>: Deserialize<MdsMatrices<F>, D>,
    With<Option<Vec<F>>, Raw<Option<Vec<F::Repr>>>>: Archive,
    Archived<With<Option<Vec<F>>, Raw<Option<Vec<F::Repr>>>>>:
        Deserialize<With<Option<Vec<F>>, Raw<Option<Vec<F::Repr>>>>, D>,
    With<Vec<F>, RawVec<F>>: Archive,
    Archived<With<Vec<F>, RawVec<F>>>: Deserialize<With<Vec<F>, RawVec<F>>, D>,
    With<Matrix<F>, RawMatrix<F>>: Archive,
    Archived<With<Matrix<F>, RawMatrix<F>>>: Deserialize<With<Matrix<F>, RawMatrix<F>>, D>,
    Vec<SparseMatrix<F>>: Archive,
    Archived<Vec<SparseMatrix<F>>>: Deserialize<Vec<SparseMatrix<F>>, D>,
    Strength: Archive,
    Archived<Strength>: Deserialize<Strength, D>,
    With<F, Raw<F::Repr>>: Archive,
    Archived<With<F, Raw<F::Repr>>>: Deserialize<With<F, Raw<F::Repr>>, D>,
    usize: Archive,
    Archived<usize>: Deserialize<usize, D>,
    HashType<F, A>: Archive,
    Archived<HashType<F, A>>: Deserialize<HashType<F, A>, D>,
    PhantomData<A>: Archive,
    Archived<PhantomData<A>>: Deserialize<PhantomData<A>, D>,
{
    #[inline]
    fn deserialize(
        &self,
        deserializer: &mut D,
    ) -> ::core::result::Result<PoseidonConstants<F, A>, D::Error> {
        let mds_matrices = self.mds_matrices.deserialize(deserializer)?;
        let compressed_round_constants = self
            .compressed_round_constants
            .deserialize(deserializer)?
            .into_inner();
        let pre_sparse_matrix = self
            .pre_sparse_matrix
            .deserialize(deserializer)?
            .into_inner();
        let sparse_matrixes = self.sparse_matrixes.deserialize(deserializer)?;
        let strength = self.strength.deserialize(deserializer)?;
        let domain_tag = self.domain_tag.deserialize(deserializer)?.into_inner();
        let full_rounds = self.full_rounds.deserialize(deserializer)?;
        let partial_rounds = self.partial_rounds.deserialize(deserializer)?;
        let hash_type = self.hash_type.deserialize(deserializer)?;
        let _a = self._a.deserialize(deserializer)?;
        Ok(PoseidonConstants {
            mds_matrices,
            round_constants: None,
            compressed_round_constants,
            pre_sparse_matrix,
            sparse_matrixes,
            strength,
            domain_tag,
            full_rounds,
            half_full_rounds: full_rounds / 2,
            partial_rounds,
            hash_type,
            _a,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Poseidon;
    use blstrs::Scalar as Fr;
    use ff::Field;
    use generic_array::typenum;
    use pasta_curves::pallas::Scalar as S1;
    use typenum::{U1, U2};

    #[test]
    fn serde_roundtrip() {
        let mut constants = PoseidonConstants::<S1, U2>::new();
        constants.round_constants = None;

        assert_eq!(
            constants,
            bincode::deserialize(&bincode::serialize(&constants).unwrap()).unwrap()
        );
        assert_eq!(
            constants,
            serde_json::from_slice(&serde_json::to_vec(&constants).unwrap()).unwrap()
        );
    }

    #[test]
    fn serde_hash_blstrs() {
        let constants = PoseidonConstants::<Fr, U2>::new();
        let constants2 = bincode::deserialize(&bincode::serialize(&constants).unwrap()).unwrap();
        let constants3 = serde_json::from_slice(&serde_json::to_vec(&constants).unwrap()).unwrap();

        let test_arity = 2;
        let preimage = vec![<Fr as Field>::ONE; test_arity];
        let mut h1 = Poseidon::<Fr, U2>::new_with_preimage(&preimage, &constants);
        let mut h2 = Poseidon::<Fr, U2>::new_with_preimage(&preimage, &constants2);
        let mut h3 = Poseidon::<Fr, U2>::new_with_preimage(&preimage, &constants3);

        assert_eq!(h1.hash(), h2.hash());
        h1.set_preimage(&preimage); // reset
        assert_eq!(h1.hash(), h3.hash());
    }

    #[test]
    fn serde_hash_pallas() {
        let constants = PoseidonConstants::<S1, U2>::new();
        let constants2 = bincode::deserialize(&bincode::serialize(&constants).unwrap()).unwrap();
        let constants3 = serde_json::from_slice(&serde_json::to_vec(&constants).unwrap()).unwrap();
        let test_arity = 2;
        let preimage = vec![<S1 as Field>::ONE; test_arity];
        let mut h1 = Poseidon::<S1, U2>::new_with_preimage(&preimage, &constants);
        let mut h2 = Poseidon::<S1, U2>::new_with_preimage(&preimage, &constants2);
        let mut h3 = Poseidon::<S1, U2>::new_with_preimage(&preimage, &constants3);

        assert_eq!(h1.hash(), h2.hash());
        h1.set_preimage(&preimage); // reset
        assert_eq!(h1.hash(), h3.hash());
    }
}
