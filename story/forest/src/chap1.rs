use ifengine::elements::*;
use ifengine::ifview;

use crate::State;

#[ifview]
pub fn p1(_s: &mut State) {
    ps!(
        "",
        link!("North lies the salt wrack. North is where you will go.", p2)
    );
}

#[ifview]
pub fn p2(_s: &mut State) {
    ps!(
        "If summer meant heat, this ground would thaw. The thin soil would flourish; the wastes beyond the city would be green.",
        "You gaze out to the east, past the old ornate window of the Observational Society. The sun glares on dead white. South, the city's low familiar skyline begins, the buildings hunched as though they fear the sky. Soon you will leave this place.",
        link!("North lies the salt wrack. North is where you will go.", p2)
    );
}
