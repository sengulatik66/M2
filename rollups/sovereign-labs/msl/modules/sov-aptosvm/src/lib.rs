#[cfg(feature = "experimental")]
pub mod call;
#[cfg(feature = "experimental")]
pub mod evm;
#[cfg(feature = "experimental")]
pub mod genesis;
#[cfg(feature = "native")]
#[cfg(feature = "experimental")]
pub mod query;
#[cfg(feature = "experimental")]
#[cfg(test)]
mod tests;

#[cfg(feature = "experimental")]
pub use aptos_experimental::{AptosVm, AptosVmConfig};

#[cfg(feature = "experimental")]
extern crate dirs;

#[cfg(feature = "experimental")]
mod aptos_experimental {

    use anyhow::anyhow;

    use sov_modules_api::Error;
    use sov_modules_macros::ModuleInfo;
    use sov_state::WorkingSet;

    use aptos_types::transaction::{Transaction};
    use aptos_db::AptosDB;
    use aptos_storage_interface::DbReaderWriter;
    use aptos_types::validator_signer::ValidatorSigner;

    use aptos_executor::block_executor::BlockExecutor;
    use aptos_executor::db_bootstrapper::{generate_waypoint, maybe_bootstrap};
    use aptos_executor_types::BlockExecutorTrait;
    use aptos_vm::AptosVM;
    // use anyhow::{Error};

    use borsh::{BorshDeserialize, BorshSerialize};

    use std::sync::Arc;

    #[derive(Clone)]
    pub struct AptosVmConfig {
        pub data: Vec<u8>,
    }

    #[allow(dead_code)]
    #[derive(ModuleInfo, Clone)]
    pub struct AptosVm<C: sov_modules_api::Context> {
        #[address]
        pub(crate) address: C::Address,

        #[state]
        pub(crate) db_path: sov_state::StateValue<String>,

        // TODO: this may be redundant with address
        #[state]
        pub(crate) validator_signer: sov_state::StateValue<Vec<u8>>, // TODO: fix validator signer incompatability

        // This is string because we are using transaction.hash: https://github.com/movemntdev/aptos-core/blob/112ad6d8e229a19cfe471153b2fd48f1f22b9684/crates/indexer/src/models/transactions.rs#L31
        #[state]
        pub(crate) transactions: sov_state::StateMap<String, Vec<u8>>, // TODO: fix Transaction serialiation incompatability
    }

    impl<C: sov_modules_api::Context> sov_modules_api::Module for AptosVm<C> {
        type Context = C;

        type Config = AptosVmConfig;

        type CallMessage = super::call::CallMessage;

        fn genesis(
            &self,
            config: &Self::Config,
            working_set: &mut WorkingSet<C::Storage>,
        ) -> Result<(), Error> {

            Ok(self.init_module(config, working_set)?)

        }

        fn call(
            &self,
            msg: Self::CallMessage,
            context: &Self::Context,
            working_set: &mut WorkingSet<C::Storage>,
        ) -> Result<sov_modules_api::CallResponse, Error> {

            Ok(self.execute_call(msg.serialized_tx, context, working_set)?)

        }
    }

 

    impl<C: sov_modules_api::Context> AptosVm<C> {

        pub(crate) fn get_db(
            &self,
            working_set: &mut WorkingSet<C::Storage>,
        ) -> Result<
            DbReaderWriter, 
            Error
        > {

            let path = self.db_path.get(working_set).ok_or(
                anyhow::Error::msg("Database path is not set.")
            )?;
            // TODO: swap for non-test db
            // TODO: swap for celestia DA
            Ok(DbReaderWriter::new(AptosDB::new_for_sov(path.as_str())))

        }

        pub(crate) fn get_executor(
            &self,
            working_set: &mut WorkingSet<C::Storage>,
        ) -> Result<
            BlockExecutor<AptosVM>, 
            Error
        > {

            let db = self.get_db(working_set)?;
            Ok(BlockExecutor::new(db.clone()))

        }

    }

}
