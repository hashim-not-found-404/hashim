use crate::prelude::*;

pub struct RuntimeS;

// #[cfg(not(target_arch = "wasm32"))]
impl Runtime for RuntimeS {
    fn spawn<F: Future>(fut: F) {
        todo!()
    }

    async fn timeout<T, F: Future<Output = T>>(
        duration: std::time::Duration,
        fut: F,
    ) -> Result<T, ()> {
        todo!()
    }
}
