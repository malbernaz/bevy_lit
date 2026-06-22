use bevy::{asset::AsAssetId, prelude::*, render::extract_component::ExtractComponent};

/// A light occluder component. Should be used alongside a Mesh2d
#[derive(Component, Clone, Debug, Default, Reflect, ExtractComponent, FromTemplate)]
pub struct LightOccluder2d {
    /// Any texture with a transparent background. The occluder will take it's shape.
    pub occluder_mask: Handle<Image>,
}

impl LightOccluder2d {
    /// Creates a new [`LightOccluder2d`] with an occlusion mask
    pub fn new(occluder_mask: Handle<Image>) -> Self {
        Self { occluder_mask }
    }
}

impl From<LightOccluder2d> for AssetId<Image> {
    fn from(material: LightOccluder2d) -> Self {
        material.occluder_mask.id()
    }
}

impl From<&LightOccluder2d> for AssetId<Image> {
    fn from(material: &LightOccluder2d) -> Self {
        material.occluder_mask.id()
    }
}

impl AsAssetId for LightOccluder2d {
    type Asset = Image;

    fn as_asset_id(&self) -> AssetId<Self::Asset> {
        self.occluder_mask.id()
    }
}
