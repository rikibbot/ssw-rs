use ssw_html::{Markup, html};

/// The supported semantic variants for badge components.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BadgeVariant {
    /// A neutral badge for quiet metadata or default state labels.
    Neutral,
    /// A badge for informational or in-progress states.
    Info,
    /// A badge for successful or healthy states.
    Success,
    /// A badge for warning or attention states.
    Warning,
    /// A badge for error or destructive states.
    Danger,
}

impl BadgeVariant {
    /// Returns a stable lowercase variant identifier for styling hooks.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Neutral => "neutral",
            Self::Info => "info",
            Self::Success => "success",
            Self::Warning => "warning",
            Self::Danger => "danger",
        }
    }
}

/// A single cell in a data table row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableCell {
    content: Markup,
    row_header: bool,
}

impl TableCell {
    /// Creates a standard body cell.
    pub fn new(content: impl Into<Markup>) -> Self {
        Self {
            content: content.into(),
            row_header: false,
        }
    }

    /// Creates a row-header cell for the leading label in a row.
    pub fn row_header(content: impl Into<Markup>) -> Self {
        Self {
            content: content.into(),
            row_header: true,
        }
    }
}

/// A single row in a data table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableRow {
    cells: Vec<TableCell>,
}

impl TableRow {
    /// Creates a row from ordered cells.
    pub fn new(cells: Vec<TableCell>) -> Self {
        Self { cells }
    }

    /// Returns the cells in display order.
    pub fn cells(&self) -> &[TableCell] {
        &self.cells
    }
}

/// Renders a badge with the default neutral variant.
pub fn badge(content: impl Into<Markup>) -> Markup {
    badge_with_variant(content, BadgeVariant::Neutral)
}

/// Renders a badge with an explicit semantic variant.
pub fn badge_with_variant(content: impl Into<Markup>, variant: BadgeVariant) -> Markup {
    let variant_name = variant.as_str();

    html! {
        span class="ssw-badge" data_variant=(variant_name) {
            (content.into())
        }
    }
}

/// Renders a simple data table with column headers and rows.
pub fn data_table(columns: &[&str], rows: &[TableRow]) -> Markup {
    html! {
        div class="ssw-table-wrapper" {
            table class="ssw-table" {
                thead class="ssw-table__head" {
                    tr class="ssw-table__row" {
                        @for column in columns {
                            th class="ssw-table__heading" scope="col" {
                                (column)
                            }
                        }
                    }
                }
                tbody class="ssw-table__body" {
                    @for row in rows {
                        tr class="ssw-table__row" {
                            @for cell in row.cells() {
                                ({
                                    if cell.row_header {
                                        html! {
                                            th class="ssw-table__cell ssw-table__cell--row-header" scope="row" {
                                                (cell.content.clone())
                                            }
                                        }
                                    } else {
                                        html! {
                                            td class="ssw-table__cell" {
                                                (cell.content.clone())
                                            }
                                        }
                                    }
                                })
                            }
                        }
                    }
                }
            }
        }
    }
}
