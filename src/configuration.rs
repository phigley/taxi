

#[derive(Deserialize, Debug)]
pub enum ReplayMode {
    None,
    First,
    FirstSuccess,
    FirstFailure,
}

#[derive(Deserialize, Debug)]
#[serde(default)]
pub struct Configuration {
    pub initial_state: String,
    pub max_steps: u32,
    pub trials: u32,
    pub replay_mode: ReplayMode,
}

impl Default for Configuration {
    fn default() -> Configuration {
        let initial_state = "\
        ┌───┬─────┐\n\
        │d .│. . .│\n\
        │   │     │\n\
        │. .│. . .│\n\
        │         │\n\
        │. . t . .│\n\
        │         │\n\
        │.│. .│. .│\n\
        │ │   │   │\n\
        │.│. .│p .│\n\
        └─┴───┴───┘\n\
        ";

        Configuration {
            initial_state: String::from(initial_state),
            trials: 1,
            max_steps: 500,
            replay_mode: ReplayMode::First,
        }
    }
}
