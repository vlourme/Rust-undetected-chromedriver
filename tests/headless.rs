#[cfg(test)]
mod tests {
    use thirtyfour::prelude::ElementQueryable;
    use thirtyfour::By;
    use tokio;
    use undetected_chromedriver::UndetectedWebDriver;

    #[tokio::test]
    async fn test_headless_detection() {
        let mut uwd = UndetectedWebDriver::new().await.unwrap();
        // Capabilities can be tweaked:
        // uwd.capabilities.set_binary("Chromium.app/Contents/MacOS/Chromium").unwrap();
        let driver = uwd.start_driver().await.unwrap();

        driver
            .goto("https://arh.antoinevastel.com/bots/areyouheadless")
            .await
            .unwrap();
        let is_headless = driver.query(By::XPath(r#"//*[@id="res"]/p"#));
        assert_eq!(
            is_headless.first().await.unwrap().text().await.unwrap(),
            "You are not Chrome headless"
        );
        driver.quit().await.unwrap();
    }
}
