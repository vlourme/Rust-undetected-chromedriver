//! # Rust Undetected ChromeDriver
//! A rust implementation of
//! ultrafunkamsterdam's [undetected-chromedriver](https://github.com/ultrafunkamsterdam/undetected-chromedriver)
//! library based on [thirtyfour](https://github.com/stevepryde/thirtyfour)
//!
//! Get started by calling `UndetectedWebDriver::new_driver()`
//! or by using the `UndetectedWebDriver::new()` method to customize the capabilities.

use rand::Rng;
#[cfg(any(target_os = "linux", target_os = "macos"))]
use std::os::unix::fs::PermissionsExt;
use std::process::Command;
use thirtyfour::{ChromeCapabilities, DesiredCapabilities, WebDriver};

pub struct UndetectedWebDriver {
    pub capabilities: ChromeCapabilities,
}

/// Get a new instance of UndetectedWebDriver.
/// 
/// Prevent breaking changes from v0.1.0
pub async fn chrome() -> Result<WebDriver, Box<dyn std::error::Error>> {
    UndetectedWebDriver::new_driver().await
}

impl UndetectedWebDriver {
    /// Start a new instance of UndetectedWebDriver.
    ///
    /// Capabilities might be changed before executing `start_driver` method.
    ///
    /// Some defaults arguments are already set:
    /// - disable-blink-features
    /// - window-size
    /// - user-agent
    /// - disable-infobars
    pub async fn new() -> Result<UndetectedWebDriver, Box<dyn std::error::Error>> {
        if std::path::Path::new("chromedriver").exists() {
            println!("ChromeDriver already exists!");
        } else {
            println!("ChromeDriver does not exist! Fetching...");
            let client = reqwest::Client::new();
            Self::fetch_chromedriver(&client).await?;
        }

        Self::patch_chromedriver();

        let mut caps = DesiredCapabilities::chrome();
        caps.set_no_sandbox()?;
        caps.set_disable_dev_shm_usage()?;
        caps.add_chrome_arg("--disable-blink-features=AutomationControlled")?;
        caps.add_chrome_arg("window-size=1920,1080")?;
        caps.add_chrome_arg("user-agent=Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/102.0.0.0 Safari/537.36")?;
        caps.add_chrome_arg("disable-infobars")?;
        caps.add_chrome_option("excludeSwitches", ["enable-automation"])?;

        Ok(UndetectedWebDriver { capabilities: caps })
    }

    /// Start the driver directly without creating
    /// an instance of UndetectedWebDriver.
    ///
    /// This can be useful if you want to use the default
    /// capabilities.
    pub async fn new_driver() -> Result<WebDriver, Box<dyn std::error::Error>> {
        let mut uwd = Self::new().await?;
        uwd.start_driver().await
    }

    /// Start the driver and pops up a browser window.
    pub async fn start_driver(&mut self) -> Result<WebDriver, Box<dyn std::error::Error>> {
        println!("Starting chromedriver...");

        let port: usize = rand::thread_rng().gen_range(2000..5000);
        Command::new(format!("./{}", Self::get_chrome_path()))
            .arg(format!("--port={}", port))
            .spawn()
            .expect("Failed to start chromedriver!");

        let mut driver = None;
        let mut attempt = 0;
        while driver.is_none() && attempt < 20 {
            attempt += 1;
            match WebDriver::new(
                &format!("http://localhost:{}", port),
                self.capabilities.clone(),
            )
            .await
            {
                Ok(d) => driver = Some(d),
                Err(_) => std::thread::sleep(std::time::Duration::from_millis(250)),
            }
        }

        Ok(driver.unwrap())
    }

    /// Get the path to the chromedriver executable.
    fn get_chrome_path() -> &'static str {
        let os = std::env::consts::OS;

        match os {
            "linux" | "macos" => "chromedriver_PATCHED",
            "windows" => "chromedriver_PATCHED.exe",
            _ => panic!("Unsupported OS!"),
        }
    }

    /// Patch the chromedriver executable.
    fn patch_chromedriver() {
        let chromedriver_executable = Self::get_chrome_path();
        match !std::path::Path::new(chromedriver_executable).exists() {
            true => {
                println!("Starting ChromeDriver executable patch...");

                let file_name = match cfg!(windows) {
                    true => "chromedriver.exe",
                    false => "chromedriver",
                };

                let f = std::fs::read(file_name).unwrap();
                let mut new_chromedriver_bytes = f.clone();
                let mut total_cdc = String::from("");
                let mut cdc_pos_list = Vec::new();
                let mut is_cdc_present = false;
                let mut patch_ct = 0;
                for i in 0..f.len() - 3 {
                    if "cdc_"
                        == format!(
                            "{}{}{}{}",
                            f[i] as char,
                            f[i + 1] as char,
                            f[i + 2] as char,
                            f[i + 3] as char
                        )
                        .as_str()
                    {
                        for x in i..i + 22 {
                            total_cdc.push_str(&(f[x] as char).to_string());
                        }
                        is_cdc_present = true;
                        cdc_pos_list.push(i);
                        total_cdc = String::from("");
                    }
                }
                match is_cdc_present {
                    true => println!("Found cdcs!"),
                    false => println!("No cdcs were found!"),
                }
                let get_random_char = || -> char {
                    "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ"
                        .chars()
                        .collect::<Vec<char>>()[rand::thread_rng().gen_range(0..48)]
                };
                for i in cdc_pos_list {
                    for x in i..i + 22 {
                        new_chromedriver_bytes[x] = get_random_char() as u8;
                    }
                    patch_ct += 1;
                }
                println!("Patched {} cdcs!", patch_ct);

                println!("Starting to write to binary file...");
                let _file = std::fs::File::create(chromedriver_executable).unwrap();
                match std::fs::write(chromedriver_executable, new_chromedriver_bytes) {
                    Ok(_res) => {
                        println!("Successfully wrote patched executable to 'chromedriver_PATCHED'!",)
                    }
                    Err(err) => println!("Error when writing patch to file! Error: {}", err),
                };
            }
            false => {
                println!("Detected patched chromedriver executable!");
            }
        }

        #[cfg(any(target_os = "linux", target_os = "macos"))]
        {
            let mut perms = std::fs::metadata(chromedriver_executable)
                .unwrap()
                .permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(chromedriver_executable, perms).unwrap();
        }
    }

    /// Fetch the latest chromedriver executable.
    async fn fetch_chromedriver(
        client: &reqwest::Client,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let os = std::env::consts::OS;
        let resp = client
            .get("https://chromedriver.storage.googleapis.com/LATEST_RELEASE")
            .send()
            .await?;
        let body = resp.text().await?;
        let url = match os {
            "linux" => format!(
                "https://chromedriver.storage.googleapis.com/{}/chromedriver_linux64.zip",
                body
            ),
            "windows" => format!(
                "https://chromedriver.storage.googleapis.com/{}/chromedriver_win32.zip",
                body
            ),
            "macos" => format!(
                "https://chromedriver.storage.googleapis.com/{}/chromedriver_mac64.zip",
                body
            ),
            _ => panic!("Unsupported OS!"),
        };
        let resp = client.get(url).send().await?;
        let body = resp.bytes().await?;

        let mut archive = zip::ZipArchive::new(std::io::Cursor::new(body))?;
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let outpath = file.mangled_name();
            if (file.name()).ends_with('/') {
                std::fs::create_dir_all(outpath)?;
            } else {
                if let Some(p) = outpath.parent() {
                    if !p.exists() {
                        std::fs::create_dir_all(p)?;
                    }
                }
                let mut outfile = std::fs::File::create(outpath)?;
                std::io::copy(&mut file, &mut outfile)?;
            }
        }

        Ok(())
    }
}
