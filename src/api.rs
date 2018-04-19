#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}
