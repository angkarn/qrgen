pub fn from_vec(row: Vec<String>, template: &String) -> String {
    let mut output: String = template.to_string();
    for (index_col, col) in row.into_iter().enumerate() {
        let from = format!("{{{}}}", format!("{{{index_col}}}"));
        output = output.replace(&from, &col);
    }
    output
}
