use std::collections::{HashSet, VecDeque};
use std::mem::transmute;

use crate::data::PTR_BITS_MASK_USIZE;
use crate::data::custom_vt::{CONTAINER_MASK, ContainerVT};
use crate::data::wrapper::{DynBase, OWN_INFO_COLLECT_MASK, Wrapper};
use crate::data::value_typed::VALUE_TYPE_MASK;
use crate::util::mem::FatPointer;
use crate::vm::al31f::alloc::Alloc;
use crate::vm::al31f::stack::Stack;

/// Default allocator for `AL31F`, with STW GC.
pub struct DefaultAlloc {
    stacks: HashSet<*const Stack<'static>>,
    managed: HashSet<FatPointer>,
    debt: usize,
    max_debt: usize,
    gc_allowed: bool
}

#[repr(u8)]
pub enum DefaultGCStatus {
    Unmarked = 0,
    Marked = 1
}

pub const DEFAULT_MAX_DEBT: usize = 512;

impl DefaultAlloc {
    pub fn new() -> Self {
        Self::with_max_debt(DEFAULT_MAX_DEBT)
    }

    pub fn with_max_debt(max_debt: usize) -> Self {
        Self {
            stacks: HashSet::new(),
            managed: HashSet::new(),
            debt: 0,
            max_debt,
            gc_allowed: false
        }
    }
}

impl Default for DefaultAlloc {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for DefaultAlloc {
    fn drop(&mut self) {
        for stack /*: *const Stack*/ in self.stacks.iter() {
            let stack: *mut Stack = *stack as *mut _;
            let boxed: Box<Stack> = unsafe { Box::from_raw(stack) };
            drop(boxed);
        }

        for ptr /*: &FatPointer*/ in self.managed.iter() {
            let raw_ptr: usize = (ptr.ptr & PTR_BITS_MASK_USIZE) as _;
            let wrapper: *mut Wrapper<()> = raw_ptr as _;

            if unsafe { !(*wrapper).ownership_info } & OWN_INFO_COLLECT_MASK != 0 {
                panic!("failed to re-claim object {:X} on destruction", raw_ptr);
            }

            if ptr.ptr & (CONTAINER_MASK as usize) != 0 {
                let container: *mut () = raw_ptr as *mut _;
                let vt: *const ContainerVT = ptr.trivia as *const _;
                unsafe { ((*vt).drop_fn)(container) };
            } else {
                let dyn_base: *mut dyn DynBase = unsafe { transmute::<>(*ptr) };
                let boxed: Box<dyn DynBase> = unsafe { Box::from_raw(dyn_base) };
                drop(boxed);
            }
        }
    }
}

impl Alloc for DefaultAlloc {
    unsafe fn add_stack(&mut self, stack: *const Stack<'_>) {
        self.stacks.insert(transmute::<>(stack));
    }

    unsafe fn remove_stack(&mut self, stack: *const Stack<'_>) {
        let removed: bool = self.stacks.remove(&transmute::<>(stack));
        debug_assert!(removed);
    }

    unsafe fn add_managed(&mut self, data: FatPointer) {
        if self.max_debt < self.debt && self.gc_allowed {
            self.collect();
        }
        self.managed.insert(data);
    }

    unsafe fn mark_object(&mut self, _data: FatPointer) {
        // do nothing
    }

    unsafe fn collect(&mut self) {
        for ptr /*: &FatPointer*/ in self.managed.iter() {
            debug_assert_eq!(ptr.ptr & (VALUE_TYPE_MASK as usize), 0);
            let wrapper: *mut Wrapper<()> = (ptr.ptr & PTR_BITS_MASK_USIZE) as *mut _;
            (*wrapper).gc_info = DefaultGCStatus::Unmarked as u8;
        }

        let mut to_scan: VecDeque<FatPointer> = VecDeque::new();

        for stack /*: &*const Stack*/ in self.stacks.iter() {
            #[cfg(debug_assertions)]
            for stack_value /*: &Option<Value>*/ in &(**stack).values {
                if let Some(stack_value /*: &Value*/) = stack_value {
                    if !stack_value.is_null() && !stack_value.is_value() {
                        to_scan.push_back(stack_value.ptr_repr);
                    }
                }
            }

            #[cfg(not(debug_assertions))]
            for stack_value /*: &Value*/ in &(**stack).values {
                if !stack_value.is_null() && !stack_value.is_value() {
                    to_scan.push_back(stack_value.ptr_repr);
                }
            }
        }

        while !to_scan.is_empty() {
            let ptr: FatPointer = to_scan.pop_front().unwrap();
            let wrapper: *mut Wrapper<()> = (ptr.ptr & PTR_BITS_MASK_USIZE) as *mut _;

            if (*wrapper).gc_info == (DefaultGCStatus::Marked as u8) {
                continue;
            }

            (*wrapper).gc_info = DefaultGCStatus::Marked as u8;
            if ptr.trivia & (CONTAINER_MASK as usize) != 0 {
                let dyn_base: *mut dyn DynBase = transmute::<>(ptr);
                if let Some(children /*: Box<dyn Iterator>*/) = (*dyn_base).children() {
                    for child /*: FatPointer*/ in children {
                        to_scan.push_back(child);
                    }
                }
            } else {
                let container_vt: *const ContainerVT = ptr.trivia as *const _;
                let ptr: *const () = (ptr.ptr & PTR_BITS_MASK_USIZE) as *const _;
                for child /*: FatPointer*/ in ((*container_vt).children_fn)(ptr) {
                    to_scan.push_back(child);
                }
            }
        }

        let mut to_collect: Vec<FatPointer> = Vec::new();
        for ptr /*: FatPointer*/ in self.managed.iter() {
            let wrapper: *mut Wrapper<()> = (ptr.ptr & PTR_BITS_MASK_USIZE) as *mut _;
            if (*wrapper).ownership_info & OWN_INFO_COLLECT_MASK != 0 {
                to_collect.push(*ptr);
            }
        }

        for ptr /*: FatPointer*/ in to_collect {
            if ptr.ptr & (CONTAINER_MASK as usize) != 0 {
                let container: *mut () = (ptr.ptr & PTR_BITS_MASK_USIZE) as *mut _;
                let vt: *const ContainerVT = ptr.trivia as *const _;
                ((*vt).drop_fn)(container);
            } else {
                let dyn_base: *mut dyn DynBase = transmute::<>(ptr);
                let boxed: Box<dyn DynBase> = Box::from_raw(dyn_base);
                drop(boxed);
            }
            self.managed.remove(&ptr);
        }
    }

    fn set_gc_allowed(&mut self, allowed: bool) {
        self.gc_allowed = allowed;
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_default_collector() {

    }
}
