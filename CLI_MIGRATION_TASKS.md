# CLI and Migration Implementation Tasks

## Progress Summary

### ✅ Completed
1. Created tracking document for task management
2. Fixed CLI database path handling:
   - Changed `db_path` from String to Option<String>
   - CLI now uses parent directory of binary as default database path
   - Allows explicit override with --db-path flag
3. Updated both `generate_store_cli` and `generate_single_definition_cli`

### 🔄 In Progress  
1. **CLI Generation System**
   - Issue: Existing client binaries use old compiled code
   - Need to create proper client binary generation workflow
   - Problem: `generate_cli!` macro creates new types instead of using existing definitions
   
2. **Documentation/Doctest Fixes**
   - 76 failing doctests identified
   - Many use overlapping paths causing conflicts
   - Need to ensure all examples use unique paths or in-memory databases

### 📋 TODO
1. **Complete CLI Implementation**
   - [ ] Add RON format support for all CRUD operations (JSON already done)
   - [ ] Implement Read command (currently placeholder)
   - [ ] Implement Delete command (currently placeholder)
   - [ ] Implement Query by secondary keys
   - [ ] Implement full Schema commands (show, export, tables, stats)
   - [ ] Test all CLI endpoints thoroughly
   - [ ] Generate and test Nushell test script

2. **Fix Migration System**
   - [ ] Implement family enum fallback for version detection
   - [ ] Add better error handling for schema mismatches
   - [ ] Ensure atomicity when database structure changes
   - [ ] Add migration validation and rollback capabilities
   - [ ] Document migration workflows

3. **Update Generated README**
   - [ ] Ensure README matches actual CLI capabilities
   - [ ] Add examples for RON format usage
   - [ ] Document all available commands
   - [ ] Add troubleshooting section

4. **Testing**
   - [ ] Create comprehensive integration tests
   - [ ] Test CLI with actual databases
   - [ ] Verify schema verification works
   - [ ] Test migration scenarios

## Known Issues

1. **Client Binary Path**: The my_database/client binary is from old build
   - Solution: Need to rebuild from source that creates it
   - OR: Delete and regenerate the example database

2. **generate_cli! Macro**: Creates new types instead of reusing existing
   - Not suitable for creating client for already-defined types
   - Works for standalone schema.toml files

3. **Schema Consistency**: Examples overwrite each other's databases
   - Need unique paths for each example/doctest
   - OR: Use in-memory databases for tests

## Next Steps

1. Focus on migration system implementation (per user request)
2. Fix CLI CRUD operations (Read, Delete, Query)
3. Add comprehensive testing
4. Fix remaining doctests

