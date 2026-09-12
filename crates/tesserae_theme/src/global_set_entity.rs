use gpui::{App, AppContext, Entity, Global};

pub trait EntityWrapper<Value> {
    fn new(inner: Entity<Value>) -> Self;

    fn entity(&self) -> &Entity<Value>;
}

pub fn global_set_entity<GlobalType, Value>(cx: &mut App, value: Value)
where
    GlobalType: Global + EntityWrapper<Value>,
    Value: 'static,
{
    if cx.has_global::<GlobalType>() {
        let global = cx.global::<GlobalType>().entity().clone();

        global.update(cx, |entity, _app| *entity = value);

        return;
    }

    let global_state = GlobalType::new(cx.new(|_app| value));

    cx.set_global::<GlobalType>(global_state);
}
