use std::sync::Arc;

use crate::error::ViteError;

#[derive(Debug, Clone)]
pub struct Asset {
    pub path: String,
    pub kind: AssetKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssetKind {
    Style,
    Script,
}

pub struct Payload {
    pub assets: Vec<Asset>,
    pub content: Option<Vec<u8>>,
}

pub trait Template<S> {
    fn render(
        &self,
        request: &axum::extract::Request,
        state: &S,
        payload: Arc<Payload>,
    ) -> Result<Vec<u8>, ViteError>;
}

impl<F, S> Template<S> for F
where
    F: Fn(&axum::extract::Request, &S, Arc<Payload>) -> Result<Vec<u8>, ViteError>,
{
    fn render(
        &self,
        request: &axum::extract::Request,
        state: &S,
        payload: Arc<Payload>,
    ) -> Result<Vec<u8>, ViteError> {
        (self)(request, state, payload)
    }
}
