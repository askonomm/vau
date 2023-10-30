use vizia::prelude::*;

pub fn sidebar(cx: &mut Context) -> Handle<VStack> {
    VStack::new(cx, |cx| {
        Label::new(cx, "sidebar");
    })
}
