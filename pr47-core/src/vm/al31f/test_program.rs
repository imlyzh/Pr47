use crate::boxed_slice;
use crate::data::Value;
use crate::data::traits::StaticBase;
use crate::ds::object::Object;
use crate::ffi::{FFIException, Signature};
use crate::ffi::sync_fn::{Function, FunctionBase, OwnershipGuard, VMContext, value_into_ref};
use crate::util::void::Void;
use crate::vm::al31f::Combustor;
use crate::vm::al31f::alloc::Alloc;
use crate::vm::al31f::compiled::{CompiledFunction, CompiledProgram, ExceptionHandlingBlock};
use crate::vm::al31f::insc::Insc;

pub fn basic_program<A: Alloc>() -> CompiledProgram<A> {
    CompiledProgram {
        code: boxed_slice![
            Insc::AddInt(0, 1, 0),
            Insc::Return(boxed_slice![0])
        ],
        const_pool: boxed_slice![],
        init_proc: 0,
        functions: boxed_slice![
            CompiledFunction::new(0, 2, 1, 2, boxed_slice![])
        ],
        ffi_funcs: boxed_slice![],
        #[cfg(feature = "async")]
        async_ffi_funcs: boxed_slice![],
    }
}

pub fn basic_fn_call_program<A: Alloc>() -> CompiledProgram<A> {
    CompiledProgram {
        code: boxed_slice![
                                                              // application_start() -> (int)
            /*00*/ Insc::MakeIntConst(1, 0),                  // %0 = $1
            /*01*/ Insc::MakeIntConst(2, 1),                  // %1 = $2
            /*02*/ Insc::Call(
                       1, boxed_slice![0, 1], boxed_slice![0] // [ %0 ] = call sum(%0, %1)
                   ),
            /*03*/ Insc::Return(boxed_slice![0]),             // return [ %0 ]

                                                              // sum(%0, %1) -> (int)
            /*04*/ Insc::AddInt(0, 1, 0),                     // [ %0 ] = add int %0, %1
            /*05*/ Insc::Return(boxed_slice![0])              // return [ %0 ]
        ],
        const_pool: boxed_slice![],
        init_proc: 0,
        functions: boxed_slice![
            CompiledFunction::new(0, 0, 1, 2, boxed_slice![]), // application_start
            CompiledFunction::new(4, 2, 1, 2, boxed_slice![]), // sum
        ],
        ffi_funcs: boxed_slice![],
        #[cfg(feature = "async")]
        async_ffi_funcs: boxed_slice![],
    }
}

pub fn fibonacci_program<A: Alloc>() -> CompiledProgram<A> {
    CompiledProgram {
        code: boxed_slice![
                                                                    // fibonacci(%0) -> (int)
            /*00*/ Insc::MakeIntConst(0, 1),                        // %1 = $0
            /*01*/ Insc::LeInt(0, 1, 2),                            // %2 = le int %0, %1
            /*02*/ Insc::JumpIfTrue(2, 12),                         // if %2 goto 12
            /*03*/ Insc::MakeIntConst(1, 1),                        // %1 = $1
            /*04*/ Insc::EqValue(0, 1, 2),                          // %2 = eq int %0, %1
            /*05*/ Insc::JumpIfTrue(2, 12),                         // if %2 goto 12
            /*06*/ Insc::SubInt(0, 1, 2),                           // %2 = sub int %0, %1
            /*07*/ Insc::MakeIntConst(2, 1),                        // %1 = $2
            /*08*/ Insc::SubInt(0, 1, 3),                           // %3 = sub int %0, %1
            /*09*/ Insc::Call(0, boxed_slice![2], boxed_slice![2]), // [ %2 ] = call fibonacci(%2)
            /*10*/ Insc::Call(0, boxed_slice![3], boxed_slice![3]), // [ %3 ] = call fibonacci(%3)
            /*11*/ Insc::AddInt(2, 3, 1),                           // %1 = add %2, %3
            /*12*/ Insc::ReturnOne(1)                               // return %1
        ],
        const_pool: boxed_slice![],
        init_proc: 0,
        functions: boxed_slice![
            CompiledFunction::new(0, 1, 1, 4, boxed_slice![])
        ],
        ffi_funcs: boxed_slice![],
        #[cfg(feature = "async")]
        async_ffi_funcs: boxed_slice![],
    }
}

pub fn alloc_1m_program<A: Alloc>() -> CompiledProgram<A> {
    CompiledProgram {
        code: boxed_slice![
                                                      // alloc_1m()
            /*00*/ Insc::MakeIntConst(0, 0),          // %0 = $0
            /*01*/ Insc::MakeIntConst(1, 1),          // %1 = $1
            /*02*/ Insc::MakeIntConst(10_000_000, 2), // %2 = $10_000_000
            /*03*/ Insc::EqValue(0, 2, 3),            // %3 = eq value %0, %2
            /*04*/ Insc::JumpIfTrue(3, 8),            // if %3 goto L.8
            /*05*/ Insc::CreateObject(3),             // %3 = new object
            /*06*/ Insc::SubInt(2, 1, 2),             // %2 = sub int %2, %1
            /*07*/ Insc::Jump(3),                     // goto L.3
            /*08*/ Insc::ReturnNothing                // return
        ],
        const_pool: boxed_slice![],
        init_proc: 0,
        functions: boxed_slice![
            CompiledFunction::new(0, 0, 0, 4, boxed_slice![])
        ],
        ffi_funcs: boxed_slice![],
        #[cfg(feature = "async")]
        async_ffi_funcs: boxed_slice![],
    }
}

pub fn exception_program<A: Alloc>() -> CompiledProgram<A> {
    CompiledProgram {
        code: boxed_slice![
                                                                  // foo() -> ()
            /*00*/ Insc::MakeIntConst(12345, 0),                  // %0 = $12345
            /*01*/ Insc::Call(1, boxed_slice![], boxed_slice![]), // call bar()
            /*02*/ Insc::ReturnOne(0),                            // return %0

                                                                  // foo:eh:Object
            /*03*/ Insc::MakeIntConst(114514, 0),                 // %0 = $114514
            /*04*/ Insc::ReturnOne(0),                            // return %0


                                                                  // bar() -> ()
            /*05*/ Insc::Call(2, boxed_slice![], boxed_slice![]), // call baz()
            /*06*/ Insc::ReturnNothing,                           // return

                                                                  // baz() -> !
            /*07*/ Insc::CreateObject(0),                         // %0 = create-object
            /*08*/ Insc::Raise(0)                                 // raise %0
        ],
        const_pool: boxed_slice![],
        init_proc: 0,
        functions: boxed_slice![
            CompiledFunction::new_with_exc(0, 0, 1, 1, boxed_slice![], boxed_slice![
                ExceptionHandlingBlock::new(0, 2, <Void as StaticBase<Object>>::type_id(), 3)
            ]),
            CompiledFunction::new(5, 0, 0, 0, boxed_slice![]),
            CompiledFunction::new(7, 0, 0, 1, boxed_slice![])
        ],
        ffi_funcs: boxed_slice![],
        #[cfg(feature = "async")]
        async_ffi_funcs: boxed_slice![]
    }
}

pub fn exception_no_eh_program<A: Alloc>() -> CompiledProgram<A> {
    CompiledProgram {
        code: boxed_slice![
                                                                   // foo() -> (int)
            /*00*/ Insc::Call(1, boxed_slice![], boxed_slice![0]), // %0 = call bar
            /*01*/ Insc::ReturnOne(0),

                                                                   // bar() -> !
            /*02*/ Insc::CreateObject(0),                          // %0 = create-object
            /*03*/ Insc::Raise(0),                                 // raise %0
        ],
        const_pool: boxed_slice![],
        init_proc: 0,
        functions: boxed_slice![
            CompiledFunction::new(0, 0, 1, 1, boxed_slice![]),
            CompiledFunction::new(2, 0, 1, 1, boxed_slice![])
        ],
        ffi_funcs: boxed_slice![],
        #[cfg(feature = "async")]
        async_ffi_funcs: boxed_slice![]
    }
}

#[inline(never)] fn ffi_function(x: &Object, y: &Object, z: &Object) {
    assert_eq!(x as *const Object as usize, y as *const Object as usize);
    assert_eq!(y as *const Object as usize, z as *const Object as usize);
}

#[allow(non_camel_case_types)]
struct Pr47Binder_ffi_function();

impl FunctionBase for Pr47Binder_ffi_function {
    fn signature() -> Signature {
        todo!()
    }

    fn call_tyck<CTX: VMContext>(
        _context: &mut CTX,
        _args: &[Value],
        _rets: &[*mut Value]
    ) -> Result<(), FFIException> {
        todo!()
    }

    unsafe fn call_rtlc<CTX: VMContext>(
        _context: &mut CTX,
        args: &[Value],
        rets: &[*mut Value]
    ) -> Result<(), FFIException> {
        debug_assert_eq!(args.len(), 3);
        debug_assert_eq!(rets.len(), 0);

        let (a1, g1): (&Object, OwnershipGuard) = value_into_ref(*args.get_unchecked(0))?;
        let (a2, g2): (&Object, OwnershipGuard) = value_into_ref(*args.get_unchecked(1))?;
        let (a3, g3): (&Object, OwnershipGuard) = value_into_ref(*args.get_unchecked(2))?;

        ffi_function(a1, a2, a3);

        std::mem::drop(g3);
        std::mem::drop(g2);
        std::mem::drop(g1);

        Ok(())
    }

    unsafe fn call_unchecked<CTX: VMContext>(
        _context: &mut CTX,
        _args: &[Value],
        _rets: &[*mut Value]
    ) -> Result<(), FFIException> {
        todo!()
    }
}

pub fn ffi_call_program<A: Alloc>() -> CompiledProgram<A> {
    CompiledProgram {
        code: boxed_slice![
                                                           // main() -> ()
            /*00*/ Insc::CreateObject(0),                  // %0 = create-object
            /*01*/ Insc::FFICall(0, boxed_slice![0, 0, 0], // ffi-call @0(%0, %0, %0)
                                 boxed_slice![]),
            /*02*/ Insc::ReturnNothing                     // return
        ],
        const_pool: boxed_slice![],
        init_proc: 0,
        functions: boxed_slice![
            CompiledFunction::new(0, 0, 0, 1, boxed_slice![])
        ],
        ffi_funcs: boxed_slice![
            Box::new(Pr47Binder_ffi_function()) as Box<dyn Function<Combustor<A>>>
        ],
        async_ffi_funcs: boxed_slice![]
    }
}
