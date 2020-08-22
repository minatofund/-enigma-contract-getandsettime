use cosmwasm_std::{
    to_binary, Api, Binary, Env, Extern, HandleResponse, InitResponse, Querier, StdError,
    StdResult, Storage,
};

use crate::msg::{TimeResponse, CountResponse, HandleMsg, InitMsg, QueryMsg};
use crate::state::{config, config_read, State};
extern crate chrono;
use chrono::Local;

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    let state = State {
        time: msg.time,
        owner: env.message.sender,
    };

    config(&mut deps.storage).save(&state)?;

    Ok(InitResponse::default())
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    match msg {
        HandleMsg::SetTime {} => try_set_time(deps, env),
    }
}

pub fn try_set_time<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    _env: Env,
) -> StdResult<HandleResponse> {
    config(&mut deps.storage).update(|mut state| {
        state.time = Local::now().format("%Y-%m-%d|%H:%M");
        Ok(state)
    })?;

    Ok(HandleResponse::default())
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetTime {} => to_binary(&query_time(deps)?),
    }
}

fn query_time<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<TimeResponse> {
    let state = config_read(&deps.storage).load()?;
    Ok(TimeResponse { time: state.time })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env};
    use cosmwasm_std::{coins, from_binary, StdError};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies(20, &[]);

        let init_now = Local::now().format("%Y-%m-%d|%H:%M");
        let msg = InitMsg { time: init_now };
        let env = mock_env(&deps.api, "creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = init(&mut deps, env, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // it worked, let's query the state
        let res = query(&deps, QueryMsg::GetTime {}).unwrap();
        let value: TimeResponse = from_binary(&res).unwrap();
        assert_eq!(init_now, value.time);
    }

    #[test]
    fn set_time() {
        let mut deps = mock_dependencies(20, &coins(2, "token"));

        let init_now = Local::now().format("%Y-%m-%d|%H:%M");
        let msg = InitMsg { time: init_now };
        let env = mock_env(&deps.api, "creator", &coins(2, "token"));
        let _res = init(&mut deps, env, msg).unwrap();

        // beneficiary can release it
        let env = mock_env(&deps.api, "anyone", &coins(2, "token"));
        let msg = HandleMsg::SetTime {};
        let _res = handle(&mut deps, env, msg).unwrap();

        // should set time 
        let res = query(&deps, QueryMsg::GetTime {}).unwrap();
        let value: TimeResponse = from_binary(&res).unwrap();
        assert_eq!(init_now, value.time);
    }
}
