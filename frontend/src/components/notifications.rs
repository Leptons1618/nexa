//! Toast notification manager — global, auto-dismissing notification system.
//!
//! Usage:
//! 1. Call `use_context_provider` with `NotificationState::new()` at root.
//! 2. Render `<NotificationContainer />` once in your layout.
//! 3. From any component: `use_notifications().push(...)` to show a toast.

use dioxus::prelude::*;

/// The kind of notification.
#[derive(Clone, Debug, PartialEq)]
#[allow(dead_code)]
pub enum NotificationType {
    Success,
    Error,
    Info,
    Warning,
}

/// A single notification toast.
#[derive(Clone, Debug, PartialEq)]
pub struct Notification {
    pub id: u64,
    pub kind: NotificationType,
    pub title: String,
    pub message: String,
    /// Auto-dismiss duration in milliseconds (0 = manual only).
    pub duration_ms: u64,
}

/// Shared state for the notification system.
#[derive(Clone, Copy)]
pub struct NotificationState {
    pub items: Signal<Vec<Notification>>,
    counter: Signal<u64>,
}

impl NotificationState {
    pub fn new() -> Self {
        Self {
            items: Signal::new(Vec::new()),
            counter: Signal::new(0),
        }
    }

    /// Push a new notification and return its ID.
    pub fn push(&mut self, kind: NotificationType, title: impl Into<String>, message: impl Into<String>) -> u64 {
        let id = {
            let mut c = self.counter;
            let v = *c.read() + 1;
            c.set(v);
            v
        };
        let duration_ms = match kind {
            NotificationType::Error => 8000,
            NotificationType::Warning => 6000,
            _ => 4000,
        };
        let notif = Notification {
            id,
            kind,
            title: title.into(),
            message: message.into(),
            duration_ms,
        };
        self.items.write().push(notif);
        id
    }

    /// Push with a custom duration.
    #[allow(dead_code)]
    pub fn push_with_duration(
        &mut self,
        kind: NotificationType,
        title: impl Into<String>,
        message: impl Into<String>,
        duration_ms: u64,
    ) -> u64 {
        let id = {
            let mut c = self.counter;
            let v = *c.read() + 1;
            c.set(v);
            v
        };
        let notif = Notification {
            id,
            kind,
            title: title.into(),
            message: message.into(),
            duration_ms,
        };
        self.items.write().push(notif);
        id
    }

    /// Remove a notification by ID.
    pub fn dismiss(&mut self, id: u64) {
        self.items.write().retain(|n| n.id != id);
    }

    /// Convenience: push a success notification.
    pub fn success(&mut self, title: impl Into<String>, message: impl Into<String>) -> u64 {
        self.push(NotificationType::Success, title, message)
    }

    /// Convenience: push an error notification.
    pub fn error(&mut self, title: impl Into<String>, message: impl Into<String>) -> u64 {
        self.push(NotificationType::Error, title, message)
    }

    /// Convenience: push an info notification.
    #[allow(dead_code)]
    pub fn info(&mut self, title: impl Into<String>, message: impl Into<String>) -> u64 {
        self.push(NotificationType::Info, title, message)
    }

    /// Convenience: push a warning notification.
    pub fn warning(&mut self, title: impl Into<String>, message: impl Into<String>) -> u64 {
        self.push(NotificationType::Warning, title, message)
    }
}

/// Retrieve the shared notification state from context.
pub fn use_notifications() -> NotificationState {
    use_context::<NotificationState>()
}

/// Render this once in your root layout — shows toast stack in top-right corner.
#[component]
pub fn NotificationContainer() -> Element {
    let mut state = use_notifications();
    let items = state.items.read().clone();

    rsx! {
        div { class: "notification-container",
            for notif in items.iter().cloned() {
                NotificationToast {
                    key: "{notif.id}",
                    notification: notif.clone(),
                    on_dismiss: move |id: u64| state.dismiss(id),
                }
            }
        }
    }
}

#[component]
fn NotificationToast(notification: Notification, on_dismiss: EventHandler<u64>) -> Element {
    let id = notification.id;
    let duration = notification.duration_ms;

    // Auto-dismiss timer
    let on_dismiss_clone = on_dismiss.clone();
    use_effect(move || {
        if duration > 0 {
            let on_dismiss_inner = on_dismiss_clone.clone();
            spawn(async move {
                gloo_timers::future::TimeoutFuture::new(duration as u32).await;
                on_dismiss_inner.call(id);
            });
        }
    });

    let type_class = match notification.kind {
        NotificationType::Success => "notification-toast--success",
        NotificationType::Error => "notification-toast--error",
        NotificationType::Info => "notification-toast--info",
        NotificationType::Warning => "notification-toast--warning",
    };

    let icon_path = match notification.kind {
        NotificationType::Success => "M22 11.08V12a10 10 0 1 1-5.93-9.14 M22 4 12 14.01l-3-3",
        NotificationType::Error => "M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z M12 9v4 M12 17h.01",
        NotificationType::Info => "M12 2a10 10 0 1 0 0 20 10 10 0 0 0 0-20z M12 16v-4 M12 8h.01",
        NotificationType::Warning => "M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z M12 9v4 M12 17h.01",
    };

    rsx! {
        div { class: "notification-toast {type_class}",
            div { class: "notification-toast-icon",
                svg {
                    xmlns: "http://www.w3.org/2000/svg",
                    width: "18",
                    height: "18",
                    view_box: "0 0 24 24",
                    fill: "none",
                    stroke: "currentColor",
                    stroke_width: "2",
                    stroke_linecap: "round",
                    stroke_linejoin: "round",
                    path { d: "{icon_path}" }
                }
            }
            div { class: "notification-toast-body",
                if !notification.title.is_empty() {
                    div { class: "notification-toast-title", "{notification.title}" }
                }
                div { class: "notification-toast-message", "{notification.message}" }
            }
            button {
                class: "notification-toast-close",
                onclick: move |_| on_dismiss.call(id),
                "×"
            }
        }
    }
}
