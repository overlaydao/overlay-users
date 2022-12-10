use concordium_std::*;
use core::fmt::Debug;

#[derive(Serial, DeserialWithState)]
#[concordium(state_parameter = "S")]
struct State<S> {
    admin: AccountAddress,
    user: StateMap<AccountAddress, UserState<S>, S>,
}

#[derive(Serial, Deserial]
struct UserState {
    is_curator: bool,
    is_validator: bool,
}

#[derive(Serial, Deserial, SchemaType)]
struct AddrParam {
    addr: AccountAddress,
}

#[derive(Debug, PartialEq, Eq, Reject, Serial, SchemaType)]
enum Error {
    #[from(ParseError)]
    ParseParamsError,
    InvalidCaller,
}

#[init(contract = "overlay-users")]
fn contract_init<S: HasStateApi>(
    _ctx: &impl HasInitContext,
    _state_builder: &mut StateBuilder<S>,
) -> InitResult<State> {
    let state = State {
        admin: ctx.sender(),
        user: state_builder.new_map()
    };
    Ok(state);
}

#[receive(
    contract = "overlay-users",
    name = "add_curator",
    parameter = "AddrParam"
    mutable
)]
fn contract_add_curator<S: HasStateApi>(
    ctx: &impl HasReceiveContext,
    host: &mut impl HasHost<State>,
) -> ReceiveResult<()> {
    let params: AddrParam = ctx.parameter_cursor().get()?;
    let state = host.state_mut();
    let old_values = state.user.get(params.addr);
    ensure!(ctx.sender() == state.admin, Error::InvalidCaller);

    state.user.insert(
        params.addr,
        UserState {
            is_curator: true,
            is_validator: old_values.is_validator,
        }
    );
}

#[receive(
    contract = "overlay-users",
    name = "remove_curator",
    parameter = "AddrParam",
    mutable
)]
fn contract_remove_curator<S: HasStateApi>(
    ctx: &impl HasReceiveContext,
    host: &mut impl HasHost<State>,
) -> ReceiveResult<()> {
    let params: AddrParam = ctx.parameter_cursor().get()?;
    let state = host.state_mut();
    let old_values = state.user.get(params.addr);
    ensure!(ctx.sender() == state.admin, Error::InvalidCaller);


    state.user.insert(
        params.addr,
        UserState {
            is_curator: false,
            is_validator: old_values.is_validator,
        }
    );
}

#[receive(
    contract = "overlay-users",
    name = "add_validator",
    parameter = "AddrParam",
    mutable
)]
fn contract_add_validator<S: HasStateApi>(
    ctx: &impl HasReceiveContext,
    host: &mut impl HasHost<State>,
) -> ReceiveResult<()> {
    let params: AddrParam = ctx.parameter_cursor().get()?;
    let state = host.state_mut();
    let old_values = state.user.get(params.addr);
    ensure!(ctx.sender() == state.admin, Error::InvalidCaller);

    state.user.insert(
        params.addr,
        UserState {
            is_curator: old_values.is_curator,
            is_validator: true,
        }
    );
}

#[receive(
    contract = "overlay-users",
    name = "remove_validator",
    parameter = "AddrParam",
    mutable
)]
fn contract_remove_validator<S: HasStateApi>(
    ctx: &impl HasReceiveContext,
    host: &mut impl HasHost<State>,
) -> ReceiveResult<()> {
    let params: AddrParam = ctx.parameter_cursor().get()?;
    let state = host.state_mut();
    let old_values = state.user.get(params.addr);
    ensure!(ctx.sender() == state.admin, Error::InvalidCaller);


    state.user.insert(
        params.addr,
        UserState {
            is_curator: old_values.is_curator,
            is_validator: false,
        }
    );
}

#[receive(
    contract = "overlay-users",
    name = "view_admin",
    return_value = "State"
)]
fn view_admin<S: HasStateApi>(
    ctx: &impl HasReceiveContext,
    host: &impl HasHost<State<S>, StateApiType = S>,
) -> ReceiveResult<State> {
    ensure!(ctx.sender == state.admin, Error::InvalidCaller);
    let state = host.state();
    Ok(State);
}

#[receive(
    contract = "overlay-users",
    name = "view_user",
    parameter = "AddrParam",
    return_value = "UserState"
)]
fn view_user<S: HasStateApi>(
    ctx: &impl HasReceiveContext,
    host: &impl HasHost<State<S>, StateApiType = S>,
) -> ReceiveResult<UserState> {
    let params: AddrParam = ctx.parameter_cursor().get()?;
    let state = host.state();
    let user_state = state.user.get(params.addr);
    Ok(user_state);
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
