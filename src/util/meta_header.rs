const NUM_COLUMNS: usize = 4;
const NAME_VALUE_DELIMITER: &str = ": ";
const HEADER_DELIMITER: &str = ", ";

pub fn align_headers(headers: Vec<String>) -> Vec<String> {
    let mut column_widths = [0usize; NUM_COLUMNS];
    let mut column = 0usize;

    for header in headers.iter() {
        if let Some((name, value)) = header.split_once(NAME_VALUE_DELIMITER) {
            for i in [name, value] {
                let width = i.trim().len();
                if width > column_widths[column] {
                    column_widths[column] = width;
                }
                column = (column + 1) % NUM_COLUMNS;
            }
        }
    }

    let mut lines = Vec::new();
    column = 0;
    for i in 0..headers.len() {
        let header = &headers[i];
        if column == 0 {
            lines.push(String::new());
        }
        if let Some((name, value)) = header.split_once(NAME_VALUE_DELIMITER) {
            let name_trimmed = name.trim();
            *lines.last_mut().unwrap() += name_trimmed;
            *lines.last_mut().unwrap() += NAME_VALUE_DELIMITER;
            *lines.last_mut().unwrap() += &" ".repeat(column_widths[column] - name_trimmed.len());
            column = (column + 1) % NUM_COLUMNS;

            let value_trimmed = value.trim();
            *lines.last_mut().unwrap() += value_trimmed;
            if column < NUM_COLUMNS - 1 && i < headers.len() - 1 {
                *lines.last_mut().unwrap() +=
                    &" ".repeat(column_widths[column] - value_trimmed.len());
                *lines.last_mut().unwrap() += HEADER_DELIMITER;
            }
            column = (column + 1) % NUM_COLUMNS;
        }
    }

    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn align_headers_even_test() {
        let headers = vec![
            "Foo: Bar".to_string(),
            "Hello: World".to_string(),
            "Dunder: Mifflin".to_string(),
            "Wernham: Hogg".to_string(),
        ];
        let aligned = align_headers(headers);
        assert_eq!(2, aligned.len());
        assert_eq!("Foo:    Bar    , Hello:   World".to_string(), aligned[0]);
        assert_eq!("Dunder: Mifflin, Wernham: Hogg".to_string(), aligned[1]);
    }

    #[test]
    fn align_headers_odd_test() {
        let headers = vec![
            "Foo: Bar".to_string(),
            "Hello: World".to_string(),
            "Dunder: Mifflin".to_string(),
        ];
        let aligned = align_headers(headers);
        assert_eq!(2, aligned.len());
        assert_eq!("Foo:    Bar    , Hello: World".to_string(), aligned[0]);
        assert_eq!("Dunder: Mifflin".to_string(), aligned[1]);
    }
}
