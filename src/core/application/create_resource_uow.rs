use std::future::Future;
use std::pin::Pin;

use crate::core::ports::UnitOfWorkPort;

#[derive(Clone)]
pub struct CreateResourceUow<P: UnitOfWorkPort> {
    port: P,
}

impl<P: UnitOfWorkPort> CreateResourceUow<P> {
    pub fn new(port: P) -> Self {
        Self { port }
    }

    pub fn port(&self) -> &P {
        &self.port
    }
}

impl<P: UnitOfWorkPort> UnitOfWorkPort for CreateResourceUow<P> {
    type Error = P::Error;
    type Tx<'tx> = P::Tx<'tx>;

    fn execute<F>(&self, operation: F) -> impl Future<Output = Result<(), Self::Error>> + Send + '_
    where
        F: for<'tx> FnOnce(
                &'tx mut Self::Tx<'tx>,
            ) -> Pin<Box<dyn Future<Output = Result<(), Self::Error>> + Send + 'tx>>
            + Send
            + 'static,
    {
        self.port.execute(operation)
    }
}

