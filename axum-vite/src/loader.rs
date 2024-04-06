use crate::{error::ViteError, template::Payload};
use async_trait::async_trait;
use axum::extract::Request;
use std::sync::Arc;

#[async_trait]
pub trait AssetLoader<S> {
    async fn load(&self, req: &Request, state: &S) -> Result<Arc<Payload>, ViteError>;
}

#[async_trait]
impl<S> AssetLoader<S> for Arc<Payload> {
    async fn load(&self, _req: &Request, _state: &S) -> Result<Arc<Payload>, ViteError> {
        Ok(self.clone())
    }
}
