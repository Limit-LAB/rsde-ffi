#![allow(
    non_upper_case_globals,
    non_camel_case_types,
    non_snake_case,
    improper_ctypes
)]
#![feature(lazy_cell)]
pub mod rustffi;

#[cfg(test)]
mod tests;

fn main() {}

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
