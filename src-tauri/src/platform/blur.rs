//! Standard compositor blur on the GTK-owned Wayland connection and surface.
use gtk::prelude::*;
use wayland_client::{
    Connection, Dispatch, EventQueue, Proxy, QueueHandle, WEnum,
    backend::{Backend, ObjectId},
    protocol::{
        wl_compositor::WlCompositor, wl_region::WlRegion, wl_registry::WlRegistry,
        wl_surface::WlSurface,
    },
};
use wayland_protocols::ext::background_effect::v1::client::{
    ext_background_effect_manager_v1::{self, ExtBackgroundEffectManagerV1},
    ext_background_effect_surface_v1::ExtBackgroundEffectSurfaceV1,
};

#[derive(Default)]
struct State {
    supported: bool,
    manager: Option<ExtBackgroundEffectManagerV1>,
    compositor: Option<WlCompositor>,
    manager_name: Option<u32>,
}

impl Dispatch<WlRegistry, ()> for State {
    fn event(
        state: &mut Self,
        registry: &WlRegistry,
        event: wayland_client::protocol::wl_registry::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        use wayland_client::protocol::wl_registry::Event;
        match event {
            Event::Global {
                name, interface, ..
            } if interface == "ext_background_effect_manager_v1" => {
                state.manager = Some(registry.bind(name, 1, qh, ()));
                state.manager_name = Some(name);
            }
            Event::Global {
                name, interface, ..
            } if interface == "wl_compositor" => {
                state.compositor = Some(registry.bind(name, 1, qh, ()));
            }
            Event::GlobalRemove { name } if state.manager_name == Some(name) => {
                state.supported = false
            }
            _ => {}
        }
    }
}
impl Dispatch<ExtBackgroundEffectManagerV1, ()> for State {
    fn event(
        state: &mut Self,
        _: &ExtBackgroundEffectManagerV1,
        event: ext_background_effect_manager_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let ext_background_effect_manager_v1::Event::Capabilities { flags } = event {
            state.supported = matches!(flags, WEnum::Value(flags) if flags.contains(ext_background_effect_manager_v1::Capability::Blur));
        }
    }
}
wayland_client::delegate_noop!(State: ignore WlCompositor);
wayland_client::delegate_noop!(State: ignore WlRegion);
wayland_client::delegate_noop!(State: ignore ExtBackgroundEffectSurfaceV1);

pub(super) struct Blur {
    queue: EventQueue<State>,
    state: State,
    effect: ExtBackgroundEffectSurfaceV1,
    enabled: bool,
    _registry: WlRegistry,
    // GDK owns the raw pointers; keep it alive until our proxies are destroyed.
    native: gtk::ApplicationWindow,
}
impl Blur {
    pub(super) fn new(native: &gtk::ApplicationWindow) -> Option<Self> {
        let display = native.display();
        if !display.type_().name().contains("Wayland") {
            return None;
        }
        let surface = native.window()?;
        // These live GDK objects stay on GTK's main thread. The guest backend does not
        // disconnect the display or take ownership of GTK's wl_surface.
        let connection = unsafe {
            let raw = gdk_wayland_sys::gdk_wayland_display_get_wl_display(display.as_ptr().cast());
            if raw.is_null() {
                return None;
            }
            Connection::from_backend(Backend::from_foreign_display(raw.cast()))
        };
        let mut queue = connection.new_event_queue();
        let registry = connection.display().get_registry(&queue.handle(), ());
        let mut state = State::default();
        queue.roundtrip(&mut state).ok()?;
        queue.roundtrip(&mut state).ok()?;
        let raw_surface =
            unsafe { gdk_wayland_sys::gdk_wayland_window_get_wl_surface(surface.as_ptr().cast()) };
        if raw_surface.is_null() {
            return None;
        }
        let id = unsafe { ObjectId::from_ptr(WlSurface::interface(), raw_surface.cast()) }.ok()?;
        let wl_surface = WlSurface::from_id(&connection, id).ok()?;
        let effect =
            state
                .manager
                .as_ref()?
                .get_background_effect(&wl_surface, &queue.handle(), ());
        connection.flush().ok()?;
        Some(Self {
            queue,
            state,
            effect,
            enabled: false,
            _registry: registry,
            native: native.clone(),
        })
    }
    pub(super) fn poll(&mut self) -> bool {
        let previous = self.state.supported;
        if self.queue.dispatch_pending(&mut self.state).is_err() {
            self.state.supported = false;
        }
        previous != self.state.supported
    }
    pub(super) fn apply(&mut self, allowed: bool) -> bool {
        let enabled = allowed && self.state.supported;
        if enabled != self.enabled {
            if enabled {
                let Some(compositor) = &self.state.compositor else {
                    return false;
                };
                let region = compositor.create_region(&self.queue.handle(), ());
                region.add(0, 0, i32::MAX, i32::MAX);
                self.effect.set_blur_region(Some(&region));
                region.destroy();
            } else {
                self.effect.set_blur_region(None);
            }
            if self.queue.flush().is_err() {
                self.state.supported = false;
                return false;
            }
            self.enabled = enabled;
            self.native.queue_draw();
        }
        enabled
    }
}
impl Drop for Blur {
    fn drop(&mut self) {
        self.effect.destroy();
        if let Some(manager) = &self.state.manager {
            manager.destroy();
        }
        let _ = self.queue.flush();
    }
}
