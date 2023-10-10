use std::collections::HashMap;

use crate::sal_enums::State;

/// Compute the commands required to bring a component from an initial state
/// to a desired final state.
///
/// # Panic
///
/// If `desired_state` is invalid, e.g. Fault or Invalid.
/// If `initial_state` is invalid.
pub fn compute_state_transitions(
    initial_state: State,
    desired_state: State,
) -> Option<Vec<String>> {
    let ordered_states = [
        State::Offline,
        State::Standby,
        State::Disabled,
        State::Enabled,
    ];

    assert!(
        desired_state != State::Fault && desired_state != State::Invalid,
        "Invalid desired state {desired_state:?}. Must be one of {ordered_states:?}."
    );

    assert_ne!(
        initial_state,
        State::Invalid,
        "Invalid initial state. Must be one of {ordered_states:?} or Fault.",
    );

    // If the initial state is the same as the desired state we can return
    // already with an empty string.
    if initial_state == desired_state {
        return None;
    }

    let mut state_transitions: Vec<String> = Vec::new();

    let current_state = if initial_state == State::Fault {
        state_transitions.push("command_standby".to_string());
        State::Standby
    } else {
        initial_state
    };

    let basic_state_transition_commands: HashMap<(State, State), String> = HashMap::from([
        (
            (State::Offline, State::Standby),
            "command_enterControl".to_string(),
        ),
        (
            (State::Standby, State::Disabled),
            "command_start".to_string(),
        ),
        (
            (State::Disabled, State::Enabled),
            "command_enable".to_string(),
        ),
        (
            (State::Enabled, State::Disabled),
            "command_disable".to_string(),
        ),
        (
            (State::Disabled, State::Standby),
            "command_standby".to_string(),
        ),
        (
            (State::Standby, State::Offline),
            "command_exitControl".to_string(),
        ),
    ]);

    if let Some(current_state_position) = ordered_states
        .iter()
        .position(|state| *state == current_state)
    {
        if let Some(desired_state_position) = ordered_states
            .iter()
            .position(|state| *state == desired_state)
        {
            let index_range: Vec<usize> = if desired_state_position > current_state_position {
                (current_state_position..desired_state_position)
                    .collect()
            } else {
                (desired_state_position + 1..current_state_position + 1)
                    .rev()
                    .collect()
            };
            let offset = if desired_state_position > current_state_position {
                1
            } else {
                -1
            };

            let mut additional_transitions: Vec<String> = index_range
                .into_iter()
                .map(|index| {
                    let transition = (
                        ordered_states[index],
                        ordered_states[(index as isize + offset) as usize],
                    );
                    basic_state_transition_commands
                        .get(&transition)
                        .unwrap()
                        .clone()
                })
                .collect();
            state_transitions.append(&mut additional_transitions);

            Some(state_transitions.to_owned())
        } else {
            None
        }
    } else {
        None
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    #[should_panic(expected = "Invalid desired state")]
    fn compute_state_transitions_desired_state_invalid() {
        compute_state_transitions(State::Standby, State::Invalid);
    }

    #[test]
    #[should_panic(expected = "Invalid desired state")]
    fn compute_state_transitions_desired_state_fault() {
        compute_state_transitions(State::Standby, State::Fault);
    }

    #[test]
    #[should_panic(expected = "Invalid initial state.")]
    fn compute_state_transitions_initial_state_invalid() {
        compute_state_transitions(State::Invalid, State::Enabled);
    }

    #[test]
    fn compute_state_transitions_fault_enabled() {
        let state_transitions = compute_state_transitions(State::Fault, State::Enabled).unwrap();

        for command in [
            "command_standby".to_string(),
            "command_start".to_string(),
            "command_enable".to_string(),
        ] {
            assert!(
                state_transitions.contains(&command),
                "{command} not in {state_transitions:?}"
            );
        }
    }

    #[test]
    fn compute_state_transitions_standby_enabled() {
        let state_transitions = compute_state_transitions(State::Standby, State::Enabled).unwrap();

        for command in ["command_start".to_string(), "command_enable".to_string()] {
            assert!(
                state_transitions.contains(&command),
                "{command} not in {state_transitions:?}"
            );
        }
    }

    #[test]
    fn compute_state_transitions_enabled_standby() {
        let state_transitions = compute_state_transitions(State::Enabled, State::Standby).unwrap();

        for command in ["command_disable".to_string(), "command_standby".to_string()] {
            assert!(
                state_transitions.contains(&command),
                "{command} not in {state_transitions:?}"
            );
        }
    }

    #[test]
    fn compute_state_transitions_enabled_offline() {
        let state_transitions = compute_state_transitions(State::Enabled, State::Offline).unwrap();

        for command in [
            "command_disable".to_string(),
            "command_standby".to_string(),
            "command_exitControl".to_string(),
        ] {
            assert!(
                state_transitions.contains(&command),
                "{command} not in {state_transitions:?}"
            );
        }
    }
}
