pub trait UnifiedServerInterface: ClientConnection {
    fn add(a: usize, b: bool) -> usize;
    fn set_slot(a: &AbsoluteCPtr);
    fn handle(a: &LocalHandle<ConnectionHandle>);
    fn option(a: Option<usize>);
    fn unimplement(a: foo);
}