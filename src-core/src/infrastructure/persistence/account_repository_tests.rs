use std::sync::Arc;

use tempfile::tempdir;

use crate::infrastructure::persistence::account_repository::{
    AccountImportRecord, AccountRepository,
};
use crate::infrastructure::persistence::database::AppDatabase;
use crate::infrastructure::persistence::runtime_paths::RuntimePaths;
use crate::infrastructure::security::credential_vault::{CredentialVault, MemoryCredentialVault};

#[test]
fn stores_accounts_without_plaintext_passwords() {
    let root = tempdir().expect("create temp dir");
    let paths = RuntimePaths::from_cwd_for_tests(root.path()).expect("create paths");
    let db = AppDatabase::open(&paths).expect("open db");
    let vault: Arc<dyn CredentialVault> = Arc::new(MemoryCredentialVault::default());
    let repo = AccountRepository::new(db, vault);

    let account = repo
        .add_account("主号", "20260001", "secret-1")
        .expect("add account");
    let store = repo.load_store().expect("load store");

    assert_eq!(store.accounts, vec![account.clone()]);
    assert_eq!(store.selected_account_id, account.id);
    assert_eq!(
        repo.load_account_with_password(&account)
            .expect("load credential")
            .password,
        "secret-1"
    );
}

#[test]
fn imports_accounts_and_overwrites_existing_usernames() {
    let root = tempdir().expect("create temp dir");
    let paths = RuntimePaths::from_cwd_for_tests(root.path()).expect("create paths");
    let db = AppDatabase::open(&paths).expect("open db");
    let vault: Arc<dyn CredentialVault> = Arc::new(MemoryCredentialVault::default());
    let repo = AccountRepository::new(db, vault);
    let existing = repo
        .add_account("旧号", "20260001", "old-secret")
        .expect("add account");

    let stats = repo
        .import_accounts(vec![
            AccountImportRecord {
                remark_name: "重复号".to_string(),
                username: "20260001".to_string(),
                password: "new-secret".to_string(),
            },
            AccountImportRecord {
                remark_name: "新号".to_string(),
                username: "20260002".to_string(),
                password: "secret-2".to_string(),
            },
        ])
        .expect("import accounts");

    assert_eq!(stats.imported_count, 1);
    assert_eq!(stats.overwritten_count, 1);
    let store = repo.load_store().expect("load store");
    assert_eq!(store.accounts.len(), 2);
    assert_eq!(store.accounts[0].remark_name, "重复号");
    assert_eq!(
        repo.load_account_with_password(&existing)
            .expect("load existing")
            .password,
        "new-secret"
    );
}

#[test]
fn import_can_overwrite_existing_passwords() {
    let root = tempdir().expect("create temp dir");
    let paths = RuntimePaths::from_cwd_for_tests(root.path()).expect("create paths");
    let db = AppDatabase::open(&paths).expect("open db");
    let vault: Arc<dyn CredentialVault> = Arc::new(MemoryCredentialVault::default());
    let repo = AccountRepository::new(db, vault);
    let existing = repo
        .add_account("旧号", "20260001", "old-secret")
        .expect("add account");

    let stats = repo
        .import_accounts(vec![AccountImportRecord {
            remark_name: "新备注".to_string(),
            username: "20260001".to_string(),
            password: "new-secret".to_string(),
        }])
        .expect("import accounts");

    assert_eq!(stats.imported_count, 0);
    assert_eq!(stats.overwritten_count, 1);
    let store = repo.load_store().expect("load store");
    assert_eq!(store.accounts.len(), 1);
    assert_eq!(store.accounts[0].remark_name, "新备注");
    assert_eq!(
        repo.load_account_with_password(&existing)
            .expect("load existing")
            .password,
        "new-secret"
    );
}
