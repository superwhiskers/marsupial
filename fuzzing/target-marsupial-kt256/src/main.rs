use afl::fuzz;
use marsupial::KT256;

use fuzzing_utils::Input;

fn main() {
    fuzz!(|data: Input<'_>| {
        fuzzing_utils::exercise_hasher::<KT256>(data);
    });
}
