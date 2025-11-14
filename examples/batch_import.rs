//! Batch Import Example
//!
//! This example demonstrates a common real-world use case: importing large amounts
//! of data from an external source (like a CSV file or API) into your database.
//!
//! This shows how batch operations make bulk imports practical and performant.
//!
//! Run this example with:
//! ```bash
//! cargo run --example batch_import --features native
//! ```

use netabase_store::NetabaseStore;
use netabase_store::netabase_definition_module;
use netabase_store::traits::batch::{BatchBuilder, Batchable};
use netabase_store::traits::store_ops::OpenTree;
use std::time::Instant;

#[netabase_definition_module(CatalogDefinition, CatalogKeys)]
pub mod models {
    use netabase_store::{NetabaseModel, netabase};

    #[derive(
        NetabaseModel,
        Clone,
        Debug,
        bincode::Encode,
        bincode::Decode,
        serde::Serialize,
        serde::Deserialize,
    )]
    #[netabase(CatalogDefinition)]
    pub struct Book {
        #[primary_key]
        pub isbn: String,
        pub title: String,
        pub author: String,
        pub year: u32,
        #[secondary_key]
        pub genre: String,
        pub pages: u32,
        pub available: bool,
    }
}

use models::*;

fn main() -> anyhow::Result<()> {
    println!("=== Batch Import Example ===\n");

    // Create a temporary store
    let store = NetabaseStore::<CatalogDefinition, _>::temp()?;
    let book_tree = store.open_tree::<Book>();

    // Simulate importing from an external data source
    println!("Importing book catalog...");

    let books_data: Vec<Book> = generate_sample_books();
    println!("Generated {} sample books", books_data.len());

    // Import using batch operations
    let start = Instant::now();
    import_books(&book_tree, books_data)?;
    let duration = start.elapsed();

    println!("✓ Import completed in {:?}", duration);

    // Verify the import
    let total_books = book_tree.len();
    println!("✓ Total books in database: {}", total_books);

    // Query some statistics
    print_statistics(&book_tree)?;

    // Demonstrate batch updates
    println!("\nUpdating availability status...");
    batch_update_availability(&book_tree)?;

    println!("\n✅ Batch import example completed successfully!");

    Ok(())
}

/// Import books using batch operations
fn import_books(
    book_tree: &netabase_store::databases::sled_store::SledStoreTree<'_, CatalogDefinition, Book>,
    books: Vec<Book>,
) -> anyhow::Result<()> {
    const BATCH_SIZE: usize = 100;

    let mut total_imported = 0;

    // Process in batches to avoid memory issues with very large imports
    for chunk in books.chunks(BATCH_SIZE) {
        let mut batch = book_tree.create_batch()?;

        for book in chunk {
            batch.put(book.clone())?;
        }

        batch.commit()?;
        total_imported += chunk.len();

        // Progress indicator
        if total_imported % 500 == 0 {
            println!("  Imported {} books...", total_imported);
        }
    }

    Ok(())
}

/// Generate sample book data (simulates reading from CSV/API)
fn generate_sample_books() -> Vec<Book> {
    let genres = vec!["Fiction", "Science", "History", "Biography", "Fantasy"];
    let mut books = Vec::new();

    for i in 0..2000 {
        let book = Book {
            isbn: format!("978-0-{:06}-{:02}-{}", i / 100, i % 100, (i * 7) % 10),
            title: format!("Book Title {}", i),
            author: format!("Author {}", i % 100),
            year: 1950 + (i % 73) as u32,
            genre: genres[i % genres.len()].to_string(),
            pages: 100 + (i % 500) as u32,
            available: i % 3 != 0, // About 2/3 available
        };
        books.push(book);
    }

    books
}

/// Print statistics about the imported data
fn print_statistics(
    book_tree: &netabase_store::databases::sled_store::SledStoreTree<'_, CatalogDefinition, Book>,
) -> anyhow::Result<()> {
    println!("\n--- Catalog Statistics ---");

    let genres = vec!["Fiction", "Science", "History", "Biography", "Fantasy"];

    for genre in genres {
        let books = book_tree.get_by_secondary_key(BookSecondaryKeys::Genre(
            BookGenreSecondaryKey(genre.to_string()),
        ))?;
        println!("  {}: {} books", genre, books.len());
    }

    Ok(())
}

/// Demonstrate batch updates: mark all Fiction books as unavailable
fn batch_update_availability(
    book_tree: &netabase_store::databases::sled_store::SledStoreTree<'_, CatalogDefinition, Book>,
) -> anyhow::Result<()> {
    // Get all Fiction books
    let fiction_books = book_tree.get_by_secondary_key(BookSecondaryKeys::Genre(
        BookGenreSecondaryKey("Fiction".to_string()),
    ))?;

    println!("Updating {} Fiction books...", fiction_books.len());

    // Create a batch to update all of them
    let mut batch = book_tree.create_batch()?;

    for mut book in fiction_books {
        book.available = false; // Mark as unavailable
        batch.put(book)?;
    }

    batch.commit()?;

    println!("✓ Updated all Fiction books atomically");

    // Verify: count how many Fiction books are now unavailable
    let fiction_books = book_tree.get_by_secondary_key(BookSecondaryKeys::Genre(
        BookGenreSecondaryKey("Fiction".to_string()),
    ))?;
    let unavailable = fiction_books.iter().filter(|b| !b.available).count();
    println!(
        "✓ Verification: {}/{} Fiction books are now unavailable",
        unavailable,
        fiction_books.len()
    );

    Ok(())
}
