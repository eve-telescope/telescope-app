//! Column layout shared by a table's header and its rows, standing in for the
//! CSS grid templates the Vue app used.

use gpui_kit::{Div, Styled as _, div, px};

#[derive(Clone, Copy)]
pub enum Col {
    Fixed(f32),
    /// Minimum width and flex weight, like `minmax(min, weight fr)`.
    Grow(f32, f32),
}

pub fn cell(col: Col) -> Div {
    let el = div().flex().items_center().overflow_hidden().flex_none();
    match col {
        Col::Fixed(width) => el.w(px(width)),
        Col::Grow(min, weight) => {
            // Flexible columns give up to 40% of their width before the fixed
            // ones get clipped.
            let mut el = el.min_w(px(min * 0.6)).flex_basis(px(min));
            let style = el.style();
            style.flex_grow = Some(weight);
            style.flex_shrink = Some(1.);
            el
        }
    }
}
