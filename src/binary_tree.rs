/// A binary heap stored as a fixed-size array.  Used to
/// transfer data from host to device.  This type does not
/// require that space be allocated on the heap.
struct BinaryHeap<P, const N: usize> {
    heap: [P; N],
}

/// A binary tree stored as a series of nodes allocated on
/// the heap.  Used to transfer data from host to device.
enum BinaryTree<P> {
    Internal {
        payload: P,
        left: Box<BinaryTree<P>>,
        right: Box<BinaryTree<P>>,
    },
    Leaf {
        payload: P,
    },
}