pub trait UnifiedServerInterface: ClientConnection {
    fn add(a: usize, b: bool, c: &AbsoluteCPtr, d: &LocalHandle<ConnectionHandle>) -> usize {}
}