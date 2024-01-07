use mozjs::{
    jsapi::{EnterRealm, JS_NewGlobalObject, OnNewGlobalHookOption},
    rust::{JSEngine, RealmOptions, Runtime, SIMPLE_GLOBAL_CLASS},
};

mod test_function_binding;
mod test_js_class;

fn start_js_test_env() -> (JSEngine, Runtime) {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE)
        .init();

    tracing::info!("starting spidermonkey engine");
    let engine = JSEngine::init().expect("failed to initalize JS engine");
    tracing::info!("starting spidermonkey runtime");
    let runtime = Runtime::new(engine.handle());
    assert!(!runtime.cx().is_null(), "failed to create JSContext");

    let span = tracing::span!(tracing::Level::TRACE, "enter global realm");
    let _enter = span.enter();
    let options = RealmOptions::default();
    unsafe {
        EnterRealm(
            runtime.cx(),
            JS_NewGlobalObject(
                runtime.cx(),
                &SIMPLE_GLOBAL_CLASS,
                std::ptr::null_mut(),
                OnNewGlobalHookOption::FireOnNewGlobalHook,
                &*options,
            ),
        );
    }
    return (engine, runtime);
}
