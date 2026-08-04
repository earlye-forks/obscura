//! Regression test for `Page::id()` on the public `obscura` facade: the
//! stable page/tab identifier already carried by `obscura-browser::Page`
//! must be reachable from an embedding consumer, and must actually identify
//! the page (distinct pages get distinct ids).

use obscura::Browser;

#[tokio::test]
async fn page_id_is_non_empty_and_unique_per_page() {
    let browser = Browser::new().unwrap();

    let page_a = browser.new_page().await.unwrap();
    let page_b = browser.new_page().await.unwrap();

    assert!(!page_a.id().is_empty());
    assert!(!page_b.id().is_empty());
    assert_ne!(page_a.id(), page_b.id());
}
