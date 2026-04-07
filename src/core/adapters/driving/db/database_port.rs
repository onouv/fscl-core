use std::pin::Pin;



pub trait DatabasePort: Clone + Send + Sync {
    type Error: Send;
    type Tx<'tx>: Send;

    fn execute_in_transaction<T, F>(&self, operation: F) -> impl Future<Output = Result<T, Self::Error>> + Send + '_
    where
        T: Send,
        F: for<'tx> FnOnce(
                &'tx mut Self::Tx<'tx>,
            ) -> Pin<Box<dyn Future<Output = Result<T, Self::Error>> + Send + 'tx>>
            + Send
            + 'static;
}