//! `Grid`: Auto, star and pixel columns, an Auto row over a star row, spacing and a column span.
use crate::{label, section};
use reveal_embedder::Size;
use reveal_widgets::*;
use reveal_winui::*;

pub fn build(resources: &ThemeResources) -> Vec<WidgetRef> {
    let text = resources.common.text_fill_color_primary;
    let cell = |name: &str, brush: Brush| {
        ControlBorder::new(
            brush,
            Brush::Solid(resources.common.control_stroke_color_default),
        )
        .corner_radius(CONTROL_CORNER_RADIUS[0])
        .padding([8.0, 4.0, 8.0, 4.0])
        .child(label(name, TextBlockStyle::Caption, text))
    };
    let accent = Brush::Solid(resources.accent.light3);
    let fill = Brush::Solid(resources.common.control_fill_color_default);
    let grid = Grid::new()
        .column_definitions([
            ColumnDefinition::new(GridLength::AUTO),
            ColumnDefinition::new(GridLength::STAR),
            ColumnDefinition::new(GridLength::pixel(80.0)),
        ])
        .row_definitions([
            RowDefinition::new(GridLength::AUTO),
            RowDefinition::new(GridLength::STAR),
        ])
        .row_spacing(8.0)
        .column_spacing(8.0)
        .children([
            GridCell::new(cell("Auto", fill)).into_widget(),
            GridCell::new(cell("*", fill)).column(1).into_widget(),
            GridCell::new(cell("80 px", fill)).column(2).into_widget(),
            GridCell::new(cell("Row 1, ColumnSpan 2", accent))
                .row(1)
                .column_span(2)
                .into_widget(),
            GridCell::new(cell("Row 1", fill))
                .row(1)
                .column(2)
                .into_widget(),
        ]);
    section(
        "Grid",
        resources,
        SizedBox::from_size(Some(Size::new(420.0, 100.0)))
            .child(grid)
            .into_widget(),
    )
}
