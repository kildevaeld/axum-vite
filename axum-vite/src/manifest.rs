use core::fmt;
use std::{collections::HashMap, future::Future, path::PathBuf, sync::Arc, task::Poll};

use axum::{
    extract::Request,
    handler::Handler,
    http::Uri,
    response::{Html, IntoResponse, Response},
    Router,
};
use pin_project_lite::pin_project;
use serde::de::Visitor;

use crate::{
    error::ViteError,
    template::{Asset, AssetKind, Payload, Template},
};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ManifestEntry {
    file: String,
    #[serde(default)]
    css: Vec<String>,
    #[serde(default, rename = "dynamicImports")]
    dynamic_imports: Vec<String>,
    #[serde(default, rename = "isEntry")]
    is_entry: bool,
    #[serde(default)]
    imports: Vec<String>,
}

// type Manifest = HashMap<String, ManifestEntry>;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct Manifest(HashMap<String, ManifestEntry>);

impl core::ops::Deref for Manifest {
    type Target = HashMap<String, ManifestEntry>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub trait Bundle {
    fn has(&self, path: &str) -> impl Future<Output = bool> + Send;
}

pub struct Options<'a> {
    assets: Option<&'a str>,
}

impl<'a> Options<'a> {
    pub async fn open<B>(self, path: B)
    where
        B: Bundle,
    {
        let assets = self.assets.unwrap_or("assets");
    }
}

pub fn vite_dev<S, T: Template<S> + Send + Sync + 'static>(
    uri: Uri,
    entry: impl fmt::Display,
    template: T,
) -> Vite<S> {
    let assets = vec![
        Asset {
            path: format!("{}/@vite/client", uri),
            kind: AssetKind::Script,
        },
        Asset {
            path: format!("{}/{}", uri, entry),
            kind: AssetKind::Script,
        },
    ];

    Vite(Arc::new(ViteInner {
        template: Box::new(template),
        assets: Box::new(Arc::new(Payload {
            assets,
            content: None,
        })),
    }))
}

pub trait AssetLoader<S> {
    fn load(&self, req: &Request, state: &S) -> Result<Arc<Payload>, ViteError>;
}

impl<S> AssetLoader<S> for Arc<Payload> {
    fn load(&self, _req: &Request, _state: &S) -> Result<Arc<Payload>, ViteError> {
        Ok(self.clone())
    }
}

struct ViteInner<S> {
    template: Box<dyn Template<S> + Send + Sync>,
    assets: Box<dyn AssetLoader<S> + Send + Sync>,
}

impl<S> ViteInner<S> {
    fn render(&self, req: &Request, state: &S) -> Response {
        let payload = match self.assets.load(req, state) {
            Ok(ret) => ret,
            Err(err) => {
                todo!("render error")
            }
        };

        match self.template.render(req, state, payload) {
            Ok(ret) => Html(ret).into_response(),
            Err(err) => {
                todo!("render error")
            }
        }
    }
}

impl<S> fmt::Debug for ViteInner<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ViteInner").finish()
    }
}

#[derive(Debug)]
pub struct Vite<S>(Arc<ViteInner<S>>);

impl<S> Clone for Vite<S> {
    fn clone(&self) -> Self {
        Vite(self.0.clone())
    }
}

impl<T, S: Send + Sync + 'static> Handler<T, S> for Vite<S> {
    type Future = ViteDevFuture<S>;

    fn call(self, req: axum::extract::Request, state: S) -> Self::Future {
        ViteDevFuture {
            inner: self.0.clone(),
            req,
            state,
        }
    }
}

pin_project! {
    pub struct ViteDevFuture<S> {
        inner: Arc<ViteInner<S>>,
        req: axum::extract::Request,
        state: S
    }
}

impl<S> Future for ViteDevFuture<S> {
    type Output = Response;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let this = self.project();
        let resp = this.inner.render(&this.req, &this.state);
        Poll::Ready(resp)
    }
}
