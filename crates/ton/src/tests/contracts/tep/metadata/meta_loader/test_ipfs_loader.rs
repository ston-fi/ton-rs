use super::join_url_path;

#[test]
fn test_join_url_path_normalizes_separator() {
    assert_eq!(join_url_path("https://example.com/ipfs", "cid/file.json"), "https://example.com/ipfs/cid/file.json");
    assert_eq!(join_url_path("https://example.com/ipfs/", "/cid/file.json"), "https://example.com/ipfs/cid/file.json");
}
