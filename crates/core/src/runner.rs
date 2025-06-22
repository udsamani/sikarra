use tokio_util::sync::CancellationToken;

use crate::error::AppResult;

/// Runner is trait for asynchronous root level task.
///
/// This trait provides a simple abstraction for component that need to perform
/// some kind of asynchronous operation at the root level of an executable
/// binary.
#[async_trait::async_trait]
pub trait Runner<P> {
    /// Name of the Runner
    fn name(&self) -> &str;

    /// Runs the task asynchronously using the provided context.
    async fn run(self, parameters: P, shutdown: CancellationToken) -> AppResult<()>;
}
