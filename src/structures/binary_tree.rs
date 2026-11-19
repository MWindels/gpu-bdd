use std::ops::{Index, IndexMut};
use std::mem::size_of;

use cuda_core::DeviceCopy;
use bytemuck::Zeroable;

/// The basis of all binary heaps.  Includes some common functionality and
/// declarations of a number of functions which must be implemented by types
/// implementing this trait.  Does not handle access or mutation.
pub trait BinaryHeap<P: Zeroable + DeviceCopy, const N: usize>: Copy + Clone + DeviceCopy {
    /// Initializes an empty binary heap.
    fn new() -> Self;

    /// Returns the capacity of the heap.
    fn cap() -> usize;

    /// Returns the parent index of the given index if it has a parent.
    fn parent(self: &Self, idx: usize) -> Option<usize> {
        if 0 < idx && idx < Self::cap() {
            Some((idx - 1) / 2)
        } else {
            None
        }
    }

    /// Returns the index of the left child of the given index if it exists.
    fn left(self: &Self, idx: usize) -> Option<usize> {
        let left_idx = 2 * idx + 1;

        if left_idx < Self::cap() {
            Some(left_idx)
        } else {
            None
        }
    }

    /// Returns the index of the right child of the given index if it exists.
    fn right(self: &Self, idx: usize) -> Option<usize> {
        let right_idx = 2 * idx + 2;

        if right_idx < Self::cap() {
            Some(right_idx)
        } else {
            None
        }
    }
}

/// A binary heap with contiguously-stored data.  Used to transfer
/// data from host to device.  This type stores node data in a fixed-size
/// array where each element is its own node in the heap.
#[derive(Copy, Clone, DeviceCopy)]
pub struct UnitBinaryHeap<P: Zeroable + DeviceCopy, const N: usize> {
    heap: [P; N],
}

impl<P: Zeroable + DeviceCopy, const N: usize> BinaryHeap<P, N> for UnitBinaryHeap<P, N> {
    fn new() -> Self {
        Self {
            heap: [P::zeroed(); N],
        }
    }

    fn cap() -> usize {
        N
    }
}

impl<P: Zeroable + DeviceCopy, const N: usize> Index<usize> for UnitBinaryHeap<P, N> {
    type Output = P;

    fn index(self: &Self, idx: usize) -> &Self::Output {
        &self.heap[idx]
    }
}

impl<P: Zeroable + DeviceCopy, const N: usize> IndexMut<usize> for UnitBinaryHeap<P, N> {
    fn index_mut(self: &mut Self, idx: usize) -> &mut Self::Output {
        &mut self.heap[idx]
    }
}

/// The type used in tightly-packed binary heaps.
pub type PackedElement = u32;

pub const PACKED_ELEMENT_BITS: usize = 8 * size_of::<PackedElement>();

/// A binary heap implementation which stores small data by packing them into
/// an array of bitfields.  Can be passed from host to device if necessary.
/// For safety's sake, access and mutation must be done by value using the
/// `get(...)` and `set(...)` functions.
#[derive(Copy, Clone, DeviceCopy)]
pub struct PackedBinaryHeap<const N: usize, const BITS: usize> {
    pub(super) heap: [PackedElement; N],
}

impl<const N: usize, const BITS: usize> BinaryHeap<PackedElement, N> for PackedBinaryHeap<N, BITS> {
    fn new() -> Self {
        Self {
            heap: [0; N],
        }
    }

    fn cap() -> usize {
        const{Self::nodes_per_element() * N}
    }
}

impl<const N: usize, const BITS: usize> PackedBinaryHeap<N, BITS> {
    const _ASSERT: () = {
        debug_assert!(
            size_of::<PackedElement>() % BITS == 0,
            "the number of bits per node does not evenly divide the size of the heap's unit element"
        );
    };

    /// Returns the number of packed nodes in each heap element.
    pub(super) const fn nodes_per_element() -> usize {
        PACKED_ELEMENT_BITS / BITS
    }

    /// Returns a triplet containing (1) the index of the bitfield that contains the value
    /// at `idx`, (2) the mask to apply to extract the packed value, and (3) the offset to
    /// use to recover the packed value after masking the bitfield.
    fn access(self: &Self, idx: usize) -> (usize, PackedElement, usize) {
        (
            idx / Self::nodes_per_element(),
            ((1 as PackedElement) << BITS) - 1,
            idx % Self::nodes_per_element(),
        )
    }

    /// Unpacks the value at `idx` and returns it by value.
    pub fn get(self: &Self, idx: usize) -> PackedElement {
        let (packed_idx, mask, offset) = self.access(idx);
        (self.heap[packed_idx] & (mask << offset)) >> offset
    }

    /// Packs and sets the value at `idx` by value.
    pub fn set(self: &mut Self, idx: usize, value: PackedElement) {
        let (packed_idx, mask, offset) = self.access(idx);
        self.heap[packed_idx] = (self.heap[packed_idx] & !(mask << offset)) ^ ((value & mask) << offset);
    }
}

/*
/// A binary tree stored as a series of nodes.  Used to
/// transfer data from host to device.  This type may
/// need to be built in HMM in order for the device to
/// see it due to the `Box`es that encolse the child nodes.
pub(super) enum BinaryTree<I, L> {
    Internal {
        payload: I,
        left: Box<BinaryTree<I, L>>,
        right: Box<BinaryTree<I, L>>,
    },
    Leaf {
        payload: L,
    },
}

impl<I: Clone, L: Clone> Clone for BinaryTree<I, L> {
    fn clone(self: &Self) -> Self {
        match self {
            Self::Internal{payload, left, right} => Self::Internal {
                payload: payload.clone(),
                left: Box::new(*left.clone()),
                right: Box::new(*right.clone()),
            },
            Self::Leaf{payload} => Self::Leaf {
                payload: payload.clone(),
            },
        }
    }
}
*/