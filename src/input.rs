pub trait UnifiedServerInterface: ClientConnection {
    fn add(a: usize, b: Option<bool>, c: usize, d: &AbsoluteCPtr) -> usize;
    fn set_slot(a: &str);
    fn handle(a: &LocalHandle<ConnectionHandle>);
    fn option(a: Option<&AbsoluteCPtr>);
    fn unimplement(a: foo);
}