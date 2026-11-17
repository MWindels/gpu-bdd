use super::binary_tree::{BinaryHeap, Node};

const MAX_VARIABLE_OCCURRENCES: usize = 100;

const NEGATION_MASK: u8 = 0b10000000;
const IDENTIFIER_MASK: u8 = 0b01111111;

/// TODO
type Operator = u8;

const AND: Operator = 0;
const OR: Operator = 1;

/// TODO
type Operand = u8;

const FALSE: Operand = 0;
const TRUE: Operand = 1;
/// TODO
const IDENTIFIER: Operand = 2;

/// TODO
pub type PropositionalTree = BinaryHeap<Operator, Operand, MAX_VARIABLE_OCCURRENCES>;

impl PropositionalTree {
    /// TODO
    pub fn from_expr(expression: &str) -> Self {
        let mut heap = Self::new();

        heap.parse_expression(expression);

        heap
    }

    /// TODO
    fn parse_expression(self: &mut Self, expression: &str) {
        self[0] = Node::Internal(AND | NEGATION_MASK);
        || -> Option<()> {
            let (l, r) = (self.left(0)?, self.right(0)?);
            
            self[l] = Node::Internal(OR); {
                let (l, r) = (self.left(0)?, self.right(0)?);

                self[l] = Node::Leaf(IDENTIFIER + 0);
                self[r] = Node::Leaf(TRUE);
            }
            
            self[r] = Node::Leaf(FALSE);

            Some(())
        }().unwrap();
    }
}

/*
impl PropositionalTree {
    // This implementation uses a binary tree to represent the prop-tree.
    // The issue with this type is the use of `Box`es to store child nodes.
    // It's not possible to implement Copy for `Box`ed types.

    /// TODO
    pub fn new(p: Operator) -> Self {
        Self::Internal {
            payload: p,
            left: Box::new(Self::new_leaf(Operand::Variable { negate: false, id: 0 })),
            right: Box::new(Self::new_leaf(Operand::Literal { negate: false, value: true })),
        }
    }

    fn new_leaf(p: Operand) -> Self {
        Self::Leaf {
            payload: p,
        }
    }
}
*/