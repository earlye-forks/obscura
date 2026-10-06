//! `Element` holds a `Weak` to its page, so it keeps working after the `Page`
//! is moved, and reports `PageDropped` instead of dangling once it is dropped.

use obscura::{Browser, Error};

#[tokio::test]
async fn element_survives_page_move() {
    let browser = Browser::new().unwrap();
    let mut page = browser.new_page().await.unwrap();
    page.goto("data:text/html,<div id=x data-k=v>hi</div>")
        .await
        .unwrap();

    let el = page.query_selector("#x").expect("element #x should be found");
    let mut pages = vec![page];
    assert_eq!(el.text().unwrap(), "hi");
    assert_eq!(el.attribute("data-k").unwrap().as_deref(), Some("v"));

    // The page stays usable while the element is alive.
    pages[0].goto("data:text/html,<p>other</p>").await.unwrap();
}

#[tokio::test]
async fn element_reports_dropped_page() {
    let browser = Browser::new().unwrap();
    let mut page = browser.new_page().await.unwrap();
    page.goto("data:text/html,<div id=x>hi</div>").await.unwrap();

    let el = page.query_selector("#x").expect("element #x should be found");
    drop(page);
    assert!(matches!(el.text(), Err(Error::PageDropped)));
    assert!(matches!(el.attribute("id"), Err(Error::PageDropped)));
    assert!(matches!(el.click(), Err(Error::PageDropped)));
}
