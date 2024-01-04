extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, token::Token, ItemFn};

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

    let expanded = quote! {
        extern "C" fn #export_fn_name (ctx: *mut JSContext, argc: u32, args: *mut Value) -> bool {
            let span = tracing::span!(tracing::Level::TRACE, stringify!(#fn_name));
            let _enter = span.enter();
            unsafe {
                let args = unsafe { CallArgs::from_vp(args, argc) };

                #(#convert_parts)*

                let r = #fn_name(#(#var_names),*);

                ToJSValConvertible::to_jsval(
                    &r,
                    ctx,
                    mozjs::rust::MutableHandle::from_raw(args.rval()),
                );

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
