use csv::Error as ErrorCsv;

// Get data from csv format file
pub fn csv_to_vec(path: &String) -> Result<Vec<Vec<String>>, ErrorCsv> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_path(path)?;

    let mut csv_data: Vec<Vec<String>> = Vec::new();

    for record in reader.records() {
        csv_data.push(record.unwrap().iter().map(String::from).collect());
    }

    Ok(csv_data)
}
