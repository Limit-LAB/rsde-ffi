use std::{cell::UnsafeCell, ptr::null};

use jstraceable_derive::JSTraceable;
use mozjs::gc::Traceable;
use mozjs::rust::{Handle, IntoHandle};
use mozjs::typedarray::JSObjectStorage;
use mozjs::{
    glue::JS_GetReservedSlot,
    jsapi::{
        CallArgs, CurrentGlobalOrNull, GCContext, JSClass, JSClassOps, JSContext, JSFunctionSpec,
        JSObject, JS_InitClass, JS_SetReservedSlot, ObjectOps, Value, JSCLASS_RESERVED_SLOTS_SHIFT,
    },
    jsval::{ObjectOrNullValue, PrivateValue, UndefinedValue},
    rooted,
    rust::Trace,
    JSCLASS_RESERVED_SLOTS_MASK,
};
use rsde_ffi_derive::{rsde_ffi_ctor, rsde_ffi_method, RSDEJSClass};

use mozjs::conversions::ConversionResult;
use mozjs::conversions::FromJSValConvertible;
use mozjs::conversions::ToJSValConvertible;
use mozjs::jsapi::JSNativeWrapper;
use mozjs::jsapi::JSTracer;
use mozjs::jsapi::{GetRealmObjectPrototype, JSPropertySpec_Name, JS_NewObjectForConstructor};
use std::ptr::{self};

#[derive(RSDEJSClass, Debug, JSTraceable)]
#[export(show_n_set_s, show_a_plus1)]
struct TestJSClass {
    s: String,
    a: i32,
}

impl TestJSClass {
    #[rsde_ffi_ctor]
    fn new(s: String, a: i32) -> Self {
        Self { s, a }
    }

    #[rsde_ffi_method]
    fn show_n_set_s(&mut self, s: String) -> i32 {
        println!("s: {}, a: {}", self.s, self.a);
        println!("received s: {}", s);
        self.s = s;
        self.a = 114513;
        self.a
    }

    #[rsde_ffi_method]
    fn show_a_plus1(&self) -> i32 {
        println!("s: {}, a: {}", self.s, self.a);
        self.a + 1
    }
}

impl Drop for TestJSClass {
    fn drop(&mut self) {
        tracing::info!("dropping TestJSClass");
    }
}

#[test]
fn test() {
    let (_engine, runtime) = super::start_js_test_env();
    tracing::info!("define global this");
    rooted!(in(runtime.cx()) let global = unsafe{CurrentGlobalOrNull(runtime.cx())});
    if global.is_null() {
        panic!("no global")
    }
    TestJSClass::register_to_global(runtime.cx(), global.handle());

    let script = r#"
function f(){
    let obj = new TestJSClass('akara', 0);
    obj.show_n_set_s('rend');
    return obj.show_a_plus1();
}
let a = f();
a
    "#;

    rooted!(in(runtime.cx()) let mut rval = UndefinedValue());
    let r = runtime.evaluate_script(
        global.handle(),
        script,
        "newjsclass.js",
        0,
        rval.handle_mut(),
    );
    if let Err(e) = r {
        tracing::error!("failed to evaluate script: {:?}", e);
    }
    assert!(r.is_ok());
    assert_eq!(rval.to_int32(), 114514);
}
