use std::marker::PhantomData;

use bevy::{app::Plugin, asset::{Asset, AssetLoader, AsyncReadExt, AssetApp}};
use serde_json::from_slice;

pub struct JsonAssetPlugin<A> {
    extensions : Vec<&'static str>,
    _marker: PhantomData<A>,
}

impl<A> Plugin for JsonAssetPlugin<A> 
where
    for<'de> A:serde::Deserialize<'de> + Asset
{

    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_asset::<A>()
            .register_asset_loader(JsonAssetLoader::<A>{
                extensions: self.extensions.clone(),
                _marker: PhantomData,
            });
    }
}

impl<A> JsonAssetPlugin<A> 
where
    for<'de> A:serde::Deserialize<'de> + Asset
{
    pub fn new(extensions: &[&'static str]) -> Self {
        Self {
            extensions: extensions.to_owned(),
            _marker: PhantomData,
        }
    }
}

struct JsonAssetLoader<A> {
    extensions: Vec<&'static str>,
    _marker: PhantomData<A>,
}

// #[non_exhaustive]
// #[derive(Debug, Error)]
// pub enum JsonLoaderError {
//     /// An [IO Error](std::io::Error)
//     #[error("Could not read the file: {0}")]
//     Io(#[from] std::io::Error),
//     /// A [JSON Error](serde_json::error::Error)
//     #[error("Could not parse the JSON: {0}")]
//     JsonError(#[from] serde_json::error::Error),
// }

impl<A> AssetLoader for JsonAssetLoader<A>  
where
    for<'de> A: serde::Deserialize<'de> + Asset
{
    type Asset = A;

    type Settings = ();

    type Error = std::io::Error;

    fn load<'a>(
        &'a self,
        reader: &'a mut bevy::asset::io::Reader,
        settings: &'a Self::Settings,
        load_context: &'a mut bevy::asset::LoadContext,
    ) -> bevy::utils::BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;
            let asset = from_slice::<A>(&bytes)?;
            Ok(asset)
        })
    }

    fn extensions(&self) -> &[&str] {
        &self.extensions
    }
}