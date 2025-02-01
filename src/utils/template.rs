pub fn from_vec(row: Vec<String>, template: &String, row_index: usize) -> String {
    let mut output: String = template.to_string();
    for (index_col, col) in row.into_iter().enumerate() {
        output = output.replace(&format!("{{{}}}", format!("{{{}}}", index_col + 1)), &col);
        output = output.replace(&"{{ROW}}", &(row_index + 1).to_string());
    }
    output
}
