pub trait UnifiedServerInterface: ClientConnection {
    fn add(a: usize, b: Option<bool>, c: usize) -> usize;
    fn set_slot(a: &AbsoluteCPtr);
    fn handle(a: &LocalHandle<ConnectionHandle>);
    fn option(a: Option<&AbsoluteCPtr>);
    fn unimplement(a: foo);
}