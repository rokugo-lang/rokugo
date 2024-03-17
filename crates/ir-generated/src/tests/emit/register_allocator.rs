use crate::emit::register_allocator::RegisterAllocator;

#[test]
fn alloc_overflow() {
    let mut allocator = RegisterAllocator::new();
    for _ in 0..u16::MAX {
        if allocator.next_nat32().is_err() {
            return;
        }
    }

    panic!("Should overflow");
}
