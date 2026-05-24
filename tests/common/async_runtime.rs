use std::{future::Future, sync::OnceLock};

use tokio::runtime::{Builder, Runtime};

static TEST_TOKIO_RUNTIME: OnceLock<Runtime> = OnceLock::new();

pub fn tokio_test_runtime() -> &'static Runtime {
    TEST_TOKIO_RUNTIME.get_or_init(|| {
        Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed to build Test Environment Tokio Runtime")
    })
}

/// Blocks until the future is complete, meant to be used with emulator tests.
pub fn block_on<F: Future>(future: F) -> F::Output {
    tokio_test_runtime().block_on(future)
}
