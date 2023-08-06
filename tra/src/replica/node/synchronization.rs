use async_recursion::async_recursion;

use crate::MyResult;

use super::Node;

// recursive methods
impl Node {
    #[async_recursion]
    pub async fn handle_sync() -> MyResult<()> {
        Ok(())
    }
}
