use afl::fuzz;
use marsupial::KT128;

use fuzzing_utils::Input;

fn main() {
    fuzz!(|data: Input<'_>| {
        fuzzing_utils::exercise_hasher::<KT128>(data);
    });
}
