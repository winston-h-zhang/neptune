use lazy_static::*;

use crate::{
    scalar_from_u64, Error, Scalar, ARITY, FULL_ROUNDS, MDS_MATRIX, PARTIAL_ROUNDS,
    ROUND_CONSTANTS, WIDTH,
};
use ff::Field;

lazy_static! {
    pub static ref ARITY_TAG: Scalar = arity_tag(ARITY);
}

/// The arity tag is the first element of a Poseidon permutation.
/// This extra element is necessary for 128-bit security.
pub fn arity_tag(arity: usize) -> Scalar {
    scalar_from_u64((1 << arity) - 1)
}

/// The `Poseidon` structure will accept a number of inputs equal to the arity.
///
/// The elements must implement [`ops::Mul`] against a [`Scalar`], because the MDS matrix and the
/// round constants are set, by default, as scalars.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Poseidon {
    constants_offset: usize,
    /// the elements to permute
    pub elements: [Scalar; WIDTH],
    pos: usize,
}

impl Default for Poseidon {
    fn default() -> Self {
        let mut elements = [Scalar::zero(); WIDTH];
        elements[0] = *ARITY_TAG;
        Poseidon {
            constants_offset: 0,
            elements,
            pos: 1,
        }
    }
}

impl Poseidon {
    /// Create a new Poseidon hasher for `preimage`.
    pub fn new(preimage: [Scalar; ARITY]) -> Self {
        let mut p = Poseidon::default();

        p.set_preimage(preimage);
        p
    }

    /// Replace the elements with the provided optional items.
    ///
    /// # Panics
    ///
    /// Panics if the provided slice is bigger than the arity.
    pub fn set_preimage(&mut self, preimage: [Scalar; ARITY]) {
        self.reset();
        self.elements[1..].copy_from_slice(&preimage);
    }

    /// Restore the initial state
    pub fn reset(&mut self) {
        self.constants_offset = 0;
        self.elements[1..]
            .iter_mut()
            .for_each(|l| *l = scalar_from_u64(0u64));
        self.elements[0] = *ARITY_TAG;
        self.pos = 1;
    }

    /// The returned `usize` represents the element position for the insert operation
    pub fn input(&mut self, element: Scalar) -> Result<usize, Error> {
        // Cannot input more elements than the defined arity
        if self.pos >= WIDTH {
            return Err(Error::FullBuffer);
        }

        // Set current element, and increase the pointer
        self.elements[self.pos] = element;
        self.pos += 1;

        Ok(self.pos - 1)
    }

    /// The number of rounds is divided into two equal parts for the full rounds, plus the partial rounds.
    ///
    /// The returned element is the second poseidon element, the first is the arity tag.
    pub fn hash(&mut self) -> Scalar {
        // This counter is incremented when a round constants is read. Therefore, the round constants never
        // repeat
        for _ in 0..FULL_ROUNDS / 2 {
            self.full_round();
        }

        for _ in 0..PARTIAL_ROUNDS {
            self.partial_round();
        }

        for _ in 0..FULL_ROUNDS / 2 {
            self.full_round();
        }

        self.elements[1]
    }

    /// The full round function will add the round constants and apply the S-Box to all poseidon elements, including the bitflags first element.
    ///
    /// After that, the poseidon elements will be set to the result of the product between the poseidon elements and the constant MDS matrix.
    pub fn full_round(&mut self) {
        // Every element of the hash buffer is incremented by the round constants
        self.add_round_constants();

        // Apply the quintic S-Box to all elements
        self.elements.iter_mut().for_each(|l| quintic_s_box(l));

        // Multiply the elements by the constant MDS matrix
        self.product_mds();
    }

    /// The partial round is the same as the full round, with the difference that we apply the S-Box only to the first bitflags poseidon leaf.
    pub fn partial_round(&mut self) {
        // Every element of the hash buffer is incremented by the round constants
        self.add_round_constants();

        // Apply the quintic S-Box to the first element
        quintic_s_box(&mut self.elements[0]);

        // Multiply the elements by the constant MDS matrix
        self.product_mds();
    }

    /// For every leaf, add the round constants with index defined by the constants offset, and increment the
    /// offset
    fn add_round_constants(&mut self) {
        let mut constants_offset = self.constants_offset;

        self.elements.iter_mut().for_each(|l| {
            l.add_assign(&ROUND_CONSTANTS[constants_offset]);
            constants_offset += 1;
        });

        self.constants_offset = constants_offset;
    }

    /// Set the provided elements with the result of the product between the elements and the constant
    /// MDS matrix
    fn product_mds(&mut self) {
        let mut result = [scalar_from_u64(0u64); WIDTH];

        for j in 0..WIDTH {
            for k in 0..WIDTH {
                let mut tmp = MDS_MATRIX[j][k];
                tmp.mul_assign(&self.elements[k]);
                result[j].add_assign(&tmp);
            }
        }

        self.elements.copy_from_slice(&result);
    }
}

/// Apply the quintic S-Box (s^5) to a given item
fn quintic_s_box(l: &mut Scalar) {
    let c = *l;
    for _ in 0..4 {
        l.mul_assign(&c);
    }
}

#[cfg(test)]
mod tests {
    use crate::*;
    use ff::Field;

    #[test]
    fn reset() {
        let preimage: [Scalar; ARITY] = [Scalar::one(); ARITY];
        let mut h = Poseidon::new(preimage);
        h.hash();
        h.reset();

        assert_eq!(Poseidon::default(), h);
    }

    #[test]
    fn hash_det() {
        let mut preimage: [Scalar; ARITY] = [Scalar::zero(); ARITY];
        preimage[0] = Scalar::one();

        let mut h = Poseidon::new(preimage);

        let mut h2 = h.clone();
        let result = h.hash();

        assert_eq!(result, h2.hash());
    }

    #[test]
    /// Simple test vectors to ensure results don't change unintentionally in development.
    fn hash_values() {
        let mut p = Poseidon::default();
        let preimage = for n in 0..ARITY {
            p.input(scalar_from_u64(n as u64)).unwrap();
        };
        for i in 0..10 {
            dbg!(ROUND_CONSTANTS[i]);
        }
        let digest = p.hash();
        let expected = match ARITY {
            2 => scalar_from_u64s([
                0xa40b50fbd526e3d8,
                0x602e561dec0276f1,
                0x876c16938745414a,
                0x586402df1f7245a6,
            ]),
            4 => scalar_from_u64s([
                0x7d24050357d9da75,
                0x0a983161fe8bcc6f,
                0xacec00e9669c838c,
                0x5563d0a74e95a99f,
            ]),
            8 => scalar_from_u64s([
                0x3a261c43c7d513f1,
                0xd90573f76693c368,
                0x94f197e804c99da8,
                0x02ff178318692402,
            ]),
            _ => {
                dbg!(digest);
                panic!("Arity lacks test vector: {}", ARITY)
            }
        };
        dbg!(ARITY);
        assert_eq!(expected, digest);
    }
}
