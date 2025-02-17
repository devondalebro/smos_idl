pub mod errors {
    #[derive(Debug)]
    pub enum Error {
        InvalidArg(String),
    }
}
