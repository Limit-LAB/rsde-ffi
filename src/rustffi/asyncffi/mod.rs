// pub struct RustJSFuture<T: Any + Clone + Send + ToJSValConvertible>(BoxFuture<'static, T>);

// async fn add_one(x: u32) -> u32 {
//     x + 1
// }

// fn add_one_js() {
//     let f = unsafe { BoxFuture::new_unchecked(Box::new(add_one(1))) };
//     let (s,rw) = futures::channel::oneshot::channel();
//     s.send(f);
//     // let (sw,r) = futures::channel::oneshot::channel();
// }

// fn shit(){
//     JS_NewFunction(cx, call, nargs, flags, name)
//     // move rw to workers pool
//     // move r to js value
// }
