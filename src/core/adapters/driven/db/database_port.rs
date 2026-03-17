use std::pin::Pin;



pub trait DatabasePort: Clone + Send + Sync {
    type Error: Send;
    type Tx<'tx>: Send;

    fn execute_in_transaction<F>(&self, operation: F) -> impl Future<Output = Result<(), Self::Error>> + Send + '_
    where
        F: for<'tx> FnOnce(
                &'tx mut Self::Tx<'tx>,
            ) -> Pin<Box<dyn Future<Output = Result<(), Self::Error>> + Send + 'tx>>
            + Send
            + 'static;
}