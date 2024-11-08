pub async fn respond_ask() -> anyhow::Result<String> {
    Ok("Don't ask to ask, just ask! \nhttps://dontasktoask.com/".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_respond_ask() {
        let response = respond_ask().await.unwrap();
        assert_eq!(
            response,
            "Don't ask to ask, just ask! \nhttps://dontasktoask.com/"
        );
    }
}