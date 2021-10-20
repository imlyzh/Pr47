use xjbutil::fat_ptr::FatPointer;

use crate::vm::al31f::alloc::Alloc;
use crate::vm::al31f::stack::Stack;

pub struct NoGCAlloc {
    managed: Vec<FatPointer>
}

impl NoGCAlloc {
    pub fn new() -> Self {
        Self {
            managed: vec![]
        }
    }
}

impl Drop for NoGCAlloc {
    fn drop(&mut self) {
        for _fat_ptr /*: &FatPointer*/ in self.managed.iter() {
            todo!()
        }
    }
}

impl Alloc for NoGCAlloc {
    #[inline(always)] unsafe fn add_stack(&mut self, _stack: *const Stack) {}

    #[inline(always)] unsafe fn remove_stack(&mut self, _stack: *const Stack) {}

    unsafe fn add_managed(&mut self, data: FatPointer) {
        self.managed.push(data);
    }

    #[inline(always)] unsafe fn mark_object(&mut self, _data: FatPointer) {}

    #[inline(always)] unsafe fn collect(&mut self) {}

    #[inline(always)] fn set_gc_allowed(&mut self, _allowed: bool) {}
}
