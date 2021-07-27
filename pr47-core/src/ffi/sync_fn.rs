use crate::data::Value;
use crate::data::exception::Exception;
use crate::ffi::Signature;
use crate::util::mem::FatPointer;

pub trait VMContext: 'static + Sized {
    fn allocate(&mut self, fat_ptr: FatPointer);
    fn mark(&mut self, fat_ptr: FatPointer);
}

pub trait FunctionBase: 'static {
    fn signature() -> Signature;

    fn call_tyck<CTX: VMContext>(
        context: &mut CTX,
        args: &[Value],
        rets: &mut [&mut Value]
    ) -> Option<Exception>;

    unsafe fn call_rtlc<CTX: VMContext>(
        context: &mut CTX,
        args: &[Value],
        rets: &mut [&mut Value]
    ) -> Option<Exception>;

    unsafe fn call_unchecked<CTX: VMContext>(
        context: &mut CTX,
        args: &[Value],
        rets: &mut [&mut Value]
    ) -> Option<Exception>;
}

pub trait Function<CTX: VMContext>: 'static {
    fn signature(&self) -> Signature;

    fn call_tyck(
        &self,
        context: &mut CTX,
        args: &[Value],
        rets: &mut [&mut Value]
    ) -> Option<Exception>;

    unsafe fn call_rtlc(
        &self,
        context: &mut CTX,
        args: &[Value],
        rets: &mut [&mut Value]
    ) -> Option<Exception>;

    unsafe fn call_unchecked(
        &self,
        context: &mut CTX,
        args: &[Value],
        rets: &mut [&mut Value]
    ) -> Option<Exception>;
}

impl<FBase, CTX> Function<CTX> for FBase where
    FBase: FunctionBase,
    CTX: VMContext
{
    #[inline] fn signature(&self) -> Signature {
        <FBase as FunctionBase>::signature()
    }

    #[inline] fn call_tyck(
        &self,
        context: &mut CTX,
        args: &[Value],
        rets: &mut [&mut Value]
    ) -> Option<Exception> {
        <FBase as FunctionBase>::call_tyck(context, args, rets)
    }

    #[inline] unsafe fn call_rtlc(
        &self,
        context: &mut CTX,
        args: &[Value],
        rets: &mut [&mut Value]
    ) -> Option<Exception> {
        <FBase as FunctionBase>::call_rtlc(context, args, rets)
    }

    #[inline] unsafe fn call_unchecked(
        &self,
        context: &mut CTX,
        args: &[Value],
        rets: &mut [&mut Value]
    ) -> Option<Exception> {
        <FBase as FunctionBase>::call_unchecked(context, args, rets)
    }
}
