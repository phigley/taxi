

#[derive(Deserialize, Debug)]
pub enum ReplayMode {
    None,
    First,
    FirstSuccess,
    FirstFailure,
}

#[derive(Deserialize, Debug)]
pub struct InitialState {
    pub taxi_pos: (i32, i32),
    pub passenger_loc: char,
    pub destination_loc: char,
}

#[derive(Deserialize, Debug)]
#[serde(default)]
pub struct Configuration {
    pub world: String,
    pub initial_states: Vec<InitialState>,
    pub max_steps: u32,
    pub trials: u32,
    pub replay_mode: ReplayMode,
}

impl Default for Configuration {
    fn default() -> Configuration {
        let world_str = "\
        ┌───┬─────┐\n\
        │R .│. . G│\n\
        │   │     │\n\
        │. .│. . .│\n\
        │         │\n\
        │. . . . .│\n\
        │         │\n\
        │.│. .│. .│\n\
        │ │   │   │\n\
        │Y│. .│B .│\n\
        └─┴───┴───┘\n\
        ";

        Configuration {
            world: String::from(world_str),
            initial_states: vec![],
            trials: 1,
            max_steps: 500,
            replay_mode: ReplayMode::First,
        }
    }
}
