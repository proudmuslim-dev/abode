pub mod posts;
pub mod sections;
pub mod sign_in;
pub mod sign_up;
pub mod submissions;
pub mod utils;

#[cfg(test)]
mod tests {
    use rocket::tokio;

    #[tokio::test]
    async fn no_auth_routes() {
        // TODO: Merge frontend & backend into a single project
    }

    #[tokio::test]
    async fn auth_routes() {
        // See above
    }
}
