use super::*;
use crate::contracts::tep::metadata::metadata_content::MetadataDict;
use crate::contracts::tep::snake_data::SnakeData;
use crate::errors::TonError;

#[derive(Debug, PartialEq, Eq)]
struct TestMetadata {
    external_loaded: bool,
}

impl Metadata for TestMetadata {
    fn from_data(_dict: &MetadataDict, json: Option<&str>) -> Result<Self, TonError> {
        Ok(Self {
            external_loaded: json.is_some(),
        })
    }
}

#[tokio::test]
async fn test_semichain_external_metadata_required() -> anyhow::Result<()> {
    let content = MetadataContent::Internal(MetadataInternal {
        data: MetadataDict::from([(**META_URI, SnakeData::from("unsupported://metadata").into())]),
    });

    let fallback_loader = MetaLoader::builder().build()?;
    let fallback_metadata: TestMetadata = fallback_loader.load(&content).await?;
    assert_eq!(fallback_metadata, TestMetadata { external_loaded: false });

    let required_loader = MetaLoader::builder().with_semichain_external_metadata_required(true).build()?;
    let required_result = required_loader.load::<TestMetadata>(&content).await;
    assert!(matches!(required_result, Err(TonError::TransportError(_))));

    Ok(())
}
