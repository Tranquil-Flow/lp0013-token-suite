//! Placeholder crate for the LP-0013 token authorities workspace.
//!
//! Implementation will land via strict TDD in follow-up commits.

/// Returns the crate name for scaffold smoke checks.
pub fn crate_name() -> &'static str {
    env!("CARGO_PKG_NAME")
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    #[test]
    fn exposes_crate_name() {
        assert!(!super::crate_name().is_empty());
    }

    #[test]
    fn fallback_idl_documents_token_authority_surface() {
        let idl: Value = serde_json::from_str(include_str!("../../idl/admin-authority.idl.json"))
            .expect("valid JSON IDL");

        assert_eq!(idl["name"], "admin_authority");
        assert_eq!(idl["metadata"]["generation"], "hand-written");
        assert_eq!(
            idl["metadata"]["tooling_status"],
            "spel-available-generated-idl-shipped-alongside"
        );

        let instructions = idl["instructions"].as_array().expect("instructions array");
        let instruction_names: Vec<&str> = instructions
            .iter()
            .map(|instruction| instruction["name"].as_str().expect("instruction name"))
            .collect();
        assert_eq!(
            instruction_names,
            vec![
                "create_mint",
                "create_holding",
                "mint_to",
                "set_mint_authority"
            ]
        );

        let accounts = idl["accounts"].as_array().expect("accounts array");
        let account_names: Vec<&str> = accounts
            .iter()
            .map(|account| account["name"].as_str().expect("account name"))
            .collect();
        assert!(account_names.contains(&"AuthorityInfo"));
        assert!(account_names.contains(&"MintDefinition"));
        assert!(account_names.contains(&"TokenHolding"));
    }
}
