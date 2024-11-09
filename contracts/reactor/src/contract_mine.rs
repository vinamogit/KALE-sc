use soroban_sdk::{contractimpl, panic_with_error, token, Address, BytesN, Env};

use crate::{
    errors::Errors,
    storage::{
        extend_instance_ttl, get_mine_admin, get_mine_paused, has_mine_admin, set_mine_admin,
        set_mine_asset, set_mine_paused,
    },
    MineContractTrait, MineKalepailContract, MineKalepailContractClient,
};

#[contractimpl]
impl MineContractTrait for MineKalepailContract {
    fn discover_mine(env: Env, admin: Address, asset: Address) {
        admin.require_auth();

        if has_mine_admin(&env) {
            panic_with_error!(&env, &Errors::AlreadyDiscovered);
        }

        if token::StellarAssetClient::new(&env, &asset).admin() != env.current_contract_address() {
            panic_with_error!(&env, &Errors::AssetAdminMismatch);
        }

        set_mine_admin(&env, &admin);
        set_mine_asset(&env, &asset);

        extend_instance_ttl(&env);
    }

    fn upgrade_mine(env: Env, hash: BytesN<32>) {
        let admin = get_mine_admin(&env);

        admin.require_auth();

        env.deployer().update_current_contract_wasm(hash);

        extend_instance_ttl(&env);
    }

    fn pause_mine(env: Env) {
        let paused = get_mine_paused(&env);

        if paused {
            panic_with_error!(&env, &Errors::MineIsPaused);
        }

        let admin = get_mine_admin(&env);

        admin.require_auth();

        set_mine_paused(&env, true);
    }

    fn unpause_mine(env: Env) {
        let paused = get_mine_paused(&env);

        if paused {
            panic_with_error!(&env, &Errors::MineIsNotPaused);
        }

        let admin = get_mine_admin(&env);

        admin.require_auth();

        set_mine_paused(&env, false);

        extend_instance_ttl(&env);
    }
}
