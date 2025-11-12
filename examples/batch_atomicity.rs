//! Batch Atomicity Example
//!
//! This example demonstrates the atomic nature of batch operations.
//! When you commit a batch, either ALL operations succeed or ALL fail together.
//! This is crucial for maintaining data consistency.
//!
//! Run this example with:
//! ```bash
//! cargo run --example batch_atomicity --features native
//! ```

use netabase_store::NetabaseStore;
use netabase_store::traits::store_ops::OpenTree;
use netabase_store::netabase_definition_module;
use netabase_store::traits::batch::{BatchBuilder, Batchable};

#[netabase_definition_module(BankDefinition, BankKeys)]
pub mod models {
    use netabase_store::{NetabaseModel, netabase};

    // Literally just illustrative, the store is NOT secure enough for sensitive, important use-cases like this
    #[derive(
        NetabaseModel,
        Clone,
        Debug,
        bincode::Encode,
        bincode::Decode,
        serde::Serialize,
        serde::Deserialize,
    )]
    #[netabase(BankDefinition)]
    pub struct Account {
        #[primary_key]
        pub account_id: u64,
        pub owner: String,
        pub balance: f64,
        #[secondary_key]
        pub account_type: String,
    }

    #[derive(
        NetabaseModel,
        Clone,
        Debug,
        bincode::Encode,
        bincode::Decode,
        serde::Serialize,
        serde::Deserialize,
    )]
    #[netabase(BankDefinition)]
    pub struct Transaction {
        #[primary_key]
        pub tx_id: String,
        pub from_account: u64,
        pub to_account: u64,
        pub amount: f64,
        pub timestamp: u64,
    }
}

use models::*;

fn main() -> anyhow::Result<()> {
    println!("=== Batch Atomicity Example ===\n");

    let store = NetabaseStore::<BankDefinition, _>::temp()?;
    let account_tree = store.open_tree::<Account>();
    let tx_tree = store.open_tree::<Transaction>();

    // Example 1: Successful Atomic Transfer
    println!("Example 1: Successful Atomic Transfer");
    println!("--------------------------------------");
    setup_accounts(&account_tree)?;
    atomic_transfer(&account_tree, &tx_tree, 1, 2, 100.0)?;

    // Example 2: Multiple Operations Must All Succeed
    println!("\nExample 2: Multiple Account Updates");
    println!("------------------------------------");
    batch_update_multiple_accounts(&account_tree)?;

    // Example 3: Conditional Batch Operations
    println!("\nExample 3: Conditional Batch Processing");
    println!("----------------------------------------");
    conditional_batch(&account_tree)?;

    println!("\n✅ All atomicity examples completed successfully!");

    Ok(())
}

/// Set up initial accounts
fn setup_accounts(
    account_tree: &netabase_store::databases::sled_store::SledStoreTree<'_, BankDefinition, Account>,
) -> anyhow::Result<()> {
    let mut batch = account_tree.create_batch()?;

    let accounts = vec![
        Account {
            account_id: 1,
            owner: "Alice".to_string(),
            balance: 1000.0,
            account_type: "Checking".to_string(),
        },
        Account {
            account_id: 2,
            owner: "Bob".to_string(),
            balance: 500.0,
            account_type: "Checking".to_string(),
        },
        Account {
            account_id: 3,
            owner: "Charlie".to_string(),
            balance: 2000.0,
            account_type: "Savings".to_string(),
        },
    ];

    for account in accounts {
        batch.put(account)?;
    }

    batch.commit()?;
    println!("✓ Created 3 accounts");

    Ok(())
}

/// Perform an atomic transfer between two accounts
/// This demonstrates why atomicity is crucial: we need BOTH operations to succeed
fn atomic_transfer(
    account_tree: &netabase_store::databases::sled_store::SledStoreTree<'_, BankDefinition, Account>,
    tx_tree: &netabase_store::databases::sled_store::SledStoreTree<'_, BankDefinition, Transaction>,
    from_id: u64,
    to_id: u64,
    amount: f64,
) -> anyhow::Result<()> {
    println!(
        "Transferring ${:.2} from account {} to account {}",
        amount, from_id, to_id
    );

    // Get current balances
    let from_account = account_tree
        .get(AccountPrimaryKey(from_id))?
        .ok_or_else(|| anyhow::anyhow!("Source account not found"))?;
    let to_account = account_tree
        .get(AccountPrimaryKey(to_id))?
        .ok_or_else(|| anyhow::anyhow!("Destination account not found"))?;

    println!(
        "  Before: {} has ${:.2}, {} has ${:.2}",
        from_account.owner, from_account.balance, to_account.owner, to_account.balance
    );

    // Validate
    if from_account.balance < amount {
        anyhow::bail!("Insufficient funds");
    }

    // Create batch for atomic transfer
    let mut account_batch = account_tree.create_batch()?;

    // Deduct from source
    let mut updated_from = from_account.clone();
    updated_from.balance -= amount;
    account_batch.put(updated_from)?;

    // Add to destination
    let mut updated_to = to_account.clone();
    updated_to.balance += amount;
    account_batch.put(updated_to)?;

    // Commit atomically - both operations succeed or both fail
    account_batch.commit()?;

    // Record the transaction
    let tx = Transaction {
        tx_id: format!(
            "TX-{}-{}-{}",
            from_id,
            to_id,
            chrono::Utc::now().timestamp()
        ),
        from_account: from_id,
        to_account: to_id,
        amount,
        timestamp: chrono::Utc::now().timestamp() as u64,
    };
    tx_tree.put(tx)?;

    // Verify
    let from_after = account_tree.get(AccountPrimaryKey(from_id))?.unwrap();
    let to_after = account_tree.get(AccountPrimaryKey(to_id))?.unwrap();

    println!(
        "  After:  {} has ${:.2}, {} has ${:.2}",
        from_after.owner, from_after.balance, to_after.owner, to_after.balance
    );
    println!("✓ Transfer completed atomically");

    Ok(())
}

/// Batch update multiple accounts - all updates happen atomically
fn batch_update_multiple_accounts(
    account_tree: &netabase_store::databases::sled_store::SledStoreTree<'_, BankDefinition, Account>,
) -> anyhow::Result<()> {
    println!("Applying interest to all savings accounts...");

    // Get all savings accounts
    let savings_accounts = account_tree.get_by_secondary_key(AccountSecondaryKeys::AccountType(
        AccountAccountTypeSecondaryKey("Savings".to_string()),
    ))?;

    println!("Found {} savings accounts", savings_accounts.len());

    // Create batch to update all of them
    let mut batch = account_tree.create_batch()?;

    for mut account in savings_accounts.clone() {
        let interest = account.balance * 0.05; // 5% interest
        account.balance += interest;
        println!("  {} +${:.2} interest", account.owner, interest);
        batch.put(account)?;
    }

    // All updates happen atomically
    batch.commit()?;

    println!("✓ Applied interest to all savings accounts atomically");

    // Verify
    for account in savings_accounts {
        let updated = account_tree
            .get(AccountPrimaryKey(account.account_id))?
            .unwrap();
        println!("  {} now has ${:.2}", updated.owner, updated.balance);
    }

    Ok(())
}

/// Conditional batch processing: only commit if certain conditions are met
fn conditional_batch(
    account_tree: &netabase_store::databases::sled_store::SledStoreTree<'_, BankDefinition, Account>,
) -> anyhow::Result<()> {
    println!("Processing fee deductions (only if all accounts have sufficient balance)...");

    const FEE: f64 = 50.0;

    // Get all checking accounts
    let checking_accounts = account_tree.get_by_secondary_key(
        AccountSecondaryKeys::AccountType(AccountAccountTypeSecondaryKey("Checking".to_string())),
    )?;

    // First, validate ALL accounts can afford the fee
    let mut can_process = true;
    for account in &checking_accounts {
        if account.balance < FEE {
            println!(
                "  ⚠️  {} only has ${:.2}, cannot deduct fee",
                account.owner, account.balance
            );
            can_process = false;
        }
    }

    if !can_process {
        println!("✗ Skipping fee deduction - not all accounts have sufficient balance");
        println!("  (This demonstrates validation before batch operations)");
        return Ok(());
    }

    // All accounts can afford it, proceed with batch
    let mut batch = account_tree.create_batch()?;

    for mut account in checking_accounts {
        println!("  Deducting ${:.2} from {}", FEE, account.owner);
        account.balance -= FEE;
        batch.put(account)?;
    }

    // Commit all deductions atomically
    batch.commit()?;

    println!("✓ Deducted fees from all checking accounts atomically");

    Ok(())
}
