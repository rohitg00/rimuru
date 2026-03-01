pub mod api;
pub mod schedules;

use iii_sdk::III;

pub fn register_all(iii: &III) {
    api::register(iii);
    schedules::register(iii);
}
