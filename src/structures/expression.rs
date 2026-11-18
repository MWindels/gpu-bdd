use std::mem::size_of;

use cuda_core::DeviceCopy;

use super::binary_tree::{BinaryHeap, UnitBinaryHeap, PackedNode, PackedBinaryHeap};

type PropTreeNode = u8;

const PROP_TREE_NODE_BITS: usize = 8 * size_of::<PropTreeNode>();

const NEGATE: PropTreeNode = 1 << (PROP_TREE_NODE_BITS - 1);
const PAYLOAD: PropTreeNode = !0 >> 1;

// The default all-zero prop-tree node.
const NULL: PropTreeNode = 0;

// Prop-tree operator indices.
const AND: PropTreeNode = 1;
const OR: PropTreeNode = 2;

// Prop-tree literal indices.
const FALSE: PropTreeNode = 3;
const TRUE: PropTreeNode = 4;

/// Prop-tree variables start at this index.
const VARIABLE: PropTreeNode = 5;

const MAX_VARIABLE_OCCURRENCES: usize = (PAYLOAD - VARIABLE) as usize;
const MAX_PROP_TREE_SIZE: usize = 2 * MAX_VARIABLE_OCCURRENCES;

/// TODO
pub type PropositionalTree = UnitBinaryHeap<PropTreeNode, MAX_PROP_TREE_SIZE>;

impl PropositionalTree {
    /// TODO
    pub fn from_expr(expression: &str) -> Self {
        let mut heap = Self::new();

        heap.parse_expression(expression);

        heap
    }

    /// TODO
    fn parse_expression(self: &mut Self, expression: &str) {
        self[0] = NEGATE | AND;
        || -> Option<()> {
            let (l, r) = (self.left(0)?, self.right(0)?);
            
            self[l] = OR; {
                let (l, r) = (self.left(l)?, self.right(l)?);

                self[l] = VARIABLE + 0;
                self[r] = TRUE;
            }
            
            self[r] = FALSE;

            Some(())
        }().unwrap();
    }
}

const UNIT_COFACTOR_BYTES: usize = size_of::<PackedNode>();

const COFACTOR_NODE_BITS: usize = 2;
const COFACTOR_NODES_PER_BYTE: usize = 8 / COFACTOR_NODE_BITS;
const COFACTOR_UNITS: usize = MAX_PROP_TREE_SIZE.div_ceil(COFACTOR_NODES_PER_BYTE * UNIT_COFACTOR_BYTES);

type Cofactor = PackedBinaryHeap<COFACTOR_UNITS, COFACTOR_NODE_BITS>;

impl Cofactor {
    // TODO: prop-tree ascent
    // TODO: branch ascent
}