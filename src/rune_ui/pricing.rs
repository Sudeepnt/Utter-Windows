//! Pricing is informational while Utter is free.
use super::Ui;
use super::controls::{METRICS, Paint, page, raised};
use crate::desktop::i18n::tr;
use rune::{El, TextStyle, col, text};

pub(super) fn pane(_ui: &Ui, p: Paint) -> El {
    let content = page("pricing-content").child(
        raised(p, "free-plan")
            .px(METRICS.row_px)
            .py(28.0)
            .gap(16.0)
            .child(
                text(tr("Free right now"))
                    .key("free-title")
                    .role("heading")
                    .level(2)
                    .font(TextStyle::new(30.0, 650.0).lh(38.0))
                    .color(p.text()),
            )
            .child(
                text("US$0")
                    .key("free-price")
                    .font(TextStyle::new(42.0, 650.0).lh(50.0))
                    .color(p.text()),
            )
            .child(
                text(tr(
                    "You can use Utter without a purchase. Paid plans are not active.",
                ))
                .key("free-description")
                .font(p.k.ty.ui)
                .color(p.muted())
                .wrap(),
            )
            .child(
                text(tr(
                    "If pricing changes in the future, we will tell you before asking for payment.",
                ))
                .key("free-note")
                .font(p.k.ty.small)
                .color(p.muted())
                .wrap(),
            ),
    );
    super::scroll_body(p.k, "pricing-scroll", col().child(content))
}
