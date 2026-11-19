use std::mem::size_of;

use super::binary_tree::{PackedElement, PACKED_ELEMENT_BITS, PackedBinaryHeap};
use super::expression::MAX_PROP_TREE_SIZE;

const COFACTOR_NODE_BITS: usize = 2;
const COFACTOR_ELEMENTS: usize = MAX_PROP_TREE_SIZE.div_ceil(PACKED_ELEMENT_BITS / COFACTOR_NODE_BITS);

type CofactorArray = [PackedElement; COFACTOR_ELEMENTS];
type Cofactor = PackedBinaryHeap<COFACTOR_ELEMENTS, COFACTOR_NODE_BITS>;

/// Generates a `CofactorArray` initialized with a given default value.
macro_rules! cofactor_array {
    ($value:expr) => {
        [($value) as PackedElement; COFACTOR_ELEMENTS]
    };
}

/// Computes the result of repeating a one-byte pattern across a packed element.
const fn repeating_byte(pattern: u8) -> PackedElement {
    let mut unit: PackedElement = 0;

    let mut byte = 0;
    while byte < size_of::<PackedElement>() {
        unit = (unit << 8) | (pattern as PackedElement);
        byte += 1;
    }

    unit
}

/// Applies a unary operation to every element in array `a`.
/// The result overwrites `a`.
fn array_unary_op(op: impl Fn(PackedElement) -> PackedElement, a: &mut CofactorArray) -> &mut CofactorArray {
    for i in 0..COFACTOR_ELEMENTS {
        a[i] = op(a[i]);
    }

    a
}

/// Applies a binary operation to every pair of elements in arrays
/// `a` and `b`.  The result overwrites `a`.
fn array_binary_op(op: impl Fn(PackedElement, PackedElement) -> PackedElement, a: &mut CofactorArray, b: &CofactorArray) {
    for i in 0..COFACTOR_ELEMENTS {
        a[i] = op(a[i], b[i]);
    }
}

/// Writes the elements of `b` into `a`.
fn array_overwrite(a: &mut CofactorArray, b: &CofactorArray) {
    array_binary_op(|_, b_i| -> PackedElement {b_i}, a, b);
}

/// Applies in-place bitwise AND to every pair of elements in `a`
/// and `b` and returns the result by overwriting `a`.
fn array_and(a: &mut CofactorArray, b: &CofactorArray) {
    array_binary_op(|a_i, b_i| -> PackedElement {a_i & b_i}, a, b);
}

/// Applies in-place bitwise AND to every element in `a` with the
/// element `b` and returns the result by overwriting `a`.
fn array_and_repeating(a: &mut CofactorArray, b: PackedElement) {
    array_unary_op(|a_i| -> PackedElement {a_i & b}, a);
}

/// Applies in-place bitwise OR to every pair of elements in `a`
/// and `b` and returns the result by overwriting `a`.
fn array_or(a: &mut CofactorArray, b: &CofactorArray) {
    array_binary_op(|a_i, b_i| -> PackedElement {a_i | b_i}, a, b);
}

/// Applies in-place bitwise OR to every element in `a` with the
/// element `b` and returns the result by overwriting `a`.
fn array_or_repeating(a: &mut CofactorArray, b: PackedElement) {
    array_unary_op(|a_i| -> PackedElement {a_i | b}, a);
}

/*
/// Applies in-place bitwise XOR to every pair of elements in `a`
/// and `b` and returns the result by overwriting `a`.
fn array_xor(a: &mut CofactorArray, b: &CofactorArray) {
    array_binary_op(|a_i, b_i| -> PackedElement {a_i ^ b_i}, a, b);
}

/// Applies in-place bitwise XOR to every element in `a` with the
/// element `b` and returns the result by overwriting `a`.
fn array_xor_repeating(a: &mut CofactorArray, b: PackedElement) {
    array_unary_op(|a_i| -> PackedElement {a_i ^ b}, a);
}
*/

/// Returns constants relevant to bit shifting a whole array.
const fn array_shift_constants(offset: usize) -> (usize, usize, usize) {
    let units_to_shift = offset / PACKED_ELEMENT_BITS;
    let bits_to_shift = offset % PACKED_ELEMENT_BITS;

    (units_to_shift, bits_to_shift, COFACTOR_ELEMENTS - units_to_shift)
}

/// Performs a right shift on the elements of an array, treating them
/// as a single contiguous value.  The result is returned in a new array.
fn array_down_shift(a: &CofactorArray, offset: usize) -> CofactorArray {
    let mut result: CofactorArray = cofactor_array!(0);

    let (units_to_shift, bits_to_shift, remaining_units) = array_shift_constants(offset);
    let remaining_bit_mask: PackedElement = (1 << bits_to_shift) - 1;

    if remaining_units > 0 {
        result[0] = a[units_to_shift] >> bits_to_shift;

        for i in 1..remaining_units {
            result[i - 1] |= (a[units_to_shift + i] & remaining_bit_mask) << (PACKED_ELEMENT_BITS - bits_to_shift);
            result[i] = a[units_to_shift + i] >> bits_to_shift;
        }
    }

    result
}

/// Performs a left shift on the elements of an array, treating them
/// as a single contiguous value.  The result is returned in a new array.
fn array_up_shift(a: &CofactorArray, offset: usize) -> CofactorArray {
    let mut result: CofactorArray = cofactor_array!(0);

    let (units_to_shift, bits_to_shift, remaining_units) = array_shift_constants(offset);
    let remaining_bit_mask: PackedElement = !(!0 >> bits_to_shift);

    if remaining_units > 0 {
        result[COFACTOR_ELEMENTS - 1] = a[remaining_units - 1] << bits_to_shift;

        for i in (units_to_shift..(COFACTOR_ELEMENTS - 1)).rev() {
            result[i + 1] |= (a[i - units_to_shift] & remaining_bit_mask) >> (PACKED_ELEMENT_BITS - bits_to_shift);
            result[i] = a[i - units_to_shift] << bits_to_shift;
        }
    }

    result
}

impl Cofactor {
    // TODO: prop-tree ascent
    // TODO: branch ascent

    /// Computes a mask which covers cofactor nodes which are non-null.
    pub fn existential_mask(self: &Self) -> CofactorArray {
        // Need two copies of the cofactor heap.
        let mut high_bits = self.heap.clone();
        let mut low_bits = self.heap.clone();

        // Extract the high bits of each cofactor node.
        array_and_repeating(&mut high_bits, repeating_byte(0xAA));
        array_down_shift(&mut high_bits, 1);

        // Extract the low bits of each cofactor node.
        array_and_repeating(&mut low_bits, repeating_byte(0x55));

        // Evaluate where either high or low bits are true and make a copy.
        array_or(&mut high_bits, &low_bits);
        array_overwrite(&mut low_bits, &high_bits);

        // Shift one copy up and join them together.
        array_up_shift(&mut high_bits, 1);
        array_or(&mut high_bits, &low_bits);

        // Since we wrote to high_bits, return it as the result.
        high_bits
    }

    /// Computes a mask which covers cofactor nodes which represent a literal.
    pub fn literal_mask(self: &Self) -> CofactorArray {
        // Need one copy of the cofactor heap and a location for the upcoming high bits.
        let mut low_bits = self.heap.clone();
        let mut copy_bits: CofactorArray = cofactor_array!(0);

        // Extract the low bits of each cofactor node.
        array_and_repeating(&mut low_bits, repeating_byte(0x55));

        // Copy the extracted low bits and move them to the high position.
        array_overwrite(&mut copy_bits, &low_bits);
        array_up_shift(&copy_bits, 1);

        // Join the original low bits with the up-shifted copy.
        array_or(&mut low_bits, &copy_bits);

        // Return low_bits since it was last written to.
        low_bits
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repeating_byte() {
        assert_eq!(0, repeating_byte(0x00));
        assert_eq!(!0, repeating_byte(0xFF));
        assert_eq!(repeating_byte(0x55) & repeating_byte(0xAA), 0);
        assert_eq!(!(repeating_byte(0x55) | repeating_byte(0xAA)), 0);
    }

    #[test]
    fn test_array_binary_ops() {
        let mut result: CofactorArray = cofactor_array!(0);

        let a: CofactorArray = cofactor_array!(repeating_byte(0x55));
        let b: CofactorArray = cofactor_array!(repeating_byte(0xAA));
        
        array_overwrite(&mut result, &a);
        assert_eq!(result, a, "expected array_overwrite to overwrite result");

        array_or(&mut result, &b);
        assert_eq!(result, cofactor_array!(repeating_byte(0xFF)), "expected array_or to produce all 1s");

        array_and(&mut result, &b);
        assert_eq!(result, b, "expected array_and to produce b");

        array_and_repeating(&mut result, repeating_byte(0x55));
        assert_eq!(result, cofactor_array!(0), "expected array_and_repeating to produce all 0s");

        array_or_repeating(&mut result, repeating_byte(0x55));
        assert_eq!(result, a, "expected array_or_repeating to produce a");
    }

    #[test]
    fn test_array_up_shift() {
        let mut result = cofactor_array!(repeating_byte(0xFF));
        const LAST_ELEMENT: usize = COFACTOR_ELEMENTS - 1;

        // Test single-bit upshift.
        array_up_shift(&mut result, 1);
        assert_eq!(result[0], repeating_byte(0xFF) << 1);
        assert_eq!(result[1..], cofactor_array!(repeating_byte(0xFF))[1..]);
        result[0] = repeating_byte(0xFF);

        // Test single-element upshift.
        array_up_shift(&mut result, PACKED_ELEMENT_BITS);
        assert_eq!(result[0], 0);
        assert_eq!(result[1..], cofactor_array!(repeating_byte(0xFF))[1..]);
        result[0] = repeating_byte(0xFF);

        // Test multi-element upshift.
        array_up_shift(&mut result, PACKED_ELEMENT_BITS * LAST_ELEMENT);
        assert_eq!(result[LAST_ELEMENT], repeating_byte(0xFF));
        assert_eq!(result[..LAST_ELEMENT], cofactor_array!(0)[..LAST_ELEMENT]);
        result = cofactor_array!(repeating_byte(0xFF));

        // Test multi-element and multi-bit upshift.
        array_up_shift(&mut result, PACKED_ELEMENT_BITS * COFACTOR_ELEMENTS - 1);
        assert_eq!(result[LAST_ELEMENT], 1 << (PACKED_ELEMENT_BITS - 1));
        assert_eq!(result[..LAST_ELEMENT], cofactor_array!(0)[..LAST_ELEMENT]);
    }

    #[test]
    fn test_array_down_shift() {
        let mut result = cofactor_array!(repeating_byte(0xFF));
        const LAST_ELEMENT: usize = COFACTOR_ELEMENTS - 1;

        // Test single-bit downshift.
        array_down_shift(&mut result, 1);
        assert_eq!(result[LAST_ELEMENT], repeating_byte(0xFF) >> 1);
        assert_eq!(result[..LAST_ELEMENT], cofactor_array!(repeating_byte(0xFF))[..LAST_ELEMENT]);
        result[LAST_ELEMENT] = repeating_byte(0xFF);

        // Test single-element downshift.
        array_down_shift(&mut result, PACKED_ELEMENT_BITS);
        assert_eq!(result[LAST_ELEMENT], 0);
        assert_eq!(result[..LAST_ELEMENT], cofactor_array!(repeating_byte(0xFF))[..LAST_ELEMENT]);
        result[LAST_ELEMENT] = repeating_byte(0xFF);

        // Test multi-element downshift.
        array_down_shift(&mut result, PACKED_ELEMENT_BITS * LAST_ELEMENT);
        assert_eq!(result[0], repeating_byte(0xFF));
        assert_eq!(result[1..], cofactor_array!(0)[1..]);
        result = cofactor_array!(repeating_byte(0xFF));

        // Test multi-element and multi-bit downshift.
        array_down_shift(&mut result, PACKED_ELEMENT_BITS * COFACTOR_ELEMENTS - 1);
        assert_eq!(result[0], 1);
        assert_eq!(result[1..], cofactor_array!(0)[1..]);
    }
}