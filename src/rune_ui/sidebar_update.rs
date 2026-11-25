//! The sidebar's update: Restart to update, Download update, or a channel
//! switch, and under Restart to update what the staged dev build changed.

use super::sidebar::FOOTER_LINE;
use super::*;

const SHOWN: usize = 3;

/// An available update, as a secondary button with a dot.
pub(super) fn control(ui: &Ui, p: Paint) -> Option<El> {
    let action = match ui.view.as_ref().map(|view| &view.update_status) {
        Some(DesktopUpdateStatus::ReadyToRestart) => Some((
            tr("Restart to update").to_owned(),
            DesktopAction::RestartIntoUpdate,
        )),
        Some(DesktopUpdateStatus::Available) => {
            Some((tr("Download update").to_owned(), DesktopAction::OpenUpdate))
        }
        Some(DesktopUpdateStatus::SwitchAvailable { channel, version }) => Some((
            crate::updater::switch_label(channel, version),
            DesktopAction::ConfirmUpdateSwitch,
        )),
        _ => None,
    };
    // An available update is a state, not the screen's main action.
    action.map(|(label, action)| {
        row()
            .key("sidebar-update-control")
            .gap(8.0)
            .pl(6.0)
            .center_y()
            .child(controls::status_dot(p.accent()))
            .child(controls::button(
                p,
                "sidebar-update",
                &label,
                controls::ButtonKind::Secondary,
                move |ui: &mut Ui, _| ui.send(Request::Desktop(action.clone())),
            ))
    })
}

/// Under Download update on Windows, why this Utter can't update itself and
/// what to do (#1352).
pub(super) fn blocked(ui: &Ui, p: Paint) -> Option<El> {
    let view = ui.view.as_ref()?;
    let why = view
        .update_blocked
        .filter(|_| view.update_status == DesktopUpdateStatus::Available)?;
    Some(
        text(tr(why))
            .key("sidebar-update-blocked")
            .font(p.k.ty.small)
            .color(p.faint())
            .wrap()
            .mt(6.0)
            .px(6.0),
    )
}

/// What the staged dev build changed, while Restart to update shows.
pub(super) fn notes(ui: &Ui, p: Paint) -> Option<El> {
    ui.view
        .as_ref()
        .filter(|view| view.update_status == DesktopUpdateStatus::ReadyToRestart)
        .and_then(|view| view.update_notes.as_ref())
        .and_then(|notes| list(p, notes))
}

/// The first few changes and What's new, which opens the build's entry in
/// the dev changelog.
fn list(p: Paint, notes: &crate::desktop::host::UpdateNotes) -> Option<El> {
    if notes.items.is_empty() {
        return None;
    }
    let c = &p.k.c;
    let url = notes.url.clone();
    let link = row()
        .key("whats-new")
        .h(FOOTER_LINE)
        .px(6.0)
        .center_y()
        .self_start()
        .radius(p.k.radius.sm)
        .role("link")
        .label(tr("What's new"))
        .hover(c.hover)
        .press(c.pressed)
        .focusable(true)
        .focus_ring(2.0, -2.0, c.border_focus)
        .on_press(move |ui: &mut Ui| {
            if ui.live()
                && let Err(error) = crate::desktop::opencode_catalog::open_url(&url)
            {
                ui.error = Some(format!("{error:#}"));
            }
        })
        .child(
            text(tr("What's new"))
                .key("label")
                .font(p.k.ty.small)
                .color(c.text_muted)
                .truncate(),
        );
    let items = notes
        .items
        .iter()
        .take(SHOWN)
        .enumerate()
        .map(|(index, item)| {
            text(format!("· {item}"))
                .key(format!("item{index}"))
                .font(p.k.ty.small)
                .color(p.faint())
                .truncate()
                .px(6.0)
        });
    Some(
        col()
            .key("sidebar-update-notes")
            .mt(6.0)
            .gap(2.0)
            .child(link)
            .children(items),
    )
}

/// A persistent update keeps its sidebar action and returns this reminder
/// ten minutes after dismissal, until the newer app is installed and launched.
#[derive(Default)]
pub(super) struct Reminder {
    pub(super) open: bool,
    due: f64,
    now: f64,
}
impl Reminder {
    fn tick(&mut self, available: bool, now: f64) -> bool {
        self.now = now;
        if !available {
            let changed = self.open;
            self.open = false;
            self.due = 0.0;
            return changed;
        }
        if !self.open && now >= self.due {
            self.open = true;
            return true;
        }
        false
    }
    pub(super) fn snooze(&mut self) {
        self.open = false;
        self.due = self.now + 600.0;
    }
}
pub(super) fn poll(ui: &mut Ui, clock: f64) -> bool {
    let available = ui.view.as_ref().is_some_and(|view| {
        matches!(
            view.update_status,
            DesktopUpdateStatus::Available | DesktopUpdateStatus::ReadyToRestart
        )
    });
    ui.update_reminder.tick(available, clock)
}
pub(super) fn modal(ui: &Ui, p: Paint) -> Option<[El; 2]> {
    if !ui.update_reminder.open || ui.tour.is_some() {
        return None;
    }
    let restart = ui
        .view
        .as_ref()
        .is_some_and(|view| view.update_status == DesktopUpdateStatus::ReadyToRestart);
    let panel=col().key("update-reminder").role("dialog").label(tr("Update Utter"))
        .w(430.0).padding(24.0).gap(18.0).radius(p.k.radius.xl).fill(p.k.c.bg_base).border(p.hair,p.k.c.border_weak)
        .on_press(|_: &mut Ui| {})
        .child(text(tr("A new Utter update is available")).key("title").font(p.k.ty.body_medium).color(p.text()).wrap())
        .child(text(if restart {tr("Restart Utter to finish installing the update.")} else {tr("Download the new version, run its installer, and reopen Utter to finish updating.")}).key("description").font(p.k.ty.ui).color(p.muted()).wrap())
        .child(row().key("actions").gap(12.0)
            .child(controls::button(p,"remind-update",tr("Remind me in 10 minutes"),controls::ButtonKind::Secondary,|ui,_| ui.update_reminder.snooze()))
            .child(controls::button(p,"download-update-now",if restart {tr("Restart to update")} else {tr("Download update")},controls::ButtonKind::Primary,move |ui,_| {
                ui.send(Request::Desktop(if restart {DesktopAction::RestartIntoUpdate} else {DesktopAction::OpenUpdate}));
                ui.update_reminder.snooze();
            }).default_button()));
    let layer = |key| {
        el().key(key)
            .role("none")
            .absolute(0.0, 0.0)
            .w_full()
            .h_full()
    };
    Some([
        layer("update-reminder-scrim")
            .fill(p.k.c.scrim)
            .on_press(|ui: &mut Ui| ui.update_reminder.snooze()),
        layer("update-reminder-layer")
            .center()
            .child(panel.modal(true, |ui: &mut Ui| ui.update_reminder.snooze())),
    ])
}
#[cfg(test)]
mod reminder_tests {
    use super::*;
    #[test]
    fn reminders_return_until_the_update_is_installed() {
        let mut r = Reminder::default();
        assert!(r.tick(true, 10.0));
        assert!(r.open);
        r.snooze();
        assert!(!r.tick(true, 609.0));
        assert!(r.tick(true, 610.0));
        r.tick(false, 611.0);
        assert!(!r.open);
    }
}
