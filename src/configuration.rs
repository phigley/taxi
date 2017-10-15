

#[derive(Deserialize, Debug)]
#[serde(default)]
pub struct Configuration {
    pub initial_state: String,
    pub max_steps: u32,
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
            max_steps: 500,
        }
    }
}
