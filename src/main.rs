#![allow(
    non_upper_case_globals,
    non_camel_case_types,
    non_snake_case,
    improper_ctypes
)]
pub mod rustffi;

use ::std::ptr;
use mozjs::conversions::{ConversionResult, FromJSValConvertible, ToJSValConvertible};
use mozjs::jsapi::*;
use mozjs::jsval::UndefinedValue;
use mozjs::rooted;
use mozjs::rust::{define_methods, SIMPLE_GLOBAL_CLASS};
use mozjs::rust::{JSEngine, RealmOptions, Runtime};
use rsde_ffi_derive::rsde_ffi;
use tracing::Level;

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .init();

    tracing::info!("starting spidermonkey engine");
    let engine = JSEngine::init().expect("failed to initalize JS engine");
    tracing::info!("starting spidermonkey runtime");
    let runtime = Runtime::new(engine.handle());
    assert!(!runtime.cx().is_null(), "failed to create JSContext");

    let span = tracing::span!(Level::TRACE, "enter global realm");
    let _enter = span.enter();
    let options = RealmOptions::default();
    unsafe {
        EnterRealm(
            runtime.cx(),
            JS_NewGlobalObject(
                runtime.cx(),
                &SIMPLE_GLOBAL_CLASS,
                ptr::null_mut(),
                OnNewGlobalHookOption::FireOnNewGlobalHook,
                &*options,
            ),
        );
    }

    #[rsde_ffi]
    fn print(s: String) -> i32 {
        println!("fromjs: {}", s);
        114514
    }

    unsafe {
        tracing::info!("define global this");
        rooted!(in(runtime.cx()) let global = CurrentGlobalOrNull(runtime.cx()));
        if global.is_null() {
            panic!("no global")
        }
        tracing::info!("define methods");
        define_methods(runtime.cx(), global.handle(), RSDE_FFI_print_SPEC).unwrap();

        rooted!(in(runtime.cx()) let mut rval = UndefinedValue());

        tracing::info!("evaluating script");
        runtime
            .evaluate_script(
                global.handle(),
                "let s = print('akara 吃答辩， rend 开 BYDBYD'); s == 114514",
                "fuck.js",
                0,
                rval.handle_mut(),
            )
            .unwrap();
    }
}

// extern "C" fn RSDE_FFI_print(ctx: *mut JSContext, argc: u32, args: *mut Value) -> bool {
//     let span = tracing::span!(tracing::Level::TRACE, "RSDE_FFI_print");
//     let _enter = span.enter();
//     unsafe {
//         let args = unsafe { CallArgs::from_vp(args, argc) };
//         let a0 = args.get(0);
//         // ignore this
//         if a0.is_int32() {
//             println!("i32 {:?}", a0.to_int32());
//             return true;
//         }
//         // a1 a2 a3 a4 ...
//         let a0 = FromJSValConvertible::from_jsval(
//             ctx,
//             mozjs::rust::Handle::from_raw(a0),
//             Default::default(),
//         )
//         .unwrap();
//         let a0 = if let ConversionResult::Success(a1) = a0 {
//             a1
//         } else {
//             panic!("failed to convert jsval to rust type")
//         };
//         let r = print(a0);
//         // return
//         ToJSValConvertible::to_jsval(
//             &r,
//             ctx,
//             mozjs::rust::MutableHandle::from_raw(args.rval()),
//         );

//         true
//     }
// }

// const m: &'static [JSFunctionSpec; 2] = &[
//     JSFunctionSpec {
//         name: JSPropertySpec_Name {
//             string_: b"print\0".as_ptr() as *const _,
//         },
//         call: JSNativeWrapper {
//             op: Some(RSDE_FFI_print),
//             info: ptr::null(),
//         },
//         nargs: 1,
//         flags: 0,
//         selfHostedName: ptr::null(),
//     },
//     JSFunctionSpec::ZERO,
// ];
