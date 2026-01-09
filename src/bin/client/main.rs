pub mod generated;

netabase_macros::generate_cli!("src/bin/tmp/dummy_db/schema.toml");

fn main() {
    use clap::Parser;

    let cli = SimpleDefinitionCli::parse();

    println!("Database path: {}", cli.db_path);
    println!("Command: {:?}", cli.command);

    // TODO: Implement command handlers
    match cli.command {
        SimpleDefinitionCommands::InventoryItem(cmd) => {
            println!("InventoryItem command: {:?}", cmd);
        }
        SimpleDefinitionCommands::BranchInventory(cmd) => {
            println!("BranchInventory command: {:?}", cmd);
        }
    }
}
