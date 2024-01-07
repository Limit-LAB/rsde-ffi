extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, ItemFn};

#[proc_macro_attribute]
pub fn rsde_ffi(_args: TokenStream, mut input: TokenStream) -> TokenStream {
    let i = input.clone();
    let ast = parse_macro_input!(i as ItemFn);

    let fn_name = &ast.sig.ident;
    // let return_type = &ast.sig.output;

    let inputs = &ast.sig.inputs;
    let input_counts: usize = inputs.len();
    let var_names = inputs
        .iter()
        .enumerate()
        .map(|(i, _)| format_ident!("a{}", i));
    let convert_parts: Vec<_> = inputs
        .iter()
        .enumerate()
        .map(|(i, _)| {
            let var = format_ident!("a{}", i);
            quote! {
                let #var = args.get(#i as u32);
                let #var = FromJSValConvertible::from_jsval(
                    ctx,
                    mozjs::rust::Handle::from_raw(#var),
                    Default::default(),
                )
                .unwrap();
                let #var = if let ConversionResult::Success(x) = #var {
                    x
                } else {
                    panic!("failed to convert jsval to rust type")
                };
            }
        })
        .collect();

    let export_fn_name = format_ident!("RSDE_FFI_{}", fn_name);
    let export_fn_spec = format_ident!("RSDE_FFI_{}_SPEC", fn_name);

    let fn_name_cstr = format!("{}\0", fn_name.to_string());
    let fn_name_cstr =
        syn::LitByteStr::new(fn_name_cstr.as_bytes(), proc_macro2::Span::call_site());

    let calling_ffi = syn::LitStr::new(
        &format!("calling ffi function {}", fn_name.to_string()),
        proc_macro2::Span::call_site(),
    );

    let returning = syn::LitStr::new(
        &format!("returning from {}", fn_name.to_string()),
        proc_macro2::Span::call_site(),
    );
    let expanded = quote! {
        extern "C" fn #export_fn_name (ctx: *mut JSContext, argc: u32, args: *mut Value) -> bool {
            let span = tracing::span!(tracing::Level::TRACE, stringify!(#fn_name));
            let _enter = span.enter();
            unsafe {
                let args = unsafe { CallArgs::from_vp(args, argc) };

                #(#convert_parts)*

                tracing::trace!(#calling_ffi);
                let r = #fn_name(#(#var_names),*);

                ToJSValConvertible::to_jsval(
                    &r,
                    ctx,
                    mozjs::rust::MutableHandle::from_raw(args.rval()),
                );
                tracing::trace!(#returning);
                true
            }
        }

        const #export_fn_spec: &'static [JSFunctionSpec; 2] = &[
            JSFunctionSpec {
                name: JSPropertySpec_Name {
                    string_: #fn_name_cstr.as_ptr() as *const _,
                },
                call: JSNativeWrapper {
                    op: Some(#export_fn_name),
                    info: ptr::null(),
                },
                nargs: #input_counts as u16,
                flags: 0,
                selfHostedName: ptr::null(),
            },
            JSFunctionSpec::ZERO,
        ];
    };

    TokenStream::extend(&mut input, TokenStream::from(expanded));
    input
}

#[proc_macro_attribute]
pub fn rsde_ffi_ctor(_args: TokenStream, mut input: TokenStream) -> TokenStream {
    let i = input.clone();
    let ast = parse_macro_input!(i as ItemFn);

    let fn_name = &ast.sig.ident;
    // let return_type = &ast.sig.output;

    let inputs = &ast.sig.inputs;
    let var_names = inputs
        .iter()
        .enumerate()
        .map(|(i, _)| format_ident!("a{}", i));
    let convert_parts: Vec<_> = inputs
        .iter()
        .enumerate()
        .map(|(i, _)| {
            let var = format_ident!("a{}", i);
            quote! {
                let #var = args.get(#i as u32);
                let #var = FromJSValConvertible::from_jsval(
                    ctx,
                    mozjs::rust::Handle::from_raw(#var),
                    Default::default(),
                )
                .unwrap();
                let #var = if let ConversionResult::Success(x) = #var {
                    x
                } else {
                    panic!("failed to convert jsval to rust type")
                };
            }
        })
        .collect();

    let export_fn_name = format_ident!("RSDE_FFI_CTOR_{}", fn_name);
    let export_dtor_name = format_ident!("RSDE_FFI_DTOR_{}", fn_name);

    let calling_ffi = syn::LitStr::new(
        &format!("calling ctor ffi function {}", fn_name.to_string()),
        proc_macro2::Span::call_site(),
    );

    let returning = syn::LitStr::new(
        &format!("returning ctor from {}", fn_name.to_string()),
        proc_macro2::Span::call_site(),
    );
    let expanded = quote! {
        unsafe extern "C" fn #export_fn_name (ctx: *mut JSContext, argc: u32, args: *mut Value) -> bool {
            let span = tracing::span!(tracing::Level::TRACE, stringify!(#fn_name));
            let _enter = span.enter();
            let args = unsafe { CallArgs::from_vp(args, argc) };

            #(#convert_parts)*

            tracing::trace!(#calling_ffi);
            let r = Self::#fn_name(#(#var_names),*);

            tracing::trace!("allocating object");
            let val = Box::leak(Box::new(r));
            tracing::trace!("self ptr address: {:?}", val as &Self as *const Self);
            let val = PrivateValue(val as &Self as *const Self as *const _);
            tracing::trace!("allocating default JS object");
            rooted!(in(ctx) let mut plain_obj = {
                let plain_obj = JS_NewObjectForConstructor(ctx, &Self::class, &args);
                if plain_obj.is_null() {
                    tracing::error!("failed to create plain object");
                }
                assert!(!plain_obj.is_null());
                plain_obj
            });
            tracing::trace!("setting reserved slot");
            JS_SetReservedSlot(plain_obj.handle_mut().as_raw(), 0, &val);
            tracing::trace!("setting rval of ctor");
            args.rval().set(ObjectOrNullValue(plain_obj.handle_mut().as_raw()));
            tracing::trace!(#returning);
            true
        }

        unsafe extern "C" fn #export_dtor_name(gc: *mut GCContext, obj: *mut JSObject) {
            let mut dest = UndefinedValue();
            JS_GetReservedSlot(obj, 0, &mut dest);
            if dest.is_undefined() {
                tracing::warn!("reserved slot 0 is undefined, I actually don't know what to do");
            } else{
                let dest = dest.to_private();
                let self_ptr : *mut Self = dest as *mut Self;
                std::mem::drop(Box::from_raw(self_ptr));
            }
        }


        unsafe extern "C" fn RSDE_FFI_TRACE (tracer: *mut JSTracer, self_o: *mut JSObject) {
            tracing::trace!("enter GC trace");
            let mut dest = UndefinedValue();
            JS_GetReservedSlot(self_o, 0, &mut dest);
            let dest = dest.to_private();
            let self_ptr : *mut Self = dest as *mut Self;
            (*self_ptr).trace(tracer);
        }

        const cOps: JSClassOps = JSClassOps {
            addProperty: None,
            delProperty: None,
            enumerate: None,
            newEnumerate: None,
            resolve: None,
            mayResolve: None,
            finalize: Some(Self::#export_dtor_name),
            call: None,
            construct: Some(Self::#export_fn_name),
            trace: Some(Self::RSDE_FFI_TRACE),
        };
        const fn reserved_slots(slots: u32) -> u32 {
            (slots & JSCLASS_RESERVED_SLOTS_MASK) << JSCLASS_RESERVED_SLOTS_SHIFT
        }

        const oOps: ObjectOps = ObjectOps {
            lookupProperty: None,
            defineProperty: None,
            hasProperty: None,
            getProperty: None,
            setProperty: None,
            getOwnPropertyDescriptor: None,
            deleteProperty: None,
            getElements: None,
            // TODO: implement
            funToString: None,
        };
    };

    TokenStream::extend(&mut input, TokenStream::from(expanded));
    input
}

#[proc_macro_attribute]
pub fn rsde_ffi_method(_args: TokenStream, mut input: TokenStream) -> TokenStream {
    let i = input.clone();
    let ast = parse_macro_input!(i as ItemFn);

    let fn_name = &ast.sig.ident;
    // let return_type = &ast.sig.output;

    let inputs = &ast.sig.inputs;
    let input_counts: usize = inputs.len();
    let var_names = inputs
        .iter()
        .enumerate()
        .skip(1)
        .map(|(i, _)| format_ident!("a{}", i));
    let convert_parts: Vec<_> = inputs
        .iter()
        .enumerate()
        .skip(1)
        .map(|(i, _)| {
            let var = format_ident!("a{}", i);
            quote! {
                let #var = args.get((#i - 1) as u32);
                let #var = FromJSValConvertible::from_jsval(
                    ctx,
                    mozjs::rust::Handle::from_raw(#var),
                    Default::default(),
                )
                .unwrap();
                let #var = if let ConversionResult::Success(x) = #var {
                    x
                } else {
                    panic!("failed to convert jsval to rust type")
                };
            }
        })
        .collect();

    let export_fn_name = format_ident!("RSDE_FFI_METHOD_{}", fn_name);
    let export_fn_spec = format_ident!("RSDE_FFI_METHOD_{}_SPEC", fn_name);
    let fn_name_cstr = format!("{}\0", fn_name.to_string());

    // let fn_name_cstr = format!("{}\0", fn_name.to_string());
    // let fn_name_cstr =
    //     syn::LitByteStr::new(fn_name_cstr.as_bytes(), proc_macro2::Span::call_site());

    let calling_ffi = syn::LitStr::new(
        &format!("calling ffi method {}", fn_name.to_string()),
        proc_macro2::Span::call_site(),
    );

    let returning = syn::LitStr::new(
        &format!("returning method {}", fn_name.to_string()),
        proc_macro2::Span::call_site(),
    );

    let expanded = quote! {
        extern "C" fn #export_fn_name (ctx: *mut JSContext, argc: u32, args: *mut Value) -> bool {
            let span = tracing::span!(tracing::Level::TRACE, stringify!(#fn_name));
            let _enter = span.enter();
            unsafe {
                let args = unsafe { CallArgs::from_vp(args, argc) };
                let thisval = args.thisv().get().to_object_or_null();
                if thisval.is_null() {
                    tracing::error!("this is null");
                    return false;
                }
                let mut self_ptr = UndefinedValue();
                JS_GetReservedSlot(thisval, 0, &mut self_ptr);
                let mut self_ptr = self_ptr.to_private();
                if self_ptr.is_null() {
                    tracing::error!("self is null");
                    return false;
                }
                let self_ptr : *mut Self = self_ptr as *mut Self;
                let self_ref = &mut *self_ptr;

                #(#convert_parts)*

                tracing::trace!(#calling_ffi);
                let r = Self::#fn_name(self_ref as _, #(#var_names),*);

                ToJSValConvertible::to_jsval(
                    &r,
                    ctx,
                    mozjs::rust::MutableHandle::from_raw(args.rval()),
                );
                tracing::trace!(#returning);
                true
            }
        }

        const #export_fn_spec : JSFunctionSpec = JSFunctionSpec {
            name: JSPropertySpec_Name {
                string_: #fn_name_cstr.as_ptr() as *const _,
            },
            call: JSNativeWrapper {
                op: Some(Self::#export_fn_name),
                info: ptr::null(),
            },
            nargs: (#input_counts - 1) as u16,
            flags: 0,
            selfHostedName: ptr::null(),
        };

    };

    TokenStream::extend(&mut input, TokenStream::from(expanded));
    input
}

struct ExportsIdents {
    idents: Vec<syn::Ident>,
}

impl syn::parse::Parse for ExportsIdents {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut idents = vec![];
        // ident (, ident)* ,?
        // fuck rust no do while
        idents.push(input.parse::<syn::Ident>()?);
        while input.peek(syn::Token![,]) {
            input.parse::<syn::Token![,]>()?;
            if input.peek(syn::Ident) {
                idents.push(input.parse::<syn::Ident>()?);
            } else {
                break;
            }
        }
        Ok(Self { idents })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_parse_export_idents() {
        let cases = ["method", "method1, method2", "method1, method2,"];
        for case in cases {
            let input = syn::parse_str::<ExportsIdents>(case).unwrap();
            println!("{:?}", input.idents);
        }
    }
}

/// ```no_run
/// #[derive(JSExportMethods)]
/// #[export(method1, method2, method3)]
/// struct TestJSClass { }
/// ````
///
/// and I expect all method signature has &mut self as the first argument
#[proc_macro_derive(RSDEJSClass, attributes(export))]
pub fn rsde_js_class(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    let name = input.ident;
    let attrs = input.attrs;
    // take all attrs with `export`
    let export_attrs: Vec<_> = attrs
        .iter()
        .filter(|attr| attr.path().is_ident("export"))
        .collect();
    let export_methods = export_attrs
        .iter()
        .map(|attr| {
            // Ident (, Ident)* ,?
            let i: ExportsIdents = attr.parse_args().unwrap();
            i.idents
        })
        .flatten()
        .collect::<Vec<_>>();

    let vec_method_names: Vec<_> = export_methods
        .into_iter()
        .map(|i| format_ident!("RSDE_FFI_METHOD_{}_SPEC", i))
        .map(|i| {
            quote! {
                array.push(Self::#i);
            }
        })
        .collect();

    let name_cstr = format!("{}\0", name.to_string());
    let name_cstr = syn::LitByteStr::new(name_cstr.as_bytes(), proc_macro2::Span::call_site());

    let impl_methods = quote! {
        impl #name {
            unsafe fn RSDE_DEFINE_METHODS() -> &'static [JSFunctionSpec] {
                struct TMPARRAY(UnsafeCell<Vec<JSFunctionSpec>>);
                unsafe impl Sync for TMPARRAY {}
                unsafe impl Send for TMPARRAY {}
                static mut ARRAY: TMPARRAY = TMPARRAY(UnsafeCell::new(Vec::new()));
                let array = ARRAY.0.get_mut();

                // push methods
                #(#vec_method_names)*
                // end push methods

                array.push(JSFunctionSpec::ZERO);
                let array = array.as_slice();
                array
            }
            const class : JSClass = JSClass {
                name: #name_cstr.as_ptr() as *const _,
                flags: Self::reserved_slots(1),
                cOps: &Self::cOps,
                spec: null(),
                ext: null(),
                oOps: &Self::oOps,
            };
            fn register_to_global(cx: *mut JSContext, global: Handle<'_, *mut JSObject>) {
                tracing::info!("registering TestJSClass to global");
                rooted!(in(cx) let realm_proto = unsafe{GetRealmObjectPrototype(cx)});
                tracing::info!("init class");
                unsafe {
                    JS_InitClass(
                        cx,
                        global.into_handle(),
                        &Self::class,
                        realm_proto.handle().into_handle(),
                        #name_cstr.as_ptr() as *const _,
                        Some(TestJSClass::RSDE_FFI_CTOR_new),
                        0,
                        null(),
                        Self::RSDE_DEFINE_METHODS().as_ptr(),
                        null(),
                        null(),
                    )
                };
            }
        }
    };

    proc_macro::TokenStream::from(impl_methods)
}
