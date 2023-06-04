use nih_plug::prelude::*;

use clip::Clip;

fn main() {
    nih_export_standalone::<Clip>();
    //let mut clip = Clip::default();
    // clip.initialize(audio_io_layout, buffer_config, context)
    // clip.editor().unwrap().spawn(parent, context)
    // loop {

    //     clip.process(buffer, aux, context)
    // }
}
