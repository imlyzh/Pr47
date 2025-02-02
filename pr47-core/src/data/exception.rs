use std::any::TypeId;
use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;
use std::ptr::NonNull;

use tokio::task::JoinError;
use xjbutil::void::Void;

use crate::data::Value;
use crate::data::generic::GenericTypeVT;
use crate::data::traits::{ChildrenType, StaticBase};
use crate::data::tyck::{TyckInfo, TyckInfoPool, ContainerTyckInfo};

pub enum UncheckedException {
    AlreadyAwaited { promise: Value },
    ArgCountMismatch { func_id: usize, expected: usize, got: usize },
    DivideByZero,
    InvalidBinaryOp { bin_op: char, lhs: Value, rhs: Value },
    InvalidCastOp { dest_type: &'static str, src: Value },
    InvalidUnaryOp { unary_op: char, src: Value },
    OwnershipCheckFailure { object: Value, expected_mask: u8 },
    TypeCheckFailure { object: Value, expected_type: NonNull<TyckInfo> },
    OverloadCallFailure { overload_table: usize },
    UnexpectedNull { value: Value },
    IndexOutOfBounds { indexed: Value, index: i64 },
    #[cfg(feature = "async")]
    JoinError { inner: JoinError }
}

pub type CheckedException = Value;

pub enum ExceptionInner {
    Unchecked(UncheckedException),
    Checked(CheckedException)
}

impl Debug for ExceptionInner {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // TODO should we provide prettier printing?
        match self {
            ExceptionInner::Unchecked(_) => write!(f, "ExceptionInner::Unchecked"),
            ExceptionInner::Checked(_) => write!(f, "ExceptionInner::Checked")
        }
    }
}

#[derive(Clone, Copy)]
pub struct StackTrace {
    pub func_id: usize,
    pub insc_ptr: usize
}

impl StackTrace {
    pub fn new(func_id: usize, insc_ptr: usize) -> Self {
        Self { func_id, insc_ptr }
    }
}

pub struct Exception {
    pub inner: ExceptionInner,
    trace: Vec<StackTrace>
}

impl Exception {
    #[inline(never)] pub fn checked_exc(checked: CheckedException) -> Self {
        Self {
            inner: ExceptionInner::Checked(checked),
            trace: vec![]
        }
    }

    #[inline(never)] pub fn unchecked_exc(unchecked: UncheckedException) -> Self {
        Self {
            inner: ExceptionInner::Unchecked(unchecked),
            trace: vec![]
        }
    }

    pub fn push_stack_trace(&mut self, func_id: usize, insc_ptr: usize) {
        self.trace.push(StackTrace::new(func_id, insc_ptr))
    }

    #[cfg(test)]
    pub fn assert_checked(&self) -> CheckedException {
        match &self.inner {
            ExceptionInner::Checked(e) => e.clone(),
            ExceptionInner::Unchecked(_) => panic!()
        }
    }
}

impl StaticBase<Exception> for Void {
    fn type_name() -> String { "Exception".into() }

    fn children(vself: *const Exception) -> ChildrenType {
        match unsafe { &(*vself).inner } {
            ExceptionInner::Unchecked(_) => None,
            ExceptionInner::Checked(checked) => {
                Some(Box::new(std::iter::once(*checked)))
            }
        }
    }
}

#[repr(transparent)]
pub struct ExceptionContainer<E> {
    pub exception: Exception,
    _phantom: PhantomData<E>
}

impl<E> StaticBase<ExceptionContainer<E>> for Void
    where E: 'static,
          Void: StaticBase<E>
{
    fn type_id() -> TypeId {
        <Void as StaticBase<Exception>>::type_id()
    }

    fn tyck_info(tyck_info_pool: &mut TyckInfoPool) -> NonNull<TyckInfo> {
        let inner_tyck_info: NonNull<TyckInfo> =
            <Void as StaticBase<E>>::tyck_info(tyck_info_pool);
        tyck_info_pool.create_container_type(Self::type_id(), &[inner_tyck_info])
    }

    fn tyck(tyck_info: &TyckInfo) -> bool {
        if let TyckInfo::Container(ContainerTyckInfo { type_id, params }) = tyck_info {
            if Self::type_id() != *type_id {
                return false;
            }
            if unsafe { params.as_ref().len() } != 1 {
                return false;
            }
            let param0: NonNull<TyckInfo> = unsafe { *params.as_ref().get_unchecked(0) };
            <Void as StaticBase<E>>::tyck(unsafe { param0.as_ref() })
        } else {
            false
        }
    }

    fn type_name() -> String {
        format!("ExcContainer<{}>", <Void as StaticBase<E>>::type_name())
    }

    fn children(vself: *const ExceptionContainer<E>) -> ChildrenType {
        let r: &ExceptionContainer<E> = unsafe { &*vself };
        match r.exception.inner {
            ExceptionInner::Unchecked(_) => None,
            ExceptionInner::Checked(checked) => {
                Some(Box::new(std::iter::once(checked)))
            }
        }
    }
}

pub fn create_exception_vt(
    tyck_info_pool: &mut TyckInfoPool,
    elem_type_name: &str,
    elem_tyck_info: NonNull<TyckInfo>
) -> GenericTypeVT {
    use crate::data::generic::gen_impls;

    #[cfg(debug_assertions)]
    unsafe fn move_out_ck(this: *mut (), out: *mut (), type_id: TypeId) {
        gen_impls::generic_move_out_ck::<Exception>(this, out, type_id)
    }

    #[cfg(not(debug_assertions))]
    unsafe fn move_out(this: *mut (), out: *mut ()) {
        gen_impls::generic_move_out::<Exception>(this, out)
    }

    unsafe fn children(this: *const ()) -> ChildrenType {
        gen_impls::generic_children::<Exception>(this)
    }

    unsafe fn exception_drop(this: *mut ()) {
        gen_impls::generic_drop::<Exception>(this)
    }

    let tyck_info: NonNull<TyckInfo> = tyck_info_pool.create_container_type(
        <Void as StaticBase<Exception>>::type_id(),
        &[elem_tyck_info]
    );
    let container_tyck_info: NonNull<ContainerTyckInfo> =
        unsafe { tyck_info.as_ref().get_container_tyck_info_unchecked() };

    GenericTypeVT {
        tyck_info: container_tyck_info,
        type_name: format!("ExcContainer<{}>", elem_type_name),
        #[cfg(debug_assertions)]
        move_out_fn: move_out_ck,
        #[cfg(not(debug_assertions))]
        move_out_fn: move_out,
        children_fn: children,
        drop_fn: exception_drop
    }
}
