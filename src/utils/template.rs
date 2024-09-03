pub fn from_vec(row: Vec<String>, template: &String, row_index: usize) -> String {
    let mut output: String = template.to_string();
    for (index_col, col) in row.into_iter().enumerate() {
        let from_index_col = format!("{{{}}}", format!("{{{index_col}}}"));
        output = output.replace(&from_index_col, &col);
        output = output.replace(&"{{ROW}}", &(row_index + 1).to_string());
    }
    output
}
