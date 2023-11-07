

pub fn run(s: &str) -> Result<String, &'static str> {
    Ok(format!("success; scanning: |{s}|"))
//    Err("There is something wrong")
}
