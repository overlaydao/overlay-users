#![cfg_attr(not(feature = "std"), no_std)]
use concordium_std::*;
use core::fmt::Debug;

#[derive(Serial, DeserialWithState)]
#[concordium(state_parameter = "S")]
struct State<S> {
    admin: AccountAddress,
    user: StateMap<AccountAddress, UserState, S>,
}

#[derive(Serial, Deserial)]
struct UserState {
    is_curator: bool,
    is_validator: bool,
}

#[derive(Serial, Deserial, SchemaType)]
struct TransferAdminParam {
    admin: AccountAddress,
}

#[derive(Serial, Deserial, SchemaType)]
struct AddrParam {
    addr: AccountAddress,
}

#[derive(Serial, Deserial, SchemaType)]
struct ViewAdminRes {
    admin: AccountAddress,
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
        user: state_builder.new_map()
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
    ensure!(ctx.sender() == Address::Account(state.admin), Error::InvalidCaller);

    state.admin = params.admin;
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
    ensure!(ctx.sender() == Address::Account(state.admin), Error::InvalidCaller);

    state.user.entry(params.addr).and_modify(|user_state| {
        user_state.is_curator = true;
    });
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
    ensure!(ctx.sender() == Address::Account(state.admin), Error::InvalidCaller);

    state.user.entry(params.addr).and_modify(|user_state| {
        user_state.is_curator = false;
    });
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
    ensure!(ctx.sender() == Address::Account(state.admin), Error::InvalidCaller);

    state.user.entry(params.addr).and_modify(|user_state| {
        user_state.is_validator = true;
    });
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
    ensure!(ctx.sender() == Address::Account(state.admin), Error::InvalidCaller);

    state.user.entry(params.addr).and_modify(|user_state| {
        user_state.is_validator = false;
    });
    Ok(())
}

#[receive(
    contract = "overlay-users",
    name = "view_admin",
    return_value = "ViewAdminRes"
)]
fn view_admin<S: HasStateApi>(
    ctx: &impl HasReceiveContext,
    host: &impl HasHost<State<S>, StateApiType = S>,
) -> ContractResult<ViewAdminRes> {
    let state = host.state();
    ensure!(ctx.sender() == Address::Account(state.admin), Error::InvalidCaller);
    Ok(ViewAdminRes {
        admin: state.admin,
    })
}

#[receive(
    contract = "overlay-users",
    name = "view_user",
    parameter = "AddrParam",
    return_value = "UserState"
)]
fn view_user<'a, S: HasStateApi + 'a>(
    ctx: &'a impl HasReceiveContext,
    host: &'a impl HasHost<State<S>, StateApiType = S>,
) -> ContractResult<StateRef<'a, UserState>> {
    let params: AddrParam = ctx.parameter_cursor().get()?;
    let state = host.state();
    let user_state = state.user.get(&params.addr).unwrap();
    Ok(user_state)
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
