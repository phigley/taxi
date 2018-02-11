use state::State;
use world::World;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Attribute {
    TaxiX,
    TaxiY,
    Passenger,
}

#[derive(Debug)]
pub enum Effect {
    Add(i32),
    Assign(Option<char>),
}

impl Effect {
    pub fn generate_effects(
        attribute: Attribute,
        world: &World,
        old_state: &State,
        new_state: &State,
    ) -> Vec<Effect> {
        let mut result = Vec::new();

        match attribute {
            Attribute::TaxiX => {
                let old_x = old_state.get_taxi().x;
                let new_x = new_state.get_taxi().x;

                if old_x != new_x {
                    let delta = new_x - old_x;
                    result.push(Effect::Add(delta));
                }
            }

            Attribute::TaxiY => {
                let old_y = old_state.get_taxi().y;
                let new_y = new_state.get_taxi().y;

                if old_y != new_y {
                    let delta = new_y - old_y;
                    result.push(Effect::Add(delta));
                }
            }

            Attribute::Passenger => {
                let old_passenger = old_state.get_passenger();
                let new_passenger = new_state.get_passenger();

                if old_passenger != new_passenger {
                    result.push(Effect::Assign(new_passenger));
                }
            }
        }

        result
    }
}
