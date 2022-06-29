use term_data_table::{Alignment, Cell, Row, Table, TableStyle};

fn main() {
    let table = Table::new()
        .with_style(TableStyle::ELEGANT)
        .with_row(
            Row::new().with_cell(
                Cell::from("This is some centered text")
                    .with_alignment(Alignment::Center)
                    .with_col_span(2),
            ),
        )
        .with_row(
            Row::new()
                .with_cell(Cell::from("This is left aligned text"))
                .with_cell(
                    Cell::from("This is right aligned text").with_alignment(Alignment::Right),
                ),
        )
        .with_row(
            Row::new()
                .with_cell(Cell::from("This is left aligned text"))
                .with_cell(
                    Cell::from("This is right aligned text").with_alignment(Alignment::Right),
                ),
        )
        .with_row(
            Row::new().with_cell(
                Cell::from(
                    "This is some really really really really really really really \
                        really really that is going to wrap to the next line",
                )
                .with_col_span(2),
            ),
        );

    println!("{}", table);
}
