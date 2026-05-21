use nexara_secrets::{AsyncSecretStore, EncryptedFileSecretStore, SecretStore, SecretValue};

#[test]
fn encrypted_store_roundtrips_without_plaintext_on_disk() {
    let dir = tempfile::tempdir().unwrap();
    let store =
        EncryptedFileSecretStore::new(dir.path().join("secrets.json"), dir.path().join("key.bin"));
    SecretStore::put(&store, "token", SecretValue::new("super-secret")).unwrap();

    assert_eq!(
        SecretStore::get(&store, "token").unwrap().unwrap().expose(),
        "super-secret"
    );
    let on_disk = std::fs::read_to_string(dir.path().join("secrets.json")).unwrap();
    assert!(!on_disk.contains("super-secret"));
}

#[tokio::test]
async fn encrypted_store_async_wrapper_roundtrips() {
    let dir = tempfile::tempdir().unwrap();
    let store =
        EncryptedFileSecretStore::new(dir.path().join("secrets.json"), dir.path().join("key.bin"));
    AsyncSecretStore::put(&store, "token", SecretValue::new("super-secret"))
        .await
        .unwrap();

    assert_eq!(
        AsyncSecretStore::get(&store, "token")
            .await
            .unwrap()
            .unwrap()
            .expose(),
        "super-secret"
    );
}
