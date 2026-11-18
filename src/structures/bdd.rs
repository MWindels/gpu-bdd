

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