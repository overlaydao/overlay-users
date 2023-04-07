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

    let mut user_state = state.user.entry(params.addr).or_insert_with(|| UserState {
        is_curator: true,
        is_validator: false,
        curated_projects: Vec::new(),
        validated_projects: Vec::new(),
    });

    user_state.is_curator = true;

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

    let mut user_state = state.user.entry(params.addr).or_insert_with(|| UserState {
        is_curator: false,
        is_validator: true,
        curated_projects: Vec::new(),
        validated_projects: Vec::new(),
    });

    user_state.is_validator = true;

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
    let user_state_ref = state.user.get(&params.addr);
    let user_state: UserState = match user_state_ref {
        None => UserState {
            is_curator: false,
            is_validator: false,
            curated_projects: Vec::new(),
            validated_projects: Vec::new(),
        },
        Some(_) => user_state_ref.unwrap().clone(),
    };
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
    let mut users_response = Vec::new();
    for (account_address, user_state) in users.iter() {
        users_response.push((account_address.clone(), user_state.clone()));
    }

    Ok(users_response)
}

#[concordium_cfg_test]
mod tests {
    use super::*;
    use test_infrastructure::*;

    #[concordium_test]
    /// Test that init succeeds.
    fn test_init() {
        // invoker will be an admin
        let invoker = AccountAddress([0; 32]);
        let mut ctx = TestInitContext::empty();
        ctx.set_init_origin(invoker.clone());

        let mut state_builder = TestStateBuilder::new();

        // execute init
        let result = contract_init(&ctx, &mut state_builder);

        // check init result
        claim!(result.is_ok());
        let state = result.unwrap();
        claim_eq!(state.admin, invoker);
        claim_eq!(
            state.project_contract_addr,
            ContractAddress::new(0u64, 0u64),
        );
        claim!(state.user.is_empty());
        claim!(state.curator_list.is_empty());
        claim!(state.validator_list.is_empty());
    }
}
