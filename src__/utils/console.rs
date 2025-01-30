use qrcode_generator::QrCodeEcc;

// Print the given qrcode object to the console
fn from_vec_bool(qr: &Vec<Vec<bool>>) {
    let symbol: char = '█';

    let print_border = || {
        for _ in 0..&qr.len() + 2 {
            print!("{0}{0}", symbol);
        }
    };

    print_border();
    println!();

    for y in qr {
        for (x_index, x) in y.iter().enumerate() {
            if x_index == 0 {
                print!("{0}{0}", symbol);
            }
            let c: char = if !*x { symbol } else { ' ' };
            print!("{0}{0}", c);
            if x_index + 1 == y.len() {
                print!("{0}{0}", symbol);
            }
        }
        println!();
    }

    print_border();
    println!();
}

pub fn print_qr(content: &String) {
    println!("{}", content);
    let result: Vec<Vec<bool>> = qrcode_generator::to_matrix(content, QrCodeEcc::Low).unwrap();
    from_vec_bool(&result);
    println!();
}
