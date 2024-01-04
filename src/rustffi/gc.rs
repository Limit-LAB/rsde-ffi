/// able to trace using the GC
unsafe trait JSTraceable {
    /// Trace `self`.
    unsafe fn trace(&self, trc: *mut JSTracer);
}

pub unsafe trait CustomTraceable {
    /// Trace `self`.
    unsafe fn trace(&self, trc: *mut JSTracer);
}