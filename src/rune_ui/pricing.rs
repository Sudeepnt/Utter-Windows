//! In-app plans and purchase previews. Checkout is not connected yet.

use rune::{El, TextStyle, col, text};

use super::Ui;
use super::controls::{ButtonKind, METRICS, Paint, button, page, raised};
use crate::desktop::i18n::tr;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Plan {
    Monthly,
    Lifetime,
}

impl Plan {
    fn key(self) -> &'static str {
        match self {
            Self::Monthly => "monthly",
            Self::Lifetime => "lifetime",
        }
    }

    fn name(self) -> &'static str {
        match self {
            Self::Monthly => tr("Monthly"),
            Self::Lifetime => tr("Lifetime"),
        }
    }

    fn price(self) -> &'static str {
        match self {
            Self::Monthly => "US$0.50",
            Self::Lifetime => "US$50",
        }
    }

    fn interval(self) -> &'static str {
        match self {
            Self::Monthly => tr("per month"),
            Self::Lifetime => tr("one payment"),
        }
    }

    fn description(self) -> &'static str {
        match self {
            Self::Monthly => tr("Fifty US cents, billed monthly."),
            Self::Lifetime => tr("Keep using Utter with a one-time purchase."),
        }
    }
}

pub(super) fn pane(ui: &Ui, p: Paint) -> El {
    let content = page("pricing-content")
        .child(
            col()
                .key("intro")
                .px(METRICS.row_px)
                .gap(6.0)
                .child(
                    text(tr("Your voice. Your choice."))
                        .key("title")
                        .role("heading")
                        .level(2)
                        .font(TextStyle::new(24.0, 650.0).lh(30.0))
                        .color(p.text())
                        .wrap(),
                )
                .child(copy(
                    p,
                    "about",
                    tr("Private, on-device dictation. Choose your plan."),
                )),
        )
        .child(plan_card(ui, p, Plan::Monthly))
        .child(plan_card(ui, p, Plan::Lifetime))
        .child(
            copy(
                p,
                "coming-soon",
                tr("Planned pricing in US dollars. Payments are coming soon."),
            )
            .px(METRICS.row_px),
        );
    super::scroll_body(p.k, "pricing-scroll", content)
}

fn copy(p: Paint, key: &str, words: &str) -> El {
    text(words).key(key).font(p.k.ty.ui).color(p.muted()).wrap()
}

fn plan_card(ui: &Ui, p: Paint, plan: Plan) -> El {
    let selected = ui.pricing_plan == Some(plan);
    let mut card = raised(p, plan.key())
        .px(METRICS.row_px)
        .py(18.0)
        .gap(8.0)
        .child(
            text(plan.name())
                .key("name")
                .role("heading")
                .level(2)
                .font(TextStyle::new(17.0, 650.0).lh(24.0))
                .color(p.text()),
        )
        .child(
            text(format!("{} · {}", plan.price(), plan.interval()))
                .key("price")
                .font(TextStyle::new(27.0, 650.0).lh(34.0))
                .color(p.text())
                .wrap(),
        )
        .child(copy(p, "description", plan.description()))
        .child(
            button(
                p,
                &format!("preview-{}", plan.key()),
                match plan {
                    Plan::Monthly => tr("Preview monthly plan"),
                    Plan::Lifetime => tr("Preview lifetime plan"),
                },
                ButtonKind::Secondary,
                move |ui, _| ui.pricing_plan = Some(plan),
            )
            .self_start(),
        );
    if selected {
        card = card.child(
            col()
                .key("purchase-preview")
                .role("status")
                .gap(6.0)
                .pt(8.0)
                .child(
                    text(tr("Purchase preview"))
                        .key("preview-title")
                        .font(p.k.ty.ui)
                        .color(p.text()),
                )
                .child(copy(
                    p,
                    "preview-detail",
                    &format!("{} · {} · {}", plan.name(), plan.price(), plan.interval()),
                ))
                .child(copy(
                    p,
                    "preview-notice",
                    tr("Checkout is coming soon. No payment has been taken."),
                ))
                .child(
                    button(
                        p,
                        "close-preview",
                        tr("Close preview"),
                        ButtonKind::Secondary,
                        |ui, _| ui.pricing_plan = None,
                    )
                    .self_start(),
                ),
        );
    }
    card
}
