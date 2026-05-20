// Generate IDL JSON for the admin_authority_spike program.
// Usage: cargo run --bin generate_idl > admin_authority_spike-idl.json
spel_framework::generate_idl!("../methods/guest/src/bin/admin_authority_spike.rs");
