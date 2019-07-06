mod seam_carving;
mod site;

use yew::prelude::*;

fn main() {
    yew::initialize();
    App::<site::Model>::new().mount_to_body();
    yew::run_loop();
}
