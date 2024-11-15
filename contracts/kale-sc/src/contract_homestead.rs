// use crate::ContractArgs;
use soroban_sdk::{contractimpl, panic_with_error, token, Address, BytesN, Env};

use crate::{
    errors::Errors,
    storage::{
        extend_instance_ttl, get_farm_homesteader, get_farm_paused, has_farm_homesteader,
        set_farm_asset, set_farm_homesteader, set_farm_paused,
    },
    types::Storage,
    Contract, ContractClient, HomesteadTrait,
};

#[contractimpl]
impl HomesteadTrait for Contract {
    fn homestead(env: Env, farmer: Address, asset: Address) {
        farmer.require_auth();

        if has_farm_homesteader(&env) {
            panic_with_error!(&env, &Errors::HomesteadExists);
        }

        if token::StellarAssetClient::new(&env, &asset).admin() != env.current_contract_address() {
            panic_with_error!(&env, &Errors::AssetAdminInvalid);
        }

        set_farm_homesteader(&env, &farmer);
        set_farm_asset(&env, &asset);

        extend_instance_ttl(&env);
    }

    fn upgrade(env: Env, hash: BytesN<32>) {
        let homesteader = get_farm_homesteader(&env);

        homesteader.require_auth();

        env.deployer().update_current_contract_wasm(hash);

        extend_instance_ttl(&env);
    }

    fn pause(env: Env) {
        let homesteader = get_farm_homesteader(&env);
        let paused = get_farm_paused(&env);

        homesteader.require_auth();

        if paused {
            panic_with_error!(&env, &Errors::FarmPaused);
        }

        set_farm_paused(&env, true);

        // no `extend_instance_ttl` as the farm is being paused
    }

    fn unpause(env: Env) {
        let homesteader = get_farm_homesteader(&env);
        let paused = get_farm_paused(&env);

        homesteader.require_auth();

        if paused {
            panic_with_error!(&env, &Errors::FarmNotPaused);
        }

        set_farm_paused(&env, false);

        extend_instance_ttl(&env);
    }

    fn remove_block(env: Env, index: u32) {
        let homesteader = get_farm_homesteader(&env);

        homesteader.require_auth();

        env.storage().temporary().remove(&Storage::Block(index));
    }
}
