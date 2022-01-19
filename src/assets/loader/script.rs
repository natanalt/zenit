use crate::{
    assets::{
        munge::{MungeName, MungeTreeNode},
        script::CompiledScript,
    },
    AnyResult,
};
use anyhow::anyhow;
use bevy::{
    asset::{AssetLoader, BoxedFuture, LoadContext, LoadedAsset},
    prelude::{AddAsset, App, Plugin},
};
use std::io::Cursor;

#[derive(Debug, Clone, Copy, Default)]
pub struct CompiledScriptLoaderPlugin;

impl Plugin for CompiledScriptLoaderPlugin {
    fn build(&self, app: &mut App) {
        app.add_asset::<CompiledScript>()
            .init_asset_loader::<CompiledScriptLoader>();
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct CompiledScriptLoader;

impl AssetLoader for CompiledScriptLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, AnyResult<()>> {
        Box::pin(async move {
            let mut cursor = Cursor::new(bytes);
            let root = MungeTreeNode::parse(&mut cursor, None)?;

            let name = root
                .find(MungeName::from_literal("NAME"))
                .ok_or(anyhow!("Invalid script node"))?
                .node
                .read_string(&mut cursor)?;

            let mysterious_info = root
                .find(MungeName::from_literal("INFO"))
                .ok_or(anyhow!("Invalid texture node"))?
                .node
                .read_contents(&mut cursor)?[0];

            let data = root
                .find(MungeName::from_literal("BODY"))
                .ok_or(anyhow!("Invalid texture node"))?
                .node
                .read_contents(&mut cursor)?;

            load_context.set_default_asset(LoadedAsset::new(CompiledScript {
                name,
                mysterious_info,
                data,
            }));

            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["luac"]
    }
}
