use gpui::{App, AppContext, Entity, Global};

pub trait EntityWrapper<T> {
    fn new(inner: Entity<T>) -> Self;

    fn entity(&self) -> &Entity<T>;
}

pub fn global_set_entity<G: Global + EntityWrapper<T>, T: 'static>(
    cx: &mut App,
    value: T,
) {
    if cx.has_global::<G>() {
        let global = cx.global::<G>().entity().clone();

        global.update(cx, |entity, _cx| *entity = value);
    } else {
        let theme_set_state = G::new(cx.new(|_cx| value));

        cx.set_global::<G>(theme_set_state);
    }
}
