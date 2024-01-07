use ::std::ptr;
use mozjs::conversions::{ConversionResult, FromJSValConvertible, ToJSValConvertible};
use mozjs::jsapi::*;
use mozjs::jsval::UndefinedValue;
use mozjs::rooted;
use mozjs::rust::define_methods;

use rsde_ffi_derive::rsde_ffi;

#[rsde_ffi]
fn print(s: String) -> i32 {
    println!("fromjs: {}", s);
    114514
}

// TODO: extract common starting code
#[test]
fn test_function_binding() {
    let (_engine, runtime) = super::start_js_test_env();

    tracing::info!("define global this");
    rooted!(in(runtime.cx()) let global = unsafe{CurrentGlobalOrNull(runtime.cx())});
    if global.is_null() {
        panic!("no global")
    }

    unsafe {
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
