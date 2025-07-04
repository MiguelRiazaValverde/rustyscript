#![allow(unused_variables)]
#![allow(clippy::derivable_impls)]
use std::collections::HashSet;

use deno_core::{
    v8::{BackingStore, SharedRef},
    CrossIsolateStore, Extension,
};

pub mod rustyscript;

trait ExtensionTrait<A> {
    fn init(options: A) -> Extension;

    /// Makes a call to `init_ops_and_esm` equivalent to `init_ops`
    fn set_esm(mut ext: Extension, is_snapshot: bool) -> Extension {
        if is_snapshot {
            ext.js_files = ::std::borrow::Cow::Borrowed(&[]);
            ext.esm_files = ::std::borrow::Cow::Borrowed(&[]);
            ext.esm_entry_point = ::std::option::Option::None;
        }
        ext
    }

    /// Builds an extension
    fn build(options: A, is_snapshot: bool) -> Extension {
        let ext = Self::init(options);
        Self::set_esm(ext, is_snapshot)
    }
}

#[cfg(feature = "webidl")]
pub mod webidl;

#[cfg(feature = "broadcast_channel")]
pub mod broadcast_channel;

#[cfg(feature = "cache")]
pub mod cache;

#[cfg(feature = "console")]
pub mod console;

#[cfg(feature = "crypto")]
pub mod crypto;

#[cfg(feature = "fs")]
pub mod fs;

#[cfg(feature = "http")]
pub mod http;

#[cfg(feature = "url")]
pub mod url;

#[cfg(feature = "web")]
pub mod web;

#[cfg(feature = "web_stub")]
pub mod web_stub;

#[cfg(feature = "io")]
pub mod io;

#[cfg(feature = "webstorage")]
pub mod webstorage;

#[cfg(feature = "websocket")]
pub mod websocket;

#[cfg(feature = "ffi")]
pub mod ffi;

#[cfg(feature = "webgpu")]
pub mod webgpu;

#[cfg(feature = "kv")]
pub mod kv;

#[cfg(feature = "cron")]
pub mod cron;

#[cfg(feature = "node_experimental")]
pub mod napi;
#[cfg(feature = "node_experimental")]
pub mod node;
#[cfg(feature = "node_experimental")]
pub mod runtime;

/// Options for configuring extensions
pub struct ExtensionOptions {
    /// Options specific to the `deno_web`, `deno_fetch` and `deno_net` extensions
    ///
    /// Requires the `web` feature to be enabled
    #[cfg(feature = "web")]
    #[cfg_attr(docsrs, doc(cfg(feature = "web")))]
    pub web: web::WebOptions,

    /// Optional seed for the `deno_crypto` extension
    ///
    /// Requires the `crypto` feature to be enabled
    #[cfg(feature = "crypto")]
    #[cfg_attr(docsrs, doc(cfg(feature = "crypto")))]
    pub crypto_seed: Option<u64>,

    /// Configures the stdin/out/err pipes for the `deno_io` extension
    ///
    /// Requires the `io` feature to be enabled
    #[cfg(feature = "io")]
    #[cfg_attr(docsrs, doc(cfg(feature = "io")))]
    pub io_pipes: Option<deno_io::Stdio>,

    /// Optional path to the directory where the webstorage extension will store its data
    ///
    /// Requires the `webstorage` feature to be enabled
    #[cfg(feature = "webstorage")]
    #[cfg_attr(docsrs, doc(cfg(feature = "webstorage")))]
    pub webstorage_origin_storage_dir: Option<std::path::PathBuf>,

    /// Optional cache configuration for the `deno_cache` extension
    ///
    /// Requires the `cache` feature to be enabled
    #[cfg(feature = "cache")]
    #[cfg_attr(docsrs, doc(cfg(feature = "cache")))]
    pub cache: Option<deno_cache::CreateCache>,

    /// Filesystem implementation for the `deno_fs` extension
    ///
    /// Requires the `fs` feature to be enabled
    #[cfg(feature = "fs")]
    #[cfg_attr(docsrs, doc(cfg(feature = "fs")))]
    pub filesystem: deno_fs::FileSystemRc,

    /// Shared in-memory broadcast channel for the `deno_broadcast_channel` extension
    /// Also used by `WebWorker` to communicate with the main thread, if node is enabled
    ///
    /// Requires the `broadcast_channel` feature to be enabled
    #[cfg(feature = "broadcast_channel")]
    #[cfg_attr(docsrs, doc(cfg(feature = "broadcast_channel")))]
    pub broadcast_channel: deno_broadcast_channel::InMemoryBroadcastChannel,

    /// Key-value store for the `deno_kv` extension
    ///
    /// Requires the `kv` feature to be enabled
    #[cfg(feature = "kv")]
    #[cfg_attr(docsrs, doc(cfg(feature = "kv")))]
    pub kv_store: kv::KvStore,

    /// Package resolver for the `deno_node` extension
    /// `RustyResolver` allows you to select the base dir for modules
    /// as well as the filesystem implementation to use
    ///
    /// Requires the `node_experimental` feature to be enabled
    #[cfg(feature = "node_experimental")]
    #[cfg_attr(docsrs, doc(cfg(feature = "node_experimental")))]
    pub node_resolver: std::sync::Arc<node::resolvers::RustyResolver>,
}

impl Default for ExtensionOptions {
    fn default() -> Self {
        Self {
            #[cfg(feature = "web")]
            web: web::WebOptions::default(),

            #[cfg(feature = "crypto")]
            crypto_seed: None,

            #[cfg(feature = "io")]
            io_pipes: Some(deno_io::Stdio::default()),

            #[cfg(feature = "webstorage")]
            webstorage_origin_storage_dir: None,

            #[cfg(feature = "cache")]
            cache: None,

            #[cfg(feature = "fs")]
            filesystem: std::sync::Arc::new(deno_fs::RealFs),

            #[cfg(feature = "broadcast_channel")]
            broadcast_channel: deno_broadcast_channel::InMemoryBroadcastChannel::default(),

            #[cfg(feature = "kv")]
            kv_store: kv::KvStore::default(),

            #[cfg(feature = "node_experimental")]
            node_resolver: std::sync::Arc::new(node::resolvers::RustyResolver::default()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ext {
    BroadcastChannel,
    Cache,
    Console,
    Cron,
    Crypto,
    FFI,
    FS,
    HTTP,
    IO,
    KV,
    URL,
    Web,
    WebIDL,
    WebGPU,
    WebSocket,
    WebStorage,
    WebStub,
    // FS import
    // URL import
    // TLS,
    NodeExperimental,
}

impl Ext {
    pub fn safe_extensions() -> HashSet<Self> {
        HashSet::from([Self::WebStub, Self::Console, Self::URL, Self::Crypto])
    }

    pub fn network_extensions() -> HashSet<Self> {
        // URL import
        HashSet::from([
            Self::Web,
            Self::WebIDL,
            Self::WebStorage,
            Self::WebSocket,
            Self::HTTP,
            Self::URL,
            Self::Crypto,
            Self::Console,
            Self::BroadcastChannel,
        ])
    }

    pub fn io_extensions() -> HashSet<Self> {
        // FS import
        HashSet::from([
            Self::Web,
            Self::WebIDL,
            Self::WebStorage,
            Self::FS,
            Self::IO,
            Self::Cache,
            Self::Console,
            Self::FFI,
            Self::WebGPU,
            Self::KV,
            Self::Cron,
        ])
    }
}

macro_rules! ifdo {
    ($cond:expr, $code:expr) => {
        if $cond {
            $code
        }
    };
}

pub(crate) fn all_extensions(
    user_extensions: Vec<Extension>,
    deno_extensions: HashSet<Ext>,
    options: ExtensionOptions,
    shared_array_buffer_store: Option<CrossIsolateStore<SharedRef<BackingStore>>>,
    is_snapshot: bool,
) -> Vec<Extension> {
    let mut extensions = rustyscript::extensions(is_snapshot);

    #[cfg(feature = "webidl")]
    ifdo!(
        deno_extensions.contains(&Ext::WebIDL),
        extensions.extend(webidl::extensions(is_snapshot))
    );

    #[cfg(feature = "console")]
    ifdo!(
        deno_extensions.contains(&Ext::Console),
        extensions.extend(console::extensions(is_snapshot))
    );

    #[cfg(feature = "url")]
    ifdo!(
        deno_extensions.contains(&Ext::URL),
        extensions.extend(url::extensions(is_snapshot))
    );

    #[cfg(feature = "web")]
    ifdo!(
        deno_extensions.contains(&Ext::Web),
        extensions.extend(web::extensions(options.web.clone(), is_snapshot))
    );

    #[cfg(feature = "broadcast_channel")]
    ifdo!(
        deno_extensions.contains(&Ext::BroadcastChannel),
        extensions.extend(broadcast_channel::extensions(
            options.broadcast_channel.clone(),
            is_snapshot,
        ))
    );

    #[cfg(feature = "cache")]
    ifdo!(
        deno_extensions.contains(&Ext::Cache),
        extensions.extend(cache::extensions(options.cache.clone(), is_snapshot))
    );

    #[cfg(feature = "web_stub")]
    ifdo!(
        !deno_extensions.contains(&Ext::Web) && deno_extensions.contains(&Ext::WebStub),
        extensions.extend(web_stub::extensions(is_snapshot))
    );

    #[cfg(feature = "crypto")]
    ifdo!(
        deno_extensions.contains(&Ext::Crypto),
        extensions.extend(crypto::extensions(options.crypto_seed, is_snapshot))
    );

    #[cfg(feature = "io")]
    ifdo!(
        deno_extensions.contains(&Ext::IO),
        extensions.extend(io::extensions(options.io_pipes.clone(), is_snapshot))
    );

    #[cfg(feature = "webstorage")]
    ifdo!(
        deno_extensions.contains(&Ext::WebStorage),
        extensions.extend(webstorage::extensions(
            options.webstorage_origin_storage_dir.clone(),
            is_snapshot,
        ))
    );

    #[cfg(feature = "websocket")]
    ifdo!(
        deno_extensions.contains(&Ext::WebSocket),
        extensions.extend(websocket::extensions(options.web.clone(), is_snapshot))
    );

    #[cfg(feature = "fs")]
    ifdo!(
        deno_extensions.contains(&Ext::FS),
        extensions.extend(fs::extensions(options.filesystem.clone(), is_snapshot))
    );

    #[cfg(feature = "http")]
    ifdo!(
        deno_extensions.contains(&Ext::HTTP),
        extensions.extend(http::extensions((), is_snapshot))
    );

    #[cfg(feature = "ffi")]
    ifdo!(
        deno_extensions.contains(&Ext::FFI),
        extensions.extend(ffi::extensions(is_snapshot))
    );

    #[cfg(feature = "kv")]
    ifdo!(
        deno_extensions.contains(&Ext::KV),
        extensions.extend(kv::extensions(options.kv_store.clone(), is_snapshot))
    );

    #[cfg(feature = "webgpu")]
    ifdo!(
        deno_extensions.contains(&Ext::WebGPU),
        extensions.extend(webgpu::extensions(is_snapshot))
    );

    #[cfg(feature = "cron")]
    ifdo!(
        deno_extensions.contains(&Ext::Cron),
        extensions.extend(cron::extensions(is_snapshot))
    );

    #[cfg(feature = "node_experimental")]
    ifdo!(deno_extensions.contains(&Ext::NodeExperimental), {
        extensions.extend(napi::extensions(is_snapshot));
        extensions.extend(node::extensions(options.node_resolver.clone(), is_snapshot));
        extensions.extend(runtime::extensions(
            &options,
            shared_array_buffer_store,
            is_snapshot,
        ));
    });

    extensions.extend(user_extensions);
    extensions
}
