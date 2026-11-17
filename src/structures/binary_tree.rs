use std::ops::{Index, IndexMut};

use cuda_core::DeviceCopy;
use bytemuck::Zeroable;

#[derive(Copy, Clone, Zeroable)]
#[repr(C)]
/// A binary heap node.  This type includes a valid all-zero variant
/// (called "None") because the type must be zero-initializable for
/// when it is copied from host to device.
pub enum Node<I: DeviceCopy, L: DeviceCopy> {
    None,
    Internal(I),
    Leaf(L),
}

// SAFETY: The Zeroable trait on Node ensures that there exists a
// valid all-zero variant.  For Node the all-zero variant is the first
// variant.  We've called it "None".  Furthermore, Node also implements
// Copy and Clone, which are requirements of DeviceCopy.
unsafe impl<I: DeviceCopy, L: DeviceCopy> DeviceCopy for Node<I, L> {}


/// A binary heap with contiguously-stored data.  Used to transfer
/// data from host to device.  This type stores node data in a fixed-size
/// array (implements Copy and Clone) so that it can be transferred to the
/// device.
#[derive(Copy, Clone, DeviceCopy)]
pub struct BinaryHeap<I: DeviceCopy, L: DeviceCopy, const N: usize> {
    heap: [Node<I, L>; N],
}

impl<I: DeviceCopy, L: DeviceCopy, const N: usize> Index<usize> for BinaryHeap<I, L, N> {
    type Output = Node<I, L>;

    fn index(self: &Self, idx: usize) -> &Self::Output {
        &self.heap[idx]
    }
}

impl<I: DeviceCopy, L: DeviceCopy, const N: usize> IndexMut<usize> for BinaryHeap<I, L, N> {
    fn index_mut(self: &mut Self, idx: usize) -> &mut Self::Output {
        &mut self.heap[idx]
    }
}

impl<I: DeviceCopy, L: DeviceCopy, const N: usize> BinaryHeap<I, L, N> {
    /// Initializes an empty binary heap.
    pub fn new() -> Self {
        Self {
            heap: [Node::<I, L>::None; N],
        }
    }

    /// Returns the parent index of the given index if it has a parent.
    pub fn parent(self: &Self, idx: usize) -> Option<usize> {
        if 0 < idx && idx < N {
            Some((idx - 1) / 2)
        } else {
            None
        }
    }

    /// Returns the index of the left child of the given index if it exists.
    pub fn left(self: &Self, idx: usize) -> Option<usize> {
        let left_idx = 2 * idx + 1;

        if 0 <= idx && left_idx < N {
            Some(left_idx)
        } else {
            None
        }
    }

    /// Returns the index of the right child of the given index if it exists.
    pub fn right(self: &Self, idx: usize) -> Option<usize> {
        let right_idx = 2 * idx + 2;

        if 0 <= idx && right_idx < N {
            Some(right_idx)
        } else {
            None
        }
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