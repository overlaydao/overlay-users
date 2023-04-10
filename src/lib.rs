#![cfg_attr(all(not(feature = "std"), not(test)), no_std)]
use concordium_std::*;
use core::fmt::Debug;

type ProjectId = String;

#[derive(Serial, DeserialWithState, StateClone)]
#[concordium(state_parameter = "S")]
struct State<S> {
    admin: AccountAddress,
    project_contract_addr: ContractAddress,
    user: StateMap<AccountAddress, UserState, S>,
    curator_list: Vec<AccountAddress>,
    validator_list: Vec<AccountAddress>,
}

#[derive(Serial, Deserial, SchemaType, Clone)]
struct UserState {
    is_curator: bool,
    is_validator: bool,
    curated_projects: Vec<ProjectId>,
    validated_projects: Vec<ProjectId>,
}

#[derive(Serial, Deserial, SchemaType)]
struct TransferAdminParam {
    admin: AccountAddress,
}

#[derive(Serial, Deserial, SchemaType)]
struct AddProjectContractParam {
    project_contract_addr: ContractAddress,
}

#[derive(Serial, Deserial, SchemaType)]
struct AddrParam {
    addr: AccountAddress,
}

#[derive(Serial, Deserial, SchemaType)]
struct CurateParam {
    addr: AccountAddress,
    project_id: ProjectId,
}

#[derive(Serial, Deserial, SchemaType)]
struct ValidateParam {
    addr: AccountAddress,
    project_id: ProjectId,
}

#[derive(Debug, Serialize, SchemaType)]
struct UpgradeParam {
    module: ModuleReference,
    migrate: Option<(OwnedEntrypointName, OwnedParameter)>,
}

#[derive(Serial, Deserial, SchemaType)]
struct ViewAdminRes {
    admin: AccountAddress,
    project_contract_addr: ContractAddress,
    curator_list: Vec<AccountAddress>,
    validator_list: Vec<AccountAddress>,
}

#[derive(Debug, PartialEq, Eq, Reject, Serial, SchemaType)]
enum Error {
    #[from(ParseError)]
    ParseParamsError,
    InvalidCaller,
}

type ContractResult<A> = Result<A, Error>;

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

#[receive(
    contract = "overlay-users",
    name = "add_curator",
    parameter = "AddrParam",
    mutable
)]
fn contract_add_curator<S: HasStateApi>(
    ctx: &impl HasReceiveContext,
    host: &mut impl HasHost<State<S>>,
) -> ContractResult<()> {
    let params: AddrParam = ctx.parameter_cursor().get()?;
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

    // TODO duplicate check (should push only when the address is not included).
    state.curator_list.push(params.addr);
    Ok(())
}

#[receive(
    contract = "overlay-users",
    name = "remove_curator",
    parameter = "AddrParam",
    mutable
)]
fn contract_remove_curator<S: HasStateApi>(
    ctx: &impl HasReceiveContext,
    host: &mut impl HasHost<State<S>>,
) -> ContractResult<()> {
    let params: AddrParam = ctx.parameter_cursor().get()?;
    let state = host.state_mut();
    ensure!(ctx.invoker() == state.admin, Error::InvalidCaller);

    state.user.entry(params.addr).and_modify(|user_state| {
        user_state.is_curator = false;
    });
    state.curator_list.retain(|x| *x != params.addr);
    Ok(())
}

#[receive(
    contract = "overlay-users",
    name = "add_validator",
    parameter = "AddrParam",
    mutable
)]
fn contract_add_validator<S: HasStateApi>(
    ctx: &impl HasReceiveContext,
    host: &mut impl HasHost<State<S>>,
) -> ContractResult<()> {
    let params: AddrParam = ctx.parameter_cursor().get()?;
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

    // TODO duplicate check (should push only when the address is not included).
    state.validator_list.push(params.addr);
    Ok(())
}

#[receive(
    contract = "overlay-users",
    name = "remove_validator",
    parameter = "AddrParam",
    mutable
)]
fn contract_remove_validator<S: HasStateApi>(
    ctx: &impl HasReceiveContext,
    host: &mut impl HasHost<State<S>>,
) -> ContractResult<()> {
    let params: AddrParam = ctx.parameter_cursor().get()?;
    let state = host.state_mut();
    ensure!(ctx.invoker() == state.admin, Error::InvalidCaller);

    state.user.entry(params.addr).and_modify(|user_state| {
        user_state.is_validator = false;
    });
    state.validator_list.retain(|x| *x != params.addr);
    Ok(())
}

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

    state.user.entry(params.addr).and_modify(|user_state| {
        // TODO is it OK to add project id for non_curator?
        // TODO confirm it's ok there saved duplicated project_id...
        user_state.curated_projects.push(params.project_id);
    });
    Ok(())
}

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

    state.user.entry(params.addr).and_modify(|user_state| {
        // TODO confirm it's ok there saved duplicated project_id...
        user_state.validated_projects.push(params.project_id);
    });
    Ok(())
}

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

#[receive(
    contract = "overlay-users",
    name = "view_user",
    parameter = "AddrParam",
    return_value = "UserState"
)]
fn contract_view_user<S: HasStateApi>(
    ctx: &impl HasReceiveContext,
    host: &impl HasHost<State<S>, StateApiType = S>,
) -> ContractResult<UserState> {
    let params: AddrParam = ctx.parameter_cursor().get()?;
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

type ViewUsersResponse = Vec<(AccountAddress, UserState)>;
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

#[concordium_cfg_test]
/// implements Debug for State inside test functions.
/// this implementation will be build only when `concordium-std/wasm-test` feature is active.
/// (e.g. when launched by `cargo concordium test`)
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

#[concordium_cfg_test]
/// implements PartialEq for `claim_eq` inside test functions.
/// this implementation will be build only when `concordium-std/wasm-test` feature is active.
/// (e.g. when launched by `cargo concordium test`)
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

#[concordium_cfg_test]
/// implements Debug for UserState inside test functions.
/// this implementation will be build only when `concordium-std/wasm-test` feature is active.
/// (e.g. when launched by `cargo concordium test`)
impl Debug for UserState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "is_curator: {}, is_validator: {}, curated_projects: {:?}, validated_projects: {:?}",
            self.is_curator, self.is_validator, self.curated_projects, self.validated_projects
        )
    }
}

#[concordium_cfg_test]
/// implements PartialEq for `claim_eq` inside test functions.
/// this implementation will be build only when `concordium-std/wasm-test` feature is active.
/// (e.g. when launched by `cargo concordium test`)
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

        // execute init
        let result = contract_init(&ctx, &mut state_builder);

        // check init result
        claim!(result.is_ok());
        let state = result.unwrap();
        claim_eq!(state.admin, invoker);
        claim_eq!(
            state.project_contract_addr,
            ContractAddress::new(0u64, 0u64)
        );
        claim!(state.user.is_empty());
        claim!(state.curator_list.is_empty());
        claim!(state.validator_list.is_empty());
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
        let params = AddrParam { addr: curator };
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
        let params = AddrParam {
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
        let params = AddrParam {
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
        let params = AddrParam {
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
        let params = AddrParam { addr: not_curator };
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
        let params = AddrParam {
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
        let params = AddrParam { addr: validator };
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
        let params = AddrParam {
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
        let params = AddrParam {
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
        let params = AddrParam {
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
        let params = AddrParam {
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
        let params = AddrParam {
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
    /// Test that overlay-users.curate succeed even if the input user has not been entried.
    fn test_contract_curate_with_no_effect() {
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
        let expected_state = State {
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
        claim!(result.is_ok());
        let actual_state = host.state();
        claim_eq!(
            *actual_state,
            expected_state,
            "state has been changed unexpectedly..."
        );
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
    /// Test that overlay-users.validate succeed even if the input user has not been entried.
    fn test_contract_validate_with_no_effect() {
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
        let expected_state = State {
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
        claim!(result.is_ok());
        let actual_state = host.state();
        claim_eq!(
            *actual_state,
            expected_state,
            "state has been changed unexpectedly..."
        );
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
