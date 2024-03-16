use rokugo_common::color::ColoredDisplay;
use rokugo_mir::emit::{content::MirContent, emitter::MirEmitter};
use termcolor::{ColorChoice, StandardStream};

#[test]
fn colored_display() {
    let mut mir = MirEmitter::new();
    let int = mir.meta_span(0..3).define_int32(65);
    mir.return_value(int);

    let mir: MirContent = mir.into();
    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    mir.fmt_with_color(&mut stdout).unwrap();
}
