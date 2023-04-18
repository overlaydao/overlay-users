//! OVERLAY users smart contract.
//!
//! This is the repository that stores OVERLAY user's data.
//! * OVERLAY user could be a curator or a validator.
//! * When project admin marks the OVERLAY user as curator, then its project id is stored in the user state.
//! * When project admin marks the OVERLAY user as validator, then its project id is stored in the user state.

#![cfg_attr(all(not(feature = "std"), not(test)), no_std)]
use concordium_std::*;
use core::fmt::Debug;

type ProjectId = String;

/// The state of the OVERLAY users
#[derive(Serial, DeserialWithState, StateClone)]
#[concordium(state_parameter = "S")]
struct State<S> {
    /// Owner/Admin address of this contract module.
    admin: AccountAddress,
    /// overlay-projects contract address that will control curator / validator data.
    project_contract_addr: ContractAddress,
    /// OVERLAY user data map.
    user: StateMap<AccountAddress, UserState, S>,
    /// All curator account addresses.
    curator_list: Vec<AccountAddress>,
    /// All validator account addresses.
    validator_list: Vec<AccountAddress>,
}

/// The state of a single OVERLAY user
#[derive(Serial, Deserial, SchemaType, Clone)]
struct UserState {
    is_curator: bool,
    is_validator: bool,
    curated_projects: Vec<ProjectId>,
    validated_projects: Vec<ProjectId>,
}

/// The parameter schema for `transfer_admin` function.
#[derive(Serial, Deserial, SchemaType)]
struct TransferAdminParam {
    admin: AccountAddress,
}

/// The parameter schema for `add_project_contract` function.
#[derive(Serial, Deserial, SchemaType)]
struct AddProjectContractParam {
    project_contract_addr: ContractAddress,
}

/// Single account address parameter that is commonly used.
#[derive(Serial, Deserial, SchemaType)]
struct AddrParam {
    addr: AccountAddress,
}
/// The parameter schema for `add_curator` function.
type AddCuratorParam = AddrParam;
/// The parameter schema for `remove_curator` function.
type RemoveCuratorParam = AddrParam;
/// The parameter schema for `add_validator` function.
type AddValidatorParam = AddrParam;
/// The parameter schema for `remove_validator` function.
type RemoveValidatorParam = AddrParam;
/// The parameter schema for `view_user` function.
type ViewUserParam = AddrParam;

/// The parameter schema for `curate` function.
#[derive(Serial, Deserial, SchemaType)]
struct CurateParam {
    addr: AccountAddress,
    project_id: ProjectId,
}

/// The parameter schema for `validate` function.
#[derive(Serial, Deserial, SchemaType)]
struct ValidateParam {
    addr: AccountAddress,
    project_id: ProjectId,
}

/// The parameter schema for `upgrade` function.
#[derive(Debug, Serialize, SchemaType)]
struct UpgradeParam {
    module: ModuleReference,
    migrate: Option<(OwnedEntrypointName, OwnedParameter)>,
}

/// The response schema for `view_admin` function.
#[derive(Serial, Deserial, SchemaType)]
struct ViewAdminRes {
    admin: AccountAddress,
    project_contract_addr: ContractAddress,
    curator_list: Vec<AccountAddress>,
    validator_list: Vec<AccountAddress>,
}

/// The response schema for `view_user` function.
type ViewUserResponse = UserState;

/// The response schema for `view_users` function.
type ViewUsersResponse = Vec<(AccountAddress, UserState)>;

/// Custom error definitions of OVERLAY users smart contract.
#[derive(Debug, PartialEq, Eq, Reject, Serial, SchemaType)]
enum Error {
    #[from(ParseError)]
    ParseParamsError,
    InvalidCaller,
    InvalidArgument,
}

type ContractResult<A> = Result<A, Error>;

/// The smart contract module init function.
/// Although anyone can init this module, this function is expected to be called by OVERLAY team.
#[init(contract = "overlay-users")]
fn contract_init<S: HasStateApi>(
    ctx: &impl HasInitContext,
    state_builder: &mut StateBuilder<S>,
) -> InitResult<State<S>> {
    let state = State {
        admin: ctx.init_origin(),
        project_contract_addr: ContractAddress::new(0u64, 0u64),
        user: state_builder.new_map(),
        curator_list: Vec::new(),
        validator_list: Vec::new(),
    };
    Ok(state)
}

/// Transfer admin of this module to another account.
///
/// Caller: current admin account.
/// Reject if:
/// * Caller is not the current admin account.
#[receive(
    contract = "overlay-users",
    name = "transfer_admin",
    parameter = "TransferAdminParam",
    mutable,
    error = "Error"
)]
fn contract_transfer_admin<S: HasStateApi>(
    ctx: &impl HasReceiveContext,
    host: &mut impl HasHost<State<S>, StateApiType = S>,
) -> ContractResult<()> {
    let params: TransferAdminParam = ctx.parameter_cursor().get()?;
    let state = host.state_mut();
    ensure!(ctx.invoker() == state.admin, Error::InvalidCaller);
    state.admin = params.admin;
    Ok(())
}

/// Set associated overlay-projects contract address.
///
/// Caller: current admin account.
/// Reject if:
/// * Caller is not the current admin account.
#[receive(
    contract = "overlay-users",
    name = "add_project_contract",
    parameter = "AddProjectContractParam",
    mutable,
    error = "Error"
)]
fn contract_add_project_contract<S: HasStateApi>(
    ctx: &impl HasReceiveContext,
    host: &mut impl HasHost<State<S>, StateApiType = S>,
) -> ContractResult<()> {
    let params: AddProjectContractParam = ctx.parameter_cursor().get()?;
    let state = host.state_mut();
    ensure!(ctx.invoker() == state.admin, Error::InvalidCaller);
    state.project_contract_addr = params.project_contract_addr;
    Ok(())
}

/// Update inputted user account as a curator.
/// If the requested user address dose not exist in the state, default user data would be created.
///
/// Caller: current admin account.
/// Reject if:
/// * Caller is not the current admin account.
#[receive(
    contract = "overlay-users",
    name = "add_curator",
    parameter = "AddCuratorParam",
    mutable
)]
fn contract_add_curator<S: HasStateApi>(
    ctx: &impl HasReceiveContext,
    host: &mut impl HasHost<State<S>>,
) -> ContractResult<()> {
    let params: AddCuratorParam = ctx.parameter_cursor().get()?;
    let state = host.state_mut();
    ensure!(ctx.invoker() == state.admin, Error::InvalidCaller);
    state
        .user
        .entry(params.addr)
        .and_modify(|user_state| user_state.is_curator = true)
        .or_insert_with(|| UserState {
            is_curator: true,
            is_validator: false,
            curated_projects: Vec::new(),
            validated_projects: Vec::new(),
        });
    if !state.curator_list.contains(&params.addr) {
        state.curator_list.push(params.addr);
    }
    Ok(())
}

/// Unmark inputted user account as a curator.
///
/// Caller: current admin account.
/// Reject if:
/// * Caller is not the current admin account.
#[receive(
    contract = "overlay-users",
    name = "remove_curator",
    parameter = "RemoveCuratorParam",
    mutable
)]
fn contract_remove_curator<S: HasStateApi>(
    ctx: &impl HasReceiveContext,
    host: &mut impl HasHost<State<S>>,
) -> ContractResult<()> {
    let params: RemoveCuratorParam = ctx.parameter_cursor().get()?;
    let state = host.state_mut();
    ensure!(ctx.invoker() == state.admin, Error::InvalidCaller);
    state.user.entry(params.addr).and_modify(|user_state| {
        user_state.is_curator = false;
    });
    state.curator_list.retain(|x| *x != params.addr);
    Ok(())
}

/// Update inputted user account as a validator.
/// If the requested user address dose not exist in the state, default user data would be created.
///
/// Caller: current admin account.
/// Reject if:
/// * Caller is not the current admin account.
#[receive(
    contract = "overlay-users",
    name = "add_validator",
    parameter = "AddValidatorParam",
    mutable
)]
fn contract_add_validator<S: HasStateApi>(
    ctx: &impl HasReceiveContext,
    host: &mut impl HasHost<State<S>>,
) -> ContractResult<()> {
    let params: AddValidatorParam = ctx.parameter_cursor().get()?;
    let state = host.state_mut();
    ensure!(ctx.invoker() == state.admin, Error::InvalidCaller);
    state
        .user
        .entry(params.addr)
        .and_modify(|user_state| user_state.is_validator = true)
        .or_insert_with(|| UserState {
            is_curator: false,
            is_validator: true,
            curated_projects: Vec::new(),
            validated_projects: Vec::new(),
        });
    if !state.validator_list.contains(&params.addr) {
        state.validator_list.push(params.addr);
    }
    Ok(())
}

/// Unmark inputted user account as a validator.
///
/// Caller: current admin account.
/// Reject if:
/// * Caller is not the current admin account.
#[receive(
    contract = "overlay-users",
    name = "remove_validator",
    parameter = "RemoveValidatorParam",
    mutable
)]
fn contract_remove_validator<S: HasStateApi>(
    ctx: &impl HasReceiveContext,
    host: &mut impl HasHost<State<S>>,
) -> ContractResult<()> {
    let params: RemoveValidatorParam = ctx.parameter_cursor().get()?;
    let state = host.state_mut();
    ensure!(ctx.invoker() == state.admin, Error::InvalidCaller);

    state.user.entry(params.addr).and_modify(|user_state| {
        user_state.is_validator = false;
    });
    state.validator_list.retain(|x| *x != params.addr);
    Ok(())
}

/// Add project id to the user curated projects state.
///
/// Caller: associated overlay-projects smart contract
/// Reject if:
/// * Caller is not the associated overlay-projects smart contract address
/// * The inputted user is not registered as a curator.
///
/// This function is designed to be called by the following smart contract functions.
/// * overlay-projects.curate_project
#[receive(
    contract = "overlay-users",
    name = "curate",
    parameter = "CurateParam",
    mutable
)]
fn contract_curate<S: HasStateApi>(
    ctx: &impl HasReceiveContext,
    host: &mut impl HasHost<State<S>>,
) -> ContractResult<()> {
    let params: CurateParam = ctx.parameter_cursor().get()?;
    let state = host.state_mut();
    ensure!(
        ctx.sender() == Address::Contract(state.project_contract_addr),
        Error::InvalidCaller
    );
    let target_user = state.user.get_mut(&params.addr);
    ensure!(target_user.is_some(), Error::InvalidArgument);
    let mut target_user = target_user.unwrap();
    ensure!(target_user.is_curator, Error::InvalidArgument);
    if !target_user.curated_projects.contains(&params.project_id) {
        target_user.curated_projects.push(params.project_id);
    }
    Ok(())
}

/// Add project id to the user validated projects state.
///
/// Caller: associated overlay-projects smart contract
/// Reject if:
/// * Caller is not the associated overlay-projects smart contract address
/// * The inputted user is not registered as a validator.
///
/// This function is designed to be called by the following smart contract functions.
/// * overlay-projects.validate_project
#[receive(
    contract = "overlay-users",
    name = "validate",
    parameter = "ValidateParam",
    mutable
)]
fn contract_validate<S: HasStateApi>(
    ctx: &impl HasReceiveContext,
    host: &mut impl HasHost<State<S>>,
) -> ContractResult<()> {
    let params: ValidateParam = ctx.parameter_cursor().get()?;
    let state = host.state_mut();
    ensure!(
        ctx.sender() == Address::Contract(state.project_contract_addr),
        Error::InvalidCaller
    );
    let target_user = state.user.get_mut(&params.addr);
    ensure!(target_user.is_some(), Error::InvalidArgument);
    let mut target_user = target_user.unwrap();
    ensure!(target_user.is_validator, Error::InvalidArgument);
    if !target_user.validated_projects.contains(&params.project_id) {
        target_user.validated_projects.push(params.project_id);
    }
    Ok(())
}

/// Smart contract module upgrade function.
/// For more information see https://developer.concordium.software/en/mainnet/smart-contracts/guides/upgradeable-contract.html#guide-upgradable-contract
#[receive(
    contract = "overlay-users",
    name = "upgrade",
    parameter = "UpgradeParam",
    mutable
)]
fn contract_upgrade<S: HasStateApi>(
    ctx: &impl HasReceiveContext,
    host: &mut impl HasHost<State<S>, StateApiType = S>,
) -> ReceiveResult<()> {
    ensure!(ctx.sender().matches_account(&ctx.owner()));
    let params: UpgradeParam = ctx.parameter_cursor().get()?;
    host.upgrade(params.module)?;
    if let Some((func, parameter)) = params.migrate {
        host.invoke_contract_raw(
            &ctx.self_address(),
            parameter.as_parameter(),
            func.as_entrypoint_name(),
            Amount::zero(),
        )?;
    }
    Ok(())
}

/// View the admin state.
///
/// Caller: Admin account only.
#[receive(
    contract = "overlay-users",
    name = "view_admin",
    return_value = "ViewAdminRes"
)]
fn contract_view_admin<S: HasStateApi>(
    ctx: &impl HasReceiveContext,
    host: &impl HasHost<State<S>, StateApiType = S>,
) -> ContractResult<ViewAdminRes> {
    let state = host.state();
    ensure!(ctx.invoker() == state.admin, Error::InvalidCaller);
    Ok(ViewAdminRes {
        admin: state.admin,
        project_contract_addr: state.project_contract_addr,
        curator_list: state.curator_list.clone(),
        validator_list: state.validator_list.clone(),
    })
}

/// View the user state.
/// If the requested user address dose not exist in the state, it returns the default data.
///
/// Caller: Any accounts / Any contracts
///
/// This function is designed to be called by the following smart contract functions.
/// * overlay-projects.curate_project
/// * overlay-projects.curate_project_admin
/// * overlay-projects.validate_project
/// * overlay-projects.validate_project_admin
#[receive(
    contract = "overlay-users",
    name = "view_user",
    parameter = "ViewUserParam",
    return_value = "UserState"
)]
fn contract_view_user<S: HasStateApi>(
    ctx: &impl HasReceiveContext,
    host: &impl HasHost<State<S>, StateApiType = S>,
) -> ContractResult<ViewUserResponse> {
    let params: ViewUserParam = ctx.parameter_cursor().get()?;
    let state = host.state();
    let user_state = state
        .user
        .get(&params.addr)
        .map(|user_state_ref| user_state_ref.clone())
        .unwrap_or(UserState {
            is_curator: false,
            is_validator: false,
            curated_projects: Vec::new(),
            validated_projects: Vec::new(),
        });
    Ok(user_state)
}

/// View the all user state.
///
/// Caller: Any accounts / Any contracts
#[receive(
    contract = "overlay-users",
    name = "view_users",
    return_value = "ViewUsersResponse"
)]
fn contract_view_users<S: HasStateApi>(
    _ctx: &impl HasReceiveContext,
    host: &impl HasHost<State<S>, StateApiType = S>,
) -> ContractResult<ViewUsersResponse> {
    let users = &host.state().user;
    let users_response = users
        .iter()
        .map(|(account_address_ref, user_state_ref)| {
            (account_address_ref.clone(), user_state_ref.clone())
        })
        .collect();
    Ok(users_response)
}

/// implements Debug for State inside test functions.
/// this implementation will be build only when `concordium-std/wasm-test` feature is active.
/// (e.g. when launched by `cargo concordium test`)
#[concordium_cfg_test]
impl<S: HasStateApi> Debug for State<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "admin: {:?}, project_contract_addr: {:?}, ",
            self.admin, self.project_contract_addr
        )?;
        for (address, state) in self.user.iter() {
            write!(f, "user_address: {:?}, user_state: {:?}, ", address, state)?;
        }
        write!(
            f,
            "curator_list: {:?}, validator_list: {:?}",
            self.curator_list, self.validator_list
        )
    }
}

/// implements PartialEq for `claim_eq` inside test functions.
/// this implementation will be build only when `concordium-std/wasm-test` feature is active.
/// (e.g. when launched by `cargo concordium test`)
#[concordium_cfg_test]
impl<S: HasStateApi> PartialEq for State<S> {
    fn eq(&self, other: &Self) -> bool {
        if self.admin != other.admin {
            return false;
        }
        if self.project_contract_addr != other.project_contract_addr {
            return false;
        }
        if self.user.iter().count() != other.user.iter().count() {
            return false;
        }
        for (my_user_address, my_user_state) in self.user.iter() {
            let other_user_state = other.user.get(&my_user_address);
            if other_user_state.is_none() {
                return false;
            }
            let other_user_state = other_user_state.unwrap();
            if my_user_state.clone() != other_user_state.clone() {
                return false;
            }
        }
        if self.curator_list != other.curator_list {
            return false;
        }
        if self.validator_list != other.validator_list {
            return false;
        }
        true
    }

    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}

/// implements Debug for UserState inside test functions.
/// this implementation will be build only when `concordium-std/wasm-test` feature is active.
/// (e.g. when launched by `cargo concordium test`)
#[concordium_cfg_test]
impl Debug for UserState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "is_curator: {}, is_validator: {}, curated_projects: {:?}, validated_projects: {:?}",
            self.is_curator, self.is_validator, self.curated_projects, self.validated_projects
        )
    }
}

/// implements PartialEq for `claim_eq` inside test functions.
/// this implementation will be build only when `concordium-std/wasm-test` feature is active.
/// (e.g. when launched by `cargo concordium test`)
#[concordium_cfg_test]
impl PartialEq for UserState {
    fn eq(&self, other: &Self) -> bool {
        if self.is_curator != other.is_curator {
            return false;
        }
        if self.is_validator != other.is_validator {
            return false;
        }
        if self.curated_projects != other.curated_projects {
            return false;
        }
        if self.validated_projects != other.validated_projects {
            return false;
        }
        true
    }

    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}

#[concordium_cfg_test]
mod tests {
    use super::*;
    use concordium_std::hashes::HashBytes;
    use test_infrastructure::*;

    #[concordium_test]
    /// Test that init succeeds.
    fn test_init() {
        // invoker will be an admin
        let invoker = AccountAddress([0; 32]);
        let mut ctx = TestInitContext::empty();
        ctx.set_init_origin(invoker);

        let mut state_builder = TestStateBuilder::new();

        let expected_state = State {
            admin: invoker,
            project_contract_addr: ContractAddress::new(0, 0),
            user: state_builder.new_map(),
            curator_list: Vec::new(),
            validator_list: Vec::new(),
        };

        // execute init
        let result = contract_init(&ctx, &mut state_builder);

        // check init result
        claim!(result.is_ok());
        let actual_state = result.unwrap();
        claim_eq!(
            actual_state,
            expected_state,
            "state has been changed unexpectedly..."
        );
    }

    #[concordium_test]
    /// Test that overlay-users.transfer_admin was successfully invoked by admin account.
    fn test_contract_transfer_admin_invoked_by_admin() {
        let admin = AccountAddress([0; 32]);
        let try_to_transfer_to = AccountAddress([2; 32]);

        let mut ctx = TestReceiveContext::empty();
        ctx.set_invoker(admin);
        let mut state_builder = TestStateBuilder::new();
        let state = State {
            admin,
            project_contract_addr: ContractAddress::new(0, 0),
            user: state_builder.new_map(),
            curator_list: Vec::new(),
            validator_list: Vec::new(),
        };
        let expected_state = State {
            admin: try_to_transfer_to,
            project_contract_addr: ContractAddress::new(0, 0),
            user: state_builder.new_map(),
            curator_list: Vec::new(),
            validator_list: Vec::new(),
        };
        let mut host = TestHost::new(state, state_builder);

        // create parameters
        let params = TransferAdminParam {
            admin: try_to_transfer_to,
        };
        let params_byte = to_bytes(&params);
        ctx.set_parameter(&params_byte);

        // invoke method
        let result = contract_transfer_admin(&ctx, &mut host);
        claim!(result.is_ok());
        let actual_state = host.state();
        claim_eq!(
            *actual_state,
            expected_state,
            "state has been changed unexpectedly..."
        );
    }

    #[concordium_test]
    /// Test that overlay-users.transfer_admin was invoked by non-admin account.
    fn test_contract_transfer_admin_invoked_by_non_admin() {
        let admin = AccountAddress([0; 32]);
        let suspicious = AccountAddress([1; 32]);
        let try_to_transfer_to = AccountAddress([2; 32]);

        let mut ctx = TestReceiveContext::empty();
        ctx.set_invoker(suspicious);
        let mut state_builder = TestStateBuilder::new();
        let state = State {
            admin,
            project_contract_addr: ContractAddress::new(0, 0),
            user: state_builder.new_map(),
            curator_list: Vec::new(),
            validator_list: Vec::new(),
        };
        let mut host = TestHost::new(state, state_builder);

        // create parameters
        let params = TransferAdminParam {
            admin: try_to_transfer_to,
        };
        let params_byte = to_bytes(&params);
        ctx.set_parameter(&params_byte);

        // invoke method
        let result = contract_transfer_admin(&ctx, &mut host);
        claim!(result.is_err());
        claim_eq!(result.err(), Some(Error::InvalidCaller));
    }

    #[concordium_test]
    /// Test that overlay-users.add_project_contract was successfully invoked by admin account.
    fn test_contract_add_project_contract_invoked_by_admin() {
        let admin = AccountAddress([0; 32]);
        let project_contract_addr_to_be_set = ContractAddress::new(1, 2);

        let mut ctx = TestReceiveContext::empty();
        ctx.set_invoker(admin);
        let mut state_builder = TestStateBuilder::new();
        let state = State {
            admin,
            project_contract_addr: ContractAddress::new(0, 0),
            user: state_builder.new_map(),
            curator_list: Vec::new(),
            validator_list: Vec::new(),
        };
        let expected_state = State {
            admin,
            project_contract_addr: project_contract_addr_to_be_set,
            user: state_builder.new_map(),
            curator_list: Vec::new(),
            validator_list: Vec::new(),
        };
        let mut host = TestHost::new(state, state_builder);

        // create parameters
        let params = AddProjectContractParam {
            project_contract_addr: project_contract_addr_to_be_set,
        };
        let params_byte = to_bytes(&params);
        ctx.set_parameter(&params_byte);

        // invoke method
        let result = contract_add_project_contract(&ctx, &mut host);
        claim!(result.is_ok());
        let actual_state = host.state();
        claim_eq!(
            *actual_state,
            expected_state,
            "state has been changed unexpectedly..."
        );
    }

    #[concordium_test]
    /// Test that overlay-users.add_project_contract was invoked by non-admin account.
    fn test_contract_add_project_contract_invoked_by_non_admin() {
        let admin = AccountAddress([0; 32]);
        let suspicious = AccountAddress([1; 32]);

        let mut ctx = TestReceiveContext::empty();
        ctx.set_invoker(suspicious);
        let mut state_builder = TestStateBuilder::new();
        let state = State {
            admin,
            project_contract_addr: ContractAddress::new(0, 0),
            user: state_builder.new_map(),
            curator_list: Vec::new(),
            validator_list: Vec::new(),
        };
        let mut host = TestHost::new(state, state_builder);

        // create parameters
        let project_contract_addr = ContractAddress::new(1, 2);
        let params = AddProjectContractParam {
            project_contract_addr,
        };
        let params_byte = to_bytes(&params);
        ctx.set_parameter(&params_byte);

        // invoke method
        let result = contract_add_project_contract(&ctx, &mut host);
        claim!(result.is_err());
        claim_eq!(result.err(), Some(Error::InvalidCaller));
    }

    #[concordium_test]
    /// Test that overlay-users.add_curator handle new user entry.
    fn test_contract_add_new_curator_invoked_by_admin() {
        let admin = AccountAddress([0; 32]);
        let project_contract_addr = ContractAddress::new(0, 0);
        let existing_user = AccountAddress([1; 32]);
        let curator = AccountAddress([2; 32]);
        let mut ctx = TestReceiveContext::empty();
        ctx.set_invoker(admin);
        // setup state
        let mut state_builder = TestStateBuilder::new();
        let mut user = state_builder.new_map();
        user.insert(
            existing_user,
            UserState {
                is_curator: false,
                is_validator: false,
                curated_projects: Vec::new(),
                validated_projects: Vec::new(),
            },
        );
        let state = State {
            admin,
            project_contract_addr,
            user,
            curator_list: Vec::new(),
            validator_list: Vec::new(),
        };
        let mut expected_user = state_builder.new_map();
        expected_user.insert(
            existing_user,
            UserState {
                is_curator: false,
                is_validator: false,
                curated_projects: Vec::new(),
                validated_projects: Vec::new(),
            },
        );
        expected_user.insert(
            curator,
            UserState {
                is_curator: true,
                is_validator: false,
                curated_projects: Vec::new(),
                validated_projects: Vec::new(),
            },
        );
        let expected_state = State {
            admin,
            project_contract_addr,
            user: expected_user,
            curator_list: vec![curator],
            validator_list: Vec::new(),
        };
        let mut host = TestHost::new(state, state_builder);

        // create parameters
        let params = AddCuratorParam { addr: curator };
        let params_byte = to_bytes(&params);
        ctx.set_parameter(&params_byte);

        // invoke method
        let result = contract_add_curator(&ctx, &mut host);
        claim!(result.is_ok());
        let actual_state = host.state();
        claim_eq!(
            *actual_state,
            expected_state,
            "state has been changed unexpectedly..."
        );
    }

    #[concordium_test]
    /// Test that overlay-users.add_curator handle existing user entry.
    fn test_contract_modify_curator_invoked_by_admin() {
        let admin = AccountAddress([0; 32]);
        let project_contract_addr = ContractAddress::new(0, 0);
        let existing_user = AccountAddress([1; 32]);
        let mut ctx = TestReceiveContext::empty();
        ctx.set_invoker(admin);
        // setup state
        let mut state_builder = TestStateBuilder::new();
        let mut user = state_builder.new_map();
        user.insert(
            existing_user,
            UserState {
                is_curator: false,
                is_validator: false,
                curated_projects: Vec::new(),
                validated_projects: Vec::new(),
            },
        );
        let state = State {
            admin,
            project_contract_addr,
            user,
            curator_list: Vec::new(),
            validator_list: Vec::new(),
        };
        let mut expected_user = state_builder.new_map();
        expected_user.insert(
            existing_user,
            UserState {
                is_curator: true,
                is_validator: false,
                curated_projects: Vec::new(),
                validated_projects: Vec::new(),
            },
        );
        let expected_state = State {
            admin,
            project_contract_addr,
            user: expected_user,
            curator_list: vec![existing_user],
            validator_list: Vec::new(),
        };
        let mut host = TestHost::new(state, state_builder);

        // create parameters
        let params = AddCuratorParam {
            addr: existing_user,
        };
        let params_byte = to_bytes(&params);
        ctx.set_parameter(&params_byte);

        // invoke method
        let result = contract_add_curator(&ctx, &mut host);
        claim!(result.is_ok());
        let actual_state = host.state();
        claim_eq!(
            *actual_state,
            expected_state,
            "state has been changed unexpectedly..."
        );
    }

    #[concordium_test]
    /// Test that overlay-users.contract_add_curator was invoked by non-admin account.
    fn test_contract_add_curator_invoked_by_non_admin() {
        let admin = AccountAddress([0; 32]);
        let suspicious = AccountAddress([1; 32]);

        let mut ctx = TestReceiveContext::empty();
        ctx.set_invoker(suspicious);
        let mut state_builder = TestStateBuilder::new();
        let state = State {
            admin,
            project_contract_addr: ContractAddress::new(0, 0),
            user: state_builder.new_map(),
            curator_list: Vec::new(),
            validator_list: Vec::new(),
        };
        let mut host = TestHost::new(state, state_builder);

        // create parameters
        let params = AddCuratorParam {
            addr: AccountAddress([2; 32]),
        };
        let params_byte = to_bytes(&params);
        ctx.set_parameter(&params_byte);

        // invoke method
        let result = contract_add_curator(&ctx, &mut host);
        claim!(result.is_err());
        claim_eq!(result.err(), Some(Error::InvalidCaller));
    }

    #[concordium_test]
    /// Test that overlay-users.contract_remove_curator successfully remove the input
    fn test_contract_remove_curator_invoked_by_admin() {
        let admin = AccountAddress([0; 32]);
        let project_contract_addr = ContractAddress::new(0, 0);
        let existing_user = AccountAddress([1; 32]);
        let mut ctx = TestReceiveContext::empty();
        ctx.set_invoker(admin);
        // setup state
        let mut state_builder = TestStateBuilder::new();
        let mut user = state_builder.new_map();
        user.insert(
            existing_user,
            UserState {
                is_curator: true,
                is_validator: false,
                curated_projects: Vec::new(),
                validated_projects: Vec::new(),
            },
        );
        let state = State {
            admin,
            project_contract_addr,
            user,
            curator_list: vec![existing_user],
            validator_list: Vec::new(),
        };
        let mut expected_user = state_builder.new_map();
        expected_user.insert(
            existing_user,
            UserState {
                is_curator: false,
                is_validator: false,
                curated_projects: Vec::new(),
                validated_projects: Vec::new(),
            },
        );
        let expected_state = State {
            admin,
            project_contract_addr,
            user: expected_user,
            curator_list: Vec::new(),
            validator_list: Vec::new(),
        };
        let mut host = TestHost::new(state, state_builder);

        // create parameters
        let params = RemoveCuratorParam {
            addr: existing_user,
        };
        let params_byte = to_bytes(&params);
        ctx.set_parameter(&params_byte);

        // invoke method
        let result = contract_remove_curator(&ctx, &mut host);
        claim!(result.is_ok());
        let actual_state = host.state();
        claim_eq!(
            *actual_state,
            expected_state,
            "state has been changed unexpectedly..."
        );
    }

    #[concordium_test]
    /// Test that overlay-users.contract_remove_curator succeeds even if the parameter user is not
    /// curator
    fn test_contract_remove_curator_with_no_effect_invoked_by_admin() {
        let admin = AccountAddress([0; 32]);
        let project_contract_addr = ContractAddress::new(0, 0);
        let existing_user = AccountAddress([1; 32]);
        let not_curator = AccountAddress([2; 32]);
        let mut ctx = TestReceiveContext::empty();
        ctx.set_invoker(admin);
        // setup state
        let mut state_builder = TestStateBuilder::new();
        let mut user = state_builder.new_map();
        user.insert(
            existing_user,
            UserState {
                is_curator: true,
                is_validator: false,
                curated_projects: Vec::new(),
                validated_projects: Vec::new(),
            },
        );
        let state = State {
            admin,
            project_contract_addr,
            user,
            curator_list: vec![existing_user],
            validator_list: Vec::new(),
        };
        let mut expected_user = state_builder.new_map();
        expected_user.insert(
            existing_user,
            UserState {
                is_curator: true,
                is_validator: false,
                curated_projects: Vec::new(),
                validated_projects: Vec::new(),
            },
        );
        let expected_state = State {
            admin,
            project_contract_addr,
            user: expected_user,
            curator_list: vec![existing_user],
            validator_list: Vec::new(),
        };
        let mut host = TestHost::new(state, state_builder);

        // create parameters
        let params = RemoveCuratorParam { addr: not_curator };
        let params_byte = to_bytes(&params);
        ctx.set_parameter(&params_byte);

        // invoke method
        let result = contract_remove_curator(&ctx, &mut host);
        claim!(result.is_ok());
        let actual_state = host.state();
        claim_eq!(
            *actual_state,
            expected_state,
            "state has been changed unexpectedly..."
        );
    }

    #[concordium_test]
    /// Test that overlay-users.contract_remove_curator was invoked by non-admin account.
    fn test_contract_remove_curator_invoked_by_non_admin() {
        let admin = AccountAddress([0; 32]);
        let suspicious = AccountAddress([1; 32]);

        let mut ctx = TestReceiveContext::empty();
        ctx.set_invoker(suspicious);
        let mut state_builder = TestStateBuilder::new();
        let state = State {
            admin,
            project_contract_addr: ContractAddress::new(0, 0),
            user: state_builder.new_map(),
            curator_list: Vec::new(),
            validator_list: Vec::new(),
        };
        let mut host = TestHost::new(state, state_builder);

        // create parameters
        let params = RemoveCuratorParam {
            addr: AccountAddress([2; 32]),
        };
        let params_byte = to_bytes(&params);
        ctx.set_parameter(&params_byte);

        // invoke method
        let result = contract_remove_curator(&ctx, &mut host);
        claim!(result.is_err());
        claim_eq!(result.err(), Some(Error::InvalidCaller));
    }

    #[concordium_test]
    /// Test that overlay-users.add_validator handle new user entry.
    fn test_contract_add_new_validator_invoked_by_admin() {
        let admin = AccountAddress([0; 32]);
        let project_contract_addr = ContractAddress::new(0, 0);
        let existing_user = AccountAddress([1; 32]);
        let validator = AccountAddress([2; 32]);
        let mut ctx = TestReceiveContext::empty();
        ctx.set_invoker(admin);
        // setup state
        let mut state_builder = TestStateBuilder::new();
        let mut user = state_builder.new_map();
        user.insert(
            existing_user,
            UserState {
                is_curator: false,
                is_validator: false,
                curated_projects: Vec::new(),
                validated_projects: Vec::new(),
            },
        );
        let state = State {
            admin,
            project_contract_addr,
            user,
            curator_list: Vec::new(),
            validator_list: Vec::new(),
        };
        let mut expected_user = state_builder.new_map();
        expected_user.insert(
            existing_user,
            UserState {
                is_curator: false,
                is_validator: false,
                curated_projects: Vec::new(),
                validated_projects: Vec::new(),
            },
        );
        expected_user.insert(
            validator,
            UserState {
                is_curator: false,
                is_validator: true,
                curated_projects: Vec::new(),
                validated_projects: Vec::new(),
            },
        );
        let expected_state = State {
            admin,
            project_contract_addr,
            user: expected_user,
            curator_list: Vec::new(),
            validator_list: vec![validator],
        };
        let mut host = TestHost::new(state, state_builder);

        // create parameters
        let params = AddValidatorParam { addr: validator };
        let params_byte = to_bytes(&params);
        ctx.set_parameter(&params_byte);

        // invoke method
        let result = contract_add_validator(&ctx, &mut host);
        claim!(result.is_ok());
        let actual_state = host.state();
        claim_eq!(
            *actual_state,
            expected_state,
            "state has been changed unexpectedly..."
        );
    }

    #[concordium_test]
    /// Test that overlay-users.add_validator handle existing user entry.
    fn test_contract_modify_validator_invoked_by_admin() {
        let admin = AccountAddress([0; 32]);
        let project_contract_addr = ContractAddress::new(0, 0);
        let existing_user = AccountAddress([1; 32]);
        let mut ctx = TestReceiveContext::empty();
        ctx.set_invoker(admin);
        // setup state
        let mut state_builder = TestStateBuilder::new();
        let mut user = state_builder.new_map();
        user.insert(
            existing_user,
            UserState {
                is_curator: false,
                is_validator: false,
                curated_projects: Vec::new(),
                validated_projects: Vec::new(),
            },
        );
        let state = State {
            admin,
            project_contract_addr,
            user,
            curator_list: Vec::new(),
            validator_list: Vec::new(),
        };
        let mut expected_user = state_builder.new_map();
        expected_user.insert(
            existing_user,
            UserState {
                is_curator: false,
                is_validator: true,
                curated_projects: Vec::new(),
                validated_projects: Vec::new(),
            },
        );
        let expected_state = State {
            admin,
            project_contract_addr,
            user: expected_user,
            curator_list: Vec::new(),
            validator_list: vec![existing_user],
        };
        let mut host = TestHost::new(state, state_builder);

        // create parameters
        let params = AddValidatorParam {
            addr: existing_user,
        };
        let params_byte = to_bytes(&params);
        ctx.set_parameter(&params_byte);

        // invoke method
        let result = contract_add_validator(&ctx, &mut host);
        claim!(result.is_ok());
        let actual_state = host.state();
        claim_eq!(
            *actual_state,
            expected_state,
            "state has been changed unexpectedly..."
        );
    }

    #[concordium_test]
    /// Test that overlay-users.contract_add_validator was invoked by non-admin account.
    fn test_contract_add_validator_invoked_by_non_admin() {
        let admin = AccountAddress([0; 32]);
        let suspicious = AccountAddress([1; 32]);

        let mut ctx = TestReceiveContext::empty();
        ctx.set_invoker(suspicious);
        let mut state_builder = TestStateBuilder::new();
        let state = State {
            admin,
            project_contract_addr: ContractAddress::new(0, 0),
            user: state_builder.new_map(),
            curator_list: Vec::new(),
            validator_list: Vec::new(),
        };
        let mut host = TestHost::new(state, state_builder);

        // create parameters
        let params = AddValidatorParam {
            addr: AccountAddress([2; 32]),
        };
        let params_byte = to_bytes(&params);
        ctx.set_parameter(&params_byte);

        // invoke method
        let result = contract_add_validator(&ctx, &mut host);
        claim!(result.is_err());
        claim_eq!(result.err(), Some(Error::InvalidCaller));
    }

    #[concordium_test]
    /// Test that overlay-users.contract_remove_validator successfully remove the input
    fn test_contract_remove_validator_invoked_by_admin() {
        let admin = AccountAddress([0; 32]);
        let project_contract_addr = ContractAddress::new(0, 0);
        let existing_user = AccountAddress([1; 32]);
        let mut ctx = TestReceiveContext::empty();
        ctx.set_invoker(admin);
        // setup state
        let mut state_builder = TestStateBuilder::new();
        let mut user = state_builder.new_map();
        user.insert(
            existing_user,
            UserState {
                is_curator: false,
                is_validator: true,
                curated_projects: Vec::new(),
                validated_projects: Vec::new(),
            },
        );
        let state = State {
            admin,
            project_contract_addr,
            user,
            curator_list: Vec::new(),
            validator_list: vec![existing_user],
        };
        let mut expected_user = state_builder.new_map();
        expected_user.insert(
            existing_user,
            UserState {
                is_curator: false,
                is_validator: false,
                curated_projects: Vec::new(),
                validated_projects: Vec::new(),
            },
        );
        let expected_state = State {
            admin,
            project_contract_addr,
            user: expected_user,
            curator_list: Vec::new(),
            validator_list: Vec::new(),
        };
        let mut host = TestHost::new(state, state_builder);

        // create parameters
        let params = RemoveValidatorParam {
            addr: existing_user,
        };
        let params_byte = to_bytes(&params);
        ctx.set_parameter(&params_byte);

        // invoke method
        let result = contract_remove_validator(&ctx, &mut host);
        claim!(result.is_ok());
        let actual_state = host.state();
        claim_eq!(
            *actual_state,
            expected_state,
            "state has been changed unexpectedly..."
        );
    }

    #[concordium_test]
    /// Test that overlay-users.contract_remove_validator succeeds even if the parameter user is not
    /// validator
    fn test_contract_remove_validator_with_no_effect_invoked_by_admin() {
        let admin = AccountAddress([0; 32]);
        let project_contract_addr = ContractAddress::new(0, 0);
        let existing_user = AccountAddress([1; 32]);
        let not_validator = AccountAddress([2; 32]);
        let mut ctx = TestReceiveContext::empty();
        ctx.set_invoker(admin);
        // setup state
        let mut state_builder = TestStateBuilder::new();
        let mut user = state_builder.new_map();
        user.insert(
            existing_user,
            UserState {
                is_curator: false,
                is_validator: true,
                curated_projects: Vec::new(),
                validated_projects: Vec::new(),
            },
        );
        let state = State {
            admin,
            project_contract_addr,
            user,
            curator_list: Vec::new(),
            validator_list: vec![existing_user],
        };
        let mut expected_user = state_builder.new_map();
        expected_user.insert(
            existing_user,
            UserState {
                is_curator: false,
                is_validator: true,
                curated_projects: Vec::new(),
                validated_projects: Vec::new(),
            },
        );
        let expected_state = State {
            admin,
            project_contract_addr,
            user: expected_user,
            curator_list: Vec::new(),
            validator_list: vec![existing_user],
        };
        let mut host = TestHost::new(state, state_builder);

        // create parameters
        let params = RemoveValidatorParam {
            addr: not_validator,
        };
        let params_byte = to_bytes(&params);
        ctx.set_parameter(&params_byte);

        // invoke method
        let result = contract_remove_validator(&ctx, &mut host);
        claim!(result.is_ok());
        let actual_state = host.state();
        claim_eq!(
            *actual_state,
            expected_state,
            "state has been changed unexpectedly..."
        );
    }

    #[concordium_test]
    /// Test that overlay-users.contract_remove_validator was invoked by non-admin account.
    fn test_contract_remove_validator_invoked_by_non_admin() {
        let admin = AccountAddress([0; 32]);
        let suspicious = AccountAddress([1; 32]);

        let mut ctx = TestReceiveContext::empty();
        ctx.set_invoker(suspicious);
        let mut state_builder = TestStateBuilder::new();
        let state = State {
            admin,
            project_contract_addr: ContractAddress::new(0, 0),
            user: state_builder.new_map(),
            curator_list: Vec::new(),
            validator_list: Vec::new(),
        };
        let mut host = TestHost::new(state, state_builder);

        // create parameters
        let params = RemoveValidatorParam {
            addr: AccountAddress([2; 32]),
        };
        let params_byte = to_bytes(&params);
        ctx.set_parameter(&params_byte);

        // invoke method
        let result = contract_remove_validator(&ctx, &mut host);
        claim!(result.is_err());
        claim_eq!(result.err(), Some(Error::InvalidCaller));
    }

    #[concordium_test]
    /// Test that overlay-users.curate successfully add project id to user entry.
    fn test_contract_curate() {
        let admin = AccountAddress([0; 32]);
        let project_contract_addr = ContractAddress::new(0, 0);
        let existing_user = AccountAddress([1; 32]);
        let project_id: ProjectId = "TEST-PRJ".into();

        let mut ctx = TestReceiveContext::empty();
        ctx.set_sender(Address::Contract(project_contract_addr));
        let mut state_builder = TestStateBuilder::new();
        let mut user = state_builder.new_map();
        user.insert(
            existing_user,
            UserState {
                is_curator: true,
                is_validator: false,
                curated_projects: Vec::new(),
                validated_projects: Vec::new(),
            },
        );
        let state = State {
            admin,
            project_contract_addr,
            user,
            curator_list: vec![existing_user],
            validator_list: Vec::new(),
        };
        let mut expected_user = state_builder.new_map();
        expected_user.insert(
            existing_user,
            UserState {
                is_curator: true,
                is_validator: false,
                curated_projects: vec![project_id.clone()],
                validated_projects: Vec::new(),
            },
        );
        let expected_state = State {
            admin,
            project_contract_addr,
            user: expected_user,
            curator_list: vec![existing_user],
            validator_list: Vec::new(),
        };
        let mut host = TestHost::new(state, state_builder);

        // create parameters
        let params = CurateParam {
            addr: existing_user,
            project_id: project_id.clone(),
        };
        let params_byte = to_bytes(&params);
        ctx.set_parameter(&params_byte);

        // invoke method
        let result = contract_curate(&ctx, &mut host);
        claim!(result.is_ok());
        let actual_state = host.state();
        claim_eq!(
            *actual_state,
            expected_state,
            "state has been changed unexpectedly..."
        );
    }

    #[concordium_test]
    /// Test that overlay-users.curate fails if the input user has not been added as a curator.
    fn test_contract_curate_fails_with_no_user() {
        let admin = AccountAddress([0; 32]);
        let project_contract_addr = ContractAddress::new(0, 0);
        let existing_user = AccountAddress([1; 32]);
        let mut ctx = TestReceiveContext::empty();
        ctx.set_sender(Address::Contract(project_contract_addr));
        let mut state_builder = TestStateBuilder::new();
        let state = State {
            admin,
            project_contract_addr,
            user: state_builder.new_map(),
            curator_list: Vec::new(),
            validator_list: Vec::new(),
        };
        let mut host = TestHost::new(state, state_builder);

        // create parameters
        let params = CurateParam {
            addr: existing_user,
            project_id: "TEST-PRJ".into(),
        };
        let params_byte = to_bytes(&params);
        ctx.set_parameter(&params_byte);

        // invoke method
        let result = contract_curate(&ctx, &mut host);
        claim!(result.is_err());
        claim_eq!(result.err(), Some(Error::InvalidArgument));
    }

    #[concordium_test]
    /// Test that overlay-users.curate was invoked by non-project contract account.
    fn test_contract_curate_invoked_by_non_project_contract_addr() {
        let project_contract_addr = ContractAddress::new(0, 0);
        let suspicious = ContractAddress::new(0, 1);

        let mut ctx = TestReceiveContext::empty();
        ctx.set_sender(Address::Contract(suspicious));
        let mut state_builder = TestStateBuilder::new();
        let state = State {
            admin: AccountAddress([0; 32]),
            project_contract_addr,
            user: state_builder.new_map(),
            curator_list: Vec::new(),
            validator_list: Vec::new(),
        };
        let mut host = TestHost::new(state, state_builder);

        // create parameters
        let params = CurateParam {
            addr: AccountAddress([2; 32]),
            project_id: "TEST-PRJ".into(),
        };
        let params_byte = to_bytes(&params);
        ctx.set_parameter(&params_byte);

        // invoke method
        let result = contract_curate(&ctx, &mut host);
        claim!(result.is_err());
        claim_eq!(result.err(), Some(Error::InvalidCaller));
    }

    #[concordium_test]
    /// Test that overlay-users.validate successfully add project id to user entry.
    fn test_contract_validate() {
        let admin = AccountAddress([0; 32]);
        let project_contract_addr = ContractAddress::new(0, 0);
        let existing_user = AccountAddress([1; 32]);
        let project_id: ProjectId = "TEST-PRJ".into();

        let mut ctx = TestReceiveContext::empty();
        ctx.set_sender(Address::Contract(project_contract_addr));
        let mut state_builder = TestStateBuilder::new();
        let mut user = state_builder.new_map();
        user.insert(
            existing_user,
            UserState {
                is_curator: false,
                is_validator: true,
                curated_projects: Vec::new(),
                validated_projects: Vec::new(),
            },
        );
        let state = State {
            admin,
            project_contract_addr,
            user,
            curator_list: Vec::new(),
            validator_list: vec![existing_user],
        };
        let mut expected_user = state_builder.new_map();
        expected_user.insert(
            existing_user,
            UserState {
                is_curator: false,
                is_validator: true,
                curated_projects: Vec::new(),
                validated_projects: vec![project_id.clone()],
            },
        );
        let expected_state = State {
            admin,
            project_contract_addr,
            user: expected_user,
            curator_list: Vec::new(),
            validator_list: vec![existing_user],
        };
        let mut host = TestHost::new(state, state_builder);

        // create parameters
        let params = ValidateParam {
            addr: existing_user,
            project_id: project_id.clone(),
        };
        let params_byte = to_bytes(&params);
        ctx.set_parameter(&params_byte);

        // invoke method
        let result = contract_validate(&ctx, &mut host);
        claim!(result.is_ok());
        let actual_state = host.state();
        claim_eq!(
            *actual_state,
            expected_state,
            "state has been changed unexpectedly..."
        );
    }

    #[concordium_test]
    /// Test that overlay-users.validate fails if the input user has not been added as a validator.
    fn test_contract_validate_fails_with_no_user() {
        let admin = AccountAddress([0; 32]);
        let project_contract_addr = ContractAddress::new(0, 0);
        let existing_user = AccountAddress([1; 32]);
        let mut ctx = TestReceiveContext::empty();
        ctx.set_sender(Address::Contract(project_contract_addr));
        let mut state_builder = TestStateBuilder::new();
        let state = State {
            admin,
            project_contract_addr,
            user: state_builder.new_map(),
            curator_list: Vec::new(),
            validator_list: Vec::new(),
        };
        let mut host = TestHost::new(state, state_builder);

        // create parameters
        let params = ValidateParam {
            addr: existing_user,
            project_id: "TEST-PRJ".into(),
        };
        let params_byte = to_bytes(&params);
        ctx.set_parameter(&params_byte);

        // invoke method
        let result = contract_validate(&ctx, &mut host);
        claim!(result.is_err());
        claim_eq!(result.err(), Some(Error::InvalidArgument));
    }

    #[concordium_test]
    /// Test that overlay-users.validate was invoked by non-project contract account.
    fn test_contract_validate_invoked_by_non_project_contract_addr() {
        let project_contract_addr = ContractAddress::new(0, 0);
        let suspicious = ContractAddress::new(0, 1);

        let mut ctx = TestReceiveContext::empty();
        ctx.set_sender(Address::Contract(suspicious));
        let mut state_builder = TestStateBuilder::new();
        let state = State {
            admin: AccountAddress([0; 32]),
            project_contract_addr,
            user: state_builder.new_map(),
            curator_list: Vec::new(),
            validator_list: Vec::new(),
        };
        let mut host = TestHost::new(state, state_builder);

        // create parameters
        let params = ValidateParam {
            addr: AccountAddress([2; 32]),
            project_id: "TEST-PRJ".into(),
        };
        let params_byte = to_bytes(&params);
        ctx.set_parameter(&params_byte);

        // invoke method
        let result = contract_validate(&ctx, &mut host);
        claim!(result.is_err());
        claim_eq!(result.err(), Some(Error::InvalidCaller));
    }

    #[concordium_test]
    /// Test that overlay-users.upgrade can not be invoked by non-admin.
    fn test_contract_upgrade_invoked_by_non_admin() {
        let owner = AccountAddress([0; 32]);
        let suspicious = AccountAddress([1; 32]);
        let mut ctx = TestReceiveContext::empty();
        ctx.set_owner(owner);
        ctx.set_sender(Address::Account(suspicious));
        let mut state_builder = TestStateBuilder::new();
        let state = State {
            admin: owner,
            project_contract_addr: ContractAddress::new(0, 0),
            user: state_builder.new_map(),
            curator_list: Vec::new(),
            validator_list: Vec::new(),
        };
        let mut host = TestHost::new(state, state_builder);

        // create parameters
        let params = UpgradeParam {
            module: HashBytes::new([0; 32]),
            migrate: None,
        };
        let params_byte = to_bytes(&params);
        ctx.set_parameter(&params_byte);

        // invoke method
        let result = contract_upgrade(&ctx, &mut host);
        claim!(result.is_err());
    }

    #[concordium_test]
    /// Test that overlay-users.contract_view_admin returns administrative data.
    fn test_contract_view_admin_invoked_by_admin() {
        let admin = AccountAddress([0; 32]);
        let project_contract_addr = ContractAddress::new(1, 2);
        let curator = AccountAddress([1; 32]);
        let validator = AccountAddress([2; 32]);
        let mut ctx = TestReceiveContext::empty();
        ctx.set_invoker(admin);
        // setup state
        let mut state_builder = TestStateBuilder::new();
        let state = State {
            admin,
            project_contract_addr,
            user: state_builder.new_map(),
            curator_list: vec![curator],
            validator_list: vec![validator],
        };
        let mut host = TestHost::new(state, state_builder);

        // invoke method
        let result = contract_view_admin(&ctx, &mut host);
        claim!(result.is_ok());
        let view = result.unwrap();
        claim_eq!(view.admin, admin);
        claim_eq!(view.project_contract_addr, project_contract_addr);
        claim_eq!(view.curator_list, vec![curator]);
        claim_eq!(view.validator_list, vec![validator]);
    }

    #[concordium_test]
    /// Test that overlay-users.contract_view_admin should fail when invoked by non-admin
    fn test_contract_view_admin_invoked_by_not_admin() {
        let admin = AccountAddress([0; 32]);
        let suspicious = AccountAddress([1; 32]);
        let mut ctx = TestReceiveContext::empty();
        ctx.set_invoker(suspicious);
        // setup state
        let mut state_builder = TestStateBuilder::new();
        let state = State {
            admin,
            project_contract_addr: ContractAddress::new(1, 2),
            user: state_builder.new_map(),
            curator_list: Vec::new(),
            validator_list: Vec::new(),
        };
        let mut host = TestHost::new(state, state_builder);

        // invoke method
        let result = contract_view_admin(&ctx, &mut host);
        claim!(result.is_err());
        claim_eq!(result.err(), Some(Error::InvalidCaller));
    }

    #[concordium_test]
    /// Test that overlay-users.contract_view_user returns single user data.
    fn test_contract_view_user_for_existing_user() {
        let admin = AccountAddress([0; 32]);
        let existing_user = AccountAddress([1; 32]);
        let validated_project_id: ProjectId = "TEST-PRJ".into();
        let mut ctx = TestReceiveContext::empty();
        ctx.set_invoker(admin);
        // setup state
        let mut state_builder = TestStateBuilder::new();
        let mut user = state_builder.new_map();
        user.insert(
            existing_user,
            UserState {
                is_curator: false,
                is_validator: true,
                curated_projects: Vec::new(),
                validated_projects: vec![validated_project_id.clone()],
            },
        );
        let state = State {
            admin,
            project_contract_addr: ContractAddress::new(1, 2),
            user,
            curator_list: vec![],
            validator_list: vec![existing_user],
        };
        let mut host = TestHost::new(state, state_builder);

        // create parameters
        let params = AddrParam {
            addr: existing_user,
        };
        let params_byte = to_bytes(&params);
        ctx.set_parameter(&params_byte);

        // invoke method
        let result = contract_view_user(&ctx, &mut host);
        claim!(result.is_ok());
        let view = result.unwrap();
        claim!(!view.is_curator);
        claim!(view.is_validator);
        claim!(view.curated_projects.is_empty());
        claim_eq!(view.validated_projects, vec![validated_project_id]);
    }

    #[concordium_test]
    /// Test that overlay-users.contract_view_user returns default user data.
    fn test_contract_view_user_for_non_existing_user() {
        let admin = AccountAddress([0; 32]);
        let anyone = AccountAddress([100; 32]);
        let existing_user = AccountAddress([1; 32]);
        let non_existing_user = AccountAddress([2; 32]);
        let validated_project_id: ProjectId = "TEST-PRJ".into();
        let mut ctx = TestReceiveContext::empty();
        // anyone can call this contract function.
        ctx.set_invoker(anyone);
        // setup state
        let mut state_builder = TestStateBuilder::new();
        let mut user = state_builder.new_map();
        user.insert(
            existing_user,
            UserState {
                is_curator: false,
                is_validator: true,
                curated_projects: Vec::new(),
                validated_projects: vec![validated_project_id],
            },
        );
        let state = State {
            admin,
            project_contract_addr: ContractAddress::new(1, 2),
            user,
            curator_list: vec![],
            validator_list: vec![existing_user],
        };
        let mut host = TestHost::new(state, state_builder);

        // create parameters
        let params = AddrParam {
            addr: non_existing_user,
        };
        let params_byte = to_bytes(&params);
        ctx.set_parameter(&params_byte);

        // invoke method
        let result = contract_view_user(&ctx, &mut host);
        claim!(result.is_ok());
        let view = result.unwrap();
        claim!(!view.is_curator);
        claim!(!view.is_validator);
        claim!(view.curated_projects.is_empty());
        claim!(view.validated_projects.is_empty());
    }

    #[concordium_test]
    /// Test that overlay-users.contract_view_users returns all user data.
    fn test_contract_view_users() {
        let admin = AccountAddress([0; 32]);
        let anyone = AccountAddress([100; 32]);
        let existing_user1 = (
            AccountAddress([1; 32]),
            UserState {
                is_curator: false,
                is_validator: true,
                curated_projects: Vec::new(),
                validated_projects: vec!["TEST-PRJ1".into()],
            },
        );
        let existing_user2 = (
            AccountAddress([2; 32]),
            UserState {
                is_curator: true,
                is_validator: false,
                curated_projects: vec!["TEST-PRJ2".into()],
                validated_projects: Vec::new(),
            },
        );
        let mut ctx = TestReceiveContext::empty();
        // anyone can call this contract function.
        ctx.set_invoker(anyone);
        // setup state
        let mut state_builder = TestStateBuilder::new();
        let mut user = state_builder.new_map();
        user.insert(existing_user1.0, existing_user1.1.clone());
        user.insert(existing_user2.0, existing_user2.1.clone());
        let state = State {
            admin,
            project_contract_addr: ContractAddress::new(1, 2),
            user,
            curator_list: vec![existing_user2.0],
            validator_list: vec![existing_user1.0],
        };
        let mut host = TestHost::new(state, state_builder);

        // invoke method
        let result = contract_view_users(&ctx, &mut host);
        claim!(result.is_ok());
        let view = result.unwrap();
        claim_eq!(view.len(), 2);
        for (addr, state) in view {
            if addr == existing_user1.0 {
                claim_eq!(state, existing_user1.1.clone());
            } else if addr == existing_user2.0 {
                claim_eq!(state, existing_user2.1.clone());
            } else {
                fail!("unexpected user address returned...");
            }
        }
    }
}
