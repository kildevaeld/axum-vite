use crate::{
    error::ViteError,
    loader::AssetLoader,
    manifest::Manifest,
    template::{Asset, AssetKind, Payload, Template},
};
use axum::{
    extract::Request,
    handler::Handler,
    http::Uri,
    response::{Html, IntoResponse, Response},
};
use core::fmt;
use std::{
    future::Future,
    path::{Path, PathBuf},
    pin::Pin,
    sync::Arc,
};

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

pub struct ServerEntry<'a> {
    pub entry: &'a str,
    pub output: Option<&'a str>,
    pub manifest: Option<&'a str>,
}

impl<'a> ServerEntry<'a> {
    pub fn new(entry: &'a str) -> ServerEntry<'a> {
        ServerEntry {
            entry,
            output: Some("server"),
            manifest: Some(".vite/manfest.json"),
        }
    }

    pub fn output(mut self, path: &'a str) -> Self {
        self.output = Some(path);
        self
    }

    pub fn manifest(mut self, path: &'a str) -> Self {
        self.manifest = Some(path);
        self
    }

    pub fn entry_path(&self, root: &Path) -> PathBuf {
        root.join(self.entry)
    }

    pub fn output_path(&self, root: &Path) -> PathBuf {
        root.join(self.output.unwrap_or("client"))
    }

    pub fn manifest_path(&self, root: &Path) -> PathBuf {
        root.join(self.manifest.unwrap_or(".vite/manifest.json"))
    }
}

pub struct ClientEntry<'a> {
    pub entry: &'a str,
    pub output: Option<&'a str>,
    pub manifest: Option<&'a str>,
    pub ssr_manifest: Option<&'a str>,
}

impl<'a> ClientEntry<'a> {
    pub fn new(entry: &'a str) -> ClientEntry<'a> {
        ClientEntry {
            entry,
            output: None,
            manifest: None,
            ssr_manifest: None,
        }
    }

    pub fn output(mut self, path: &'a str) -> Self {
        self.output = Some(path);
        self
    }

    pub fn manifest(mut self, path: &'a str) -> Self {
        self.manifest = Some(path);
        self
    }

    pub fn ssr_manifest(mut self, path: &'a str) -> Self {
        self.ssr_manifest = Some(path);
        self
    }

    pub fn entry_path(&self, root: &Path) -> PathBuf {
        root.join(self.entry)
    }

    pub fn output_path(&self, root: &Path) -> PathBuf {
        root.join(self.output.unwrap_or("client"))
    }

    pub fn manifest_path(&self, root: &Path) -> PathBuf {
        root.join(self.manifest.unwrap_or(".vite/manifest.json"))
    }

    pub fn ssr_manifest_path(&self, root: &Path) -> PathBuf {
        root.join(self.ssr_manifest.unwrap_or(".vite/ssr-manifest.json"))
    }
}

pub enum Mode {
    /// Server side rendering
    SSR,
    // Client side rendering
    CSR,
}

pub struct ViteSSROptions<'a> {
    pub path: &'a Path,
    pub asset_path: &'a str,
    pub server: ServerEntry<'a>,
    pub client: ClientEntry<'a>,
}

pub struct ViteCSROptions<'a> {
    pub path: &'a Path,
    pub asset_path: &'a str,
    pub manifest: &'a str,
    pub entry: &'a str,
}

pub struct ViteDevOptions<'a> {
    pub uri: Uri,
    pub entry: &'a str,
}

impl<'a> ViteDevOptions<'a> {
    pub fn build<S, T>(self, template: T) -> Vite<S>
    where
        T: Template<S> + Send + Sync + 'static,
    {
        let assets = vec![
            Asset {
                path: format!("{}/@vite/client", self.uri),
                kind: AssetKind::Script,
            },
            Asset {
                path: format!("{}/{}", self.uri, self.entry),
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
}

impl<'a> ViteCSROptions<'a> {
    pub async fn build<S, T>(self, template: T) -> Result<Vite<S>, ViteError>
    where
        T: Template<S> + Send + Sync + 'static,
    {
        let manifest: Manifest = load_json(&self.path.join(self.manifest)).await?;

        let Some(entry) = manifest.get(self.entry) else {
            panic!("entrypoint not found");
        };

        let mut assets = vec![Asset {
            path: entry.file.clone(),
            kind: AssetKind::Script,
        }];

        for css in &entry.css {
            assets.push(Asset {
                path: css.clone(),
                kind: AssetKind::Style,
            });
        }

        Ok(Vite(Arc::new(ViteInner {
            template: Box::new(template),
            assets: Box::new(Arc::new(Payload {
                assets,
                content: None,
            })),
        })))
    }
}

// impl<'a> ViteSSROptions<'a> {
//     pub async fn build<S, T, F>(self, template: T, fetcher: F) -> Result<Vite<S>, ViteError>
//     where
//         T: Template<S> + Send + Sync + 'static,
//     {
//     }
// }
// impl<'a> ViteOptions<'a> {
//     pub async fn build<T>(self) -> Result<Vite<T::Renderer>, ViteError>
//     where
//         T: RenderFactory,
//         T::Error: std::error::Error + Send + Sync + 'static,
//     {
//         self.build_with::<T, _>(NoopFetcher).await
//     }

//     pub async fn build_with<T, F>(self, fetcher: F) -> Result<Vite<T::Renderer>, ViteError>
//     where
//         T: RenderFactory,
//         T::Error: std::error::Error + Send + Sync + 'static,
//         F: LocalFetch + Send + Sync + 'static + Clone,
//     {
//         let path = self.path.canonicalize()?;

//         let client_path = self.client.output_path(&path);
//         let server_path = self.server.output_path(&path);

//         let client_manifest_path = self.client.manifest_path(&client_path);
//         let client_ssr_manifest_path = self.client.ssr_manifest_path(&client_path);

//         let server_manifest_path = self.server.manifest_path(&server_path);

//         let client_manifest: Manifest = load_json(&client_manifest_path).await?;
//         let client_entry =
//             client_manifest
//                 .get(self.client.entry)
//                 .ok_or_else(|| ViteError::Manifest {
//                     path: self.client.entry.to_string(),
//                 })?;

//         let server_manifest: Manifest = load_json(&server_manifest_path).await?;
//         let server_entry =
//             server_manifest
//                 .get(self.server.entry)
//                 .ok_or_else(|| ViteError::Manifest {
//                     path: self.client.entry.to_string(),
//                 })?;

//         let manifest = load_json(&client_ssr_manifest_path).await?;

// let mut assets = vec![Asset {
//     file: client_entry.file.clone(),
//     kind: AssetKind::Script,
// }];

// for css in &client_entry.css {
//     assets.push(Asset {
//         file: css.clone(),
//         kind: AssetKind::Styling,
//     });
// }

//         let file = server_path.join(&server_entry.file);

//         let renderer = T::create(move || Worker::new(&file).fetcher(fetcher.clone()))
//             .await
//             .map_err(|err| ViteError::Render(Box::new(err)))?;

//         Ok(Vite {
//             manifest,
//             assets,
//             renderer,
//         })
//     }
// }

struct ViteInner<S> {
    template: Box<dyn Template<S> + Send + Sync>,
    assets: Box<dyn AssetLoader<S> + Send + Sync>,
}

impl<S> ViteInner<S> {
    async fn render(&self, req: Request, state: S) -> Response {
        let payload = match self.assets.load(&req, &state).await {
            Ok(ret) => ret,
            Err(err) => {
                todo!("render error")
            }
        };

        match self.template.render(&req, &state, payload) {
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
    type Future = Pin<Box<dyn Future<Output = Response> + Send>>;

    fn call(self, req: axum::extract::Request, state: S) -> Self::Future {
        Box::pin(async move { self.0.render(req, state).await })
    }
}

async fn load_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, ViteError> {
    let cmb = tokio::fs::read(path).await?;
    serde_json::from_slice(&cmb).map_err(|_| ViteError::Manifest {
        path: path.display().to_string(),
    })
}
