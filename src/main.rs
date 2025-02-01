mod folddb;
mod graphql;
mod setup;
mod tests;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tests::run_example_tests().await
}
