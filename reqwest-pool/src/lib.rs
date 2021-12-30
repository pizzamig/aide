//! Connection pool for Reqwest
use reqwest::Client;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::{Mutex, Semaphore};

const MAX_POOL_SIZE: usize = 100;

/// Convenient Result redefinition that uses [ReqwestPoolError] Error
pub type Result<T> = ::std::result::Result<T, ReqwestPoolError>;

#[derive(Debug, Clone)]
/// The ReqwestPool opaque struct, used to get a valid Reqwest client
pub struct ReqwestPool {
    pool: Vec<Arc<Mutex<Client>>>,
    semaphore: Arc<Semaphore>,
}

#[derive(Debug, Default)]
/// The Builder opaque struct
pub struct ReqwestPoolBuilder {
    size: usize,
}

#[derive(Debug, Error)]
pub enum ReqwestPoolError {
    #[error("Size {0} is not valid (0 < size < {})", MAX_POOL_SIZE)]
    SizeNotValid(usize),
    #[error("Error acquiring the sempahore (already closed?) [{0:?}]")]
    Semaphore(#[from] tokio::sync::AcquireError),
    #[error("All mutexes are locked (impossible situation)")]
    AllMutexLocked,
}

impl ReqwestPoolBuilder {
    /// Create a new builder, passing the pool size
    ///
    /// The parameter size is checked if it's valid (not zero )and
    /// reasonable (less than 100)
    /// ```rust
    /// let builder = reqwest_pool::ReqwestPoolBuilder::new(3).unwrap();
    /// ```
    pub fn new(size: usize) -> Result<Self> {
        if size == 0 || size > MAX_POOL_SIZE {
            return Err(ReqwestPoolError::SizeNotValid(size));
        }
        Ok(ReqwestPoolBuilder { size })
    }
    /// The build function that creates the @ReqwestPool
    ///
    /// ```rust
    /// # tokio_test::block_on( async {
    /// let builder = reqwest_pool::ReqwestPoolBuilder::new(3).unwrap();
    /// let pool = builder.build().await;
    /// # } )
    /// ```
    pub async fn build(self) -> ReqwestPool {
        let mut pool = Vec::with_capacity(self.size);
        (0..self.size).for_each(|_| {
            let m = Arc::new(Mutex::new(Client::new()));
            pool.push(m);
        });
        let semaphore = Arc::new(Semaphore::new(self.size));
        ReqwestPool { pool, semaphore }
    }
}

use tokio::sync::{MutexGuard, SemaphorePermit};
pub struct Handler<'a> {
    _sp: SemaphorePermit<'a>,
    mg: MutexGuard<'a, Client>,
}

impl<'a> ReqwestPool {
    /// The function return the hanlder that contains the Reqwest client (via [`get_client`])
    ///
    /// This function access the pool and return when a Reqwest is available, otherwise it waits.
    /// To improve Clients availability, it's important to drop the handler as soon as we are done
    /// with the Client
    /// The function can fail if the semaphore, counting the pool availabillity,
    /// has been acquired when closed (usually during shutdown phases)
    ///
    /// The ['ReqwestPoolError::AllMutexLocked'] error should never happen.
    /// ```rust
    /// # tokio_test::block_on( async {
    /// let builder = reqwest_pool::ReqwestPoolBuilder::new(3).unwrap();
    /// let pool = builder.build().await;
    /// let handler = pool.get_handler().await;
    /// assert!(handler.is_ok());
    /// # } )
    /// ```
    pub async fn get_handler(&'a self) -> Result<Handler<'a>> {
        let sp = self.semaphore.acquire().await?;
        for m in &self.pool {
            if let Ok(mg) = (*m).try_lock() {
                return Ok(Handler { _sp: sp, mg });
            }
        }
        Err(ReqwestPoolError::AllMutexLocked)
    }
}

impl<'a> Handler<'a> {
    /// This function returns the reference of the reqwest client inside
    /// the handler
    /// If the connection has been already established, it can be immediately
    /// re-used without TCP or TLS handshake
    /// ```rust
    /// # tokio_test::block_on( async {
    /// let builder = reqwest_pool::ReqwestPoolBuilder::new(3).unwrap();
    /// let pool = builder.build().await;
    /// let handler = pool.get_handler().await.unwrap();
    /// handler.get_client().get("https://httpbin.org").send().await;
    ///
    /// # })
    /// ```
    pub fn get_client(&'a self) -> &'a Client {
        &*self.mg
    }
}
#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
