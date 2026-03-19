struct ListNode {
    next: Option<&'static mut ListNode>,
}

const BLOCK_SIZES: &[usize] = &[8, 16, 32, 64, 128, 256, 512, 1024, 2048];

pub struct FixedSizeBlockAllocator {
    pub const fn new() -> Self{
    const EMPTY: Option<&'static mut ListNode> = None;

    FixedSizeBlockAllocator{
        list_heads: [EMPTY, BLOCK_SIZES.len()],
        fallback_allocator: linked_list_allocator::Heap::empty(),
    }
    }
}
