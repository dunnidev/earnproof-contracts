#![no_std]

#[cfg(test)]
extern crate std;

#[cfg(test)]
mod tests {
    use earnproof_shared::{ContractError, IssuerError, ProofError};
    use issuer_registry::{IssuerRegistryContract, IssuerRegistryContractClient};
    use protocol_config::{ProtocolConfigContract, ProtocolConfigContractClient};
    use proof_registry::{ProofRegistryContract, ProofRegistryContractClient};
    use soroban_sdk::{testutils::Address as _, Address, BytesN, Env};

    const ZERO_ADDR: &str = "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";

    fn bytes(env: &Env, value: u8) -> BytesN<32> {
        BytesN::from_array(env, &[value; 32])
    }

    #[test]
    fn protocol_config_rejects_zero_and_sentinel_admins() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(ProtocolConfigContract, ());
        let client = ProtocolConfigContractClient::new(&env, &contract_id);
        let zero = Address::from_str(&env, ZERO_ADDR);

        let init = client.try_initialize(&zero);
        assert_eq!(init, Err(Ok(ContractError::InvalidInput)));

        client.initialize(&Address::generate(&env));
        let replacement = Address::from_str(&env, ZERO_ADDR);
        let result = client.try_set_admin(&replacement);
        assert_eq!(result, Err(Ok(ContractError::InvalidInput)));
    }

    #[test]
    fn issuer_registry_rejects_zero_sentinel_and_self_referential_issuer_addresses() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(IssuerRegistryContract, ());
        let client = IssuerRegistryContractClient::new(&env, &contract_id);
        let admin = Address::generate(&env);
        client.initialize(&admin);

        let zero = Address::from_str(&env, ZERO_ADDR);
        let result = client.try_register_issuer(&bytes(&env, 1), &zero, &bytes(&env, 2));
        assert_eq!(result, Err(Ok(IssuerError::InvalidAddress)));

        let issuer = Address::generate(&env);
        let issuer_id = bytes(&env, 9);
        client.register_issuer(&issuer_id, &issuer, &bytes(&env, 7));
        let same = client.try_rotate_issuer_address(&issuer_id, &issuer);
        assert_eq!(same, Err(Ok(IssuerError::InvalidAddress)));
    }

    #[test]
    fn proof_registry_rejects_zero_dependency_addresses_and_self_references() {
        let env = Env::default();
        env.mock_all_auths();
        let config_id = env.register(ProtocolConfigContract, ());
        let config = ProtocolConfigContractClient::new(&env, &config_id);
        let admin = Address::generate(&env);
        let issuer = Address::generate(&env);
        let issuer_registry_id = env.register(IssuerRegistryContract, ());
        let issuer_registry = IssuerRegistryContractClient::new(&env, &issuer_registry_id);
        config.initialize(&admin);
        config.approve_schema_version(&1);
        issuer_registry.initialize(&admin);
        issuer_registry.register_issuer(&bytes(&env, 1), &issuer, &bytes(&env, 2));

        let proof_id = env.register(ProofRegistryContract, ());
        let proof_client = ProofRegistryContractClient::new(&env, &proof_id);
        let zero = Address::from_str(&env, ZERO_ADDR);

        let bad_init = proof_client.try_initialize(&admin, &zero, &config_id);
        assert_eq!(bad_init, Err(Ok(ContractError::InvalidInput)));

        proof_client.initialize(&admin, &issuer_registry_id, &config_id);
        let result = proof_client.try_register_proof(
            &bytes(&env, 3),
            &bytes(&env, 4),
            &Address::from_str(&env, ZERO_ADDR),
            &1,
            &1_000,
        );
        assert_eq!(result, Err(Ok(ProofError::InvalidAddress)));

        let self_result = proof_client.try_register_proof(
            &bytes(&env, 5),
            &bytes(&env, 6),
            &proof_id,
            &1,
            &1_000,
        );
        assert_eq!(self_result, Err(Ok(ProofError::InvalidAddress)));
    }

    #[test]
    fn malformed_encoded_addresses_do_not_mutate_state() {
        let env = Env::default();
        env.mock_all_auths();
        let config_id = env.register(ProtocolConfigContract, ());
        let config = ProtocolConfigContractClient::new(&env, &config_id);
        let admin = Address::generate(&env);
        let issuer_registry_id = env.register(IssuerRegistryContract, ());
        let issuer_registry = IssuerRegistryContractClient::new(&env, &issuer_registry_id);
        config.initialize(&admin);
        issuer_registry.initialize(&admin);

        let malformed = Address::from_str(&env, "BADADDRESS");
        let result = issuer_registry.try_register_issuer(&bytes(&env, 1), &malformed, &bytes(&env, 2));
        assert_eq!(result, Err(Ok(IssuerError::InvalidAddress)));

        assert_eq!(issuer_registry.try_get_issuer(&bytes(&env, 1)), Err(Ok(IssuerError::IssuerNotFound)));
        assert_eq!(config.get_admin(), admin);
    }
}
