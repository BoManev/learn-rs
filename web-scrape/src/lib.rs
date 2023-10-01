use serde::Serialize;
use std::error::Error;
use std::thread;
use std::time::Duration;
use thirtyfour::{
    prelude::{ElementWaitable, WebDriverError},
    By, DesiredCapabilities, WebDriver, WebElement,
};
use url::Url;

pub async fn scrape_entry(location: &str) -> Result<(), Box<dyn Error>> {
    let driver = driver_init().await?;
    let url = Url::parse("https://www.airbnb.it/")?;
    driver.goto(url).await?;
    // wait for page to load
    thread::sleep(Duration::from_secs(3));

    location_field(&driver).await?;
    input_location(&driver, location).await?;
    search_location(&driver).await?;
    thread::sleep(Duration::from_secs(3));

    driver
        .execute("window.scrollTo(0, document.body.scrollHeight);", vec![])
        .await?;
    thread::sleep(Duration::from_secs(1));
    let mut writer = csv::Writer::from_path("airbnb.csv")?;

    loop {
        if let Ok(next_btn) = driver.find(By::Css("#site-content > div > div.p1szzjq8.dir.dir-ltr > div > div > div > nav > div > a.l1ovpqvx.c1ytbx3a.dir.dir-ltr")).await {
            if next_btn.is_clickable().await? {
                 for el in driver.find_all(By::Css("#site-content > div > div:nth-child(2) > div > div > div > div > div.gsgwcjk.g8ge8f1.g14v8520.dir.dir-ltr > div.dir.dir-ltr > div > div.c1l1h97y.dir.dir-ltr > div > div > div > div.cy5jw6o.dir.dir-ltr > div > div.g1qv1ctd.c1v0rf5q.dir.dir-ltr")).await? {
                    let prop = Property::from(el).await?;
                    writer.serialize(prop)?;
                 }
                 // goto next page
                 next_btn.click().await?;
                 thread::sleep(Duration::from_secs(2));
             
                 driver
                     .execute("window.scrollTo(0, document.body.scrollHeight);", vec![])
                     .await?;
                 thread::sleep(Duration::from_secs(1));
            } else {
                break;
            }
        } else {
            for el in driver.find_all(By::Css("#site-content > div > div:nth-child(2) > div > div > div > div > div.gsgwcjk.g8ge8f1.g14v8520.dir.dir-ltr > div.dir.dir-ltr > div > div.c1l1h97y.dir.dir-ltr > div > div > div > div.cy5jw6o.dir.dir-ltr > div > div.g1qv1ctd.c1v0rf5q.dir.dir-ltr")).await? {
                let prop = Property::from(el).await?;
                writer.serialize(prop)?;
            }

        }
    }

    Ok(())
}

async fn driver_init() -> Result<WebDriver, WebDriverError> {
    let capabilities = DesiredCapabilities::chrome();
    let driver = WebDriver::new("http://localhost:9515", capabilities).await?;
    driver.maximize_window().await?;
    Ok(driver)
}

async fn location_field(driver: &WebDriver) -> Result<(), WebDriverError> {
    driver
        .find(By::Css("body > div:nth-child(8) > div > div > div:nth-child(1) > div > div.cd56ld.cb80sj1.dir.dir-ltr > div.h1ta6hky.dir.dir-ltr > div > div > div > header > div > div.cb994eh.dir.dir-ltr > div.lkm6i7z.lr5v90m.l1rzxhu2.l1kj223i.dir.dir-ltr > div > span.ij8oydg.dir.dir-ltr > button:nth-child(1)"))
        .await?.click().await?;

    Ok(())
}

async fn input_location(driver: &WebDriver, location: &str) -> Result<(), WebDriverError> {
    let input = driver
        .find(By::Css("#bigsearch-query-location-input"))
        .await?;
    input.wait_until().clickable().await?;

    input.send_keys(location).await?;

    Ok(())
}

async fn search_location(driver: &WebDriver) -> Result<(), WebDriverError> {
    driver.find(By::Css("#search-tabpanel > div.i1flv5qo.dir.dir-ltr > div.c6ezw63.c1geg2ah.dir.dir-ltr > div.c192dx2b.ckzf1ch.dir.dir-ltr > div.s31emu3.dir.dir-ltr > button")).await?.click().await?;
    Ok(())
}

#[derive(Debug, Serialize)]
struct Property {
    title: String,
    desc: String,
    host: String,
    availability: String,
    star: String,
    price: String,
}

impl Property {
    async fn from(el: WebElement) -> Result<Self, WebDriverError> {
        let title = el.find(By::Css("div:nth-child(1)")).await?.text().await?;
        let desc = el
            .find(By::Css("div:nth-child(2) > span"))
            .await?
            .text()
            .await?;
        let host = if let Ok(host) = el.find(By::Css("div:nth-child(3) > span > span")).await {
            host.text().await?
        } else {
            el.find(By::Css("div:nth-child(3) > span"))
                .await?
                .text()
                .await?
        };
        
        let availability = el
            .find(By::Css("div:nth-child(4) > span > span"))
            .await?
            .text()
            .await?;
        let price = el
            .find(By::XPath("div[5]/div/div/span[1]/div/span[1]"))
            .await?
            .text()
            .await?;
        eprintln!("Here");
        let star = if let Ok(star) = el.find(By::Css("span > span.r1dxllyb.dir.dir-ltr")).await {
            star.text().await?
        } else {
            "no rating".into()
        };
        
        Ok(Self {
            title,
            desc,
            host,
            availability,
            price,
            star,
        })
    }
}
