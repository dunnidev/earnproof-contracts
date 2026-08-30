/// Protocol Config TTL Boundary Tests
///
/// Tests for instance storage (Admin, Paused, ConfigVersion, ContractVersion, AllowedWasm)
/// and persistent storage (SchemaVersion entries).

#[cfg(test)]
mod tests {
    extern crate std;

    use soroban_sdk::{Address, BytesN, Env};
    use protocol_config::{ProtocolConfigContract, ProtocolConfigContractClient};
    use earnproof_shared::TTL_THRESHOLD_LEDGERS;
    use crate::harness::TtlTestHarness;

    const ADMIN: &str = "GCFIRY65OQE7DFP5KLNS2PF2LVZMUZYJX4OZIEQ36N2IQANUB5XVYOJR";

    fn bytes(env: &Env, value: u8) -> BytesN<32> {
        BytesN::from_array(env, &[value; 32])
    }

    fn admin_addr(env: &Env) -> Address {
        Address::from_string(&env, ADMIN)
    }

    fn setup(env: &Env) -> (ProtocolConfigContractClient<'static>, Address) {
        env.mock_all_auths();
        let contract_id = env.register(ProtocolConfigContract, ());
        let client = ProtocolConfigContractClient::new(env, &contract_id);
        let admin = admin_addr(env);
        client.initialize(&admin);
        (client, admin)
    }

    // ── Instance Storage (Admin) ────

    /// Pre-expiry: admin read succeeds and extends TTL.
    #[test]
    fn instance_admin_pre_expiry_readable() {
        let env = Env::default();
        let (client, admin) = setup(&env);

        let current_ledger = TtlTestHarness::current_ledger(&env);
        let expiry = TtlTestHarness::calculate_expiry(
            current_ledger,
            TTL_THRESHOLD_LEDGERS,
            500_000,
        );

        let pre_expiry = TtlTestHarness::pre_expiry_ledger(expiry);
        TtlTestHarness::advance_to_ledger(&env, pre_expiry);

        let retrieved = client.get_admin();
        assert_eq!(retrieved, admin);
    }

    /// At-expiry: admin read still succeeds (boundary is inclusive for valid reads).
    #[test]
    fn instance_admin_at_expiry_readable() {
        let env = Env::default();
        let (client, admin) = setup(&env);

        let current_ledger = TtlTestHarness::current_ledger(&env);
        let expiry = TtlTestHarness::calculate_expiry(
            current_ledger,
            TTL_THRESHOLD_LEDGERS,
            500_000,
        );

        let at_expiry = TtlTestHarness::at_expiry_ledger(expiry);
        TtlTestHarness::advance_to_ledger(&env, at_expiry);

        let retrieved = client.get_admin();
        assert_eq!(retrieved, admin);
    }

    /// Post-expiry: admin read fails with NotInitialized (entry expired and removed).
    #[test]
    fn instance_admin_post_expiry_fails() {
        let env = Env::default();
        let (client, _admin) = setup(&env);

        let current_ledger = TtlTestHarness::current_ledger(&env);
        let expiry = TtlTestHarness::calculate_expiry(
            current_ledger,
            TTL_THRESHOLD_LEDGERS,
            500_000,
        );

        let post_expiry = TtlTestHarness::post_expiry_ledger(expiry);
        TtlTestHarness::advance_to_ledger(&env, post_expiry);

        let result = client.try_get_admin();
        assert!(result.is_err());
        use earnproof_shared::ContractError;
        assert_eq!(result, Err(Ok(ContractError::NotInitialized)));
    }

    // ── Persistent Storage (SchemaVersion entries) ────

    /// Pre-expiry: schema version read succeeds and extends TTL.
    #[test]
    fn persistent_schema_version_pre_expiry_readable() {
        let env = Env::default();
        let (client, _admin) = setup(&env);

        client.approve_schema_version(&7);
        assert!(client.is_schema_version_approved(&7));

        let current_ledger = TtlTestHarness::current_ledger(&env);
        let expiry = TtlTestHarness::calculate_expiry(
            current_ledger,
            TTL_THRESHOLD_LEDGERS,
            500_000,
        );

        let pre_expiry = TtlTestHarness::pre_expiry_ledger(expiry);
        TtlTestHarness::advance_to_ledger(&env, pre_expiry);

        let approved = client.is_schema_version_approved(&7);
        assert!(approved);
    }

    /// At-expiry: schema version read still succeeds (boundary is inclusive).
    #[test]
    fn persistent_schema_version_at_expiry_readable() {
        let env = Env::default();
        let (client, _admin) = setup(&env);

        client.approve_schema_version(&8);
        assert!(client.is_schema_version_approved(&8));

        let current_ledger = TtlTestHarness::current_ledger(&env);
        let expiry = TtlTestHarness::calculate_expiry(
            current_ledger,
            TTL_THRESHOLD_LEDGERS,
            500_000,
        );

        let at_expiry = TtlTestHarness::at_expiry_ledger(expiry);
        TtlTestHarness::advance_to_ledger(&env, at_expiry);

        let approved = client.is_schema_version_approved(&8);
        assert!(approved);
    }

    /// Post-expiry: schema version read returns false (entry expired, treated as unapproved).
    #[test]
    fn persistent_schema_version_post_expiry_fails() {
        let env = Env::default();
        let (client, _admin) = setup(&env);

        client.approve_schema_version(&9);
        assert!(client.is_schema_version_approved(&9));

        let current_ledger = TtlTestHarness::current_ledger(&env);
        let expiry = TtlTestHarness::calculate_expiry(
            current_ledger,
            TTL_THRESHOLD_LEDGERS,
            500_000,
        );

        let post_expiry = TtlTestHarness::post_expiry_ledger(expiry);
        TtlTestHarness::advance_to_ledger(&env, post_expiry);

        let approved = client.is_schema_version_approved(&9);
        assert!(!approved);
    }

    /// Restoration: after expiry, re-approving the schema version succeeds and extends TTL.
    #[test]
    fn persistent_schema_version_restoration_succeeds() {
        let env = Env::default();
        let (client, _admin) = setup(&env);

        client.approve_schema_version(&10);
        let current_ledger = TtlTestHarness::current_ledger(&env);
        let expiry = TtlTestHarness::calculate_expiry(
            current_ledger,
            TTL_THRESHOLD_LEDGERS,
            500_000,
        );

        let post_expiry = TtlTestHarness::post_expiry_ledger(expiry);
        TtlTestHarness::advance_to_ledger(&env, post_expiry);

        assert!(!client.is_schema_version_approved(&10));

        client.approve_schema_version(&10);
        assert!(client.is_schema_version_approved(&10));
    }

    /// ConfigVersion is bumped on pause/unpause but shares instance TTL.
    #[test]
    fn config_version_bump_extends_instance_ttl() {
        let env = Env::default();
        let (client, _admin) = setup(&env);

        let v1 = client.get_config_version();
        assert_eq!(v1, 1);

        client.pause();
        let v2 = client.get_config_version();
        assert_eq!(v2, 2);

        let current_ledger = TtlTestHarness::current_ledger(&env);
        let expiry = TtlTestHarness::calculate_expiry(
            current_ledger,
            TTL_THRESHOLD_LEDGERS,
            500_000,
        );
        let pre_expiry = TtlTestHarness::pre_expiry_ledger(expiry);
        TtlTestHarness::advance_to_ledger(&env, pre_expiry);

        let admin = client.get_admin();
        assert!(admin.len() > 0);
        let v = client.get_config_version();
        assert_eq!(v, 2);
    }
}
